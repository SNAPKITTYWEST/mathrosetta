use crate::{MathIR, Variable, Constant, Domain, AssumptionSet, SymbolicConst, Dir};
use super::{Parser, ParseError};

pub struct LatexParser;

impl Parser for LatexParser {
    fn parse(&self, input: &str) -> Result<MathIR, ParseError> {
        let trimmed = input.trim();

        // Try equation first: lhs = rhs
        if let Some(eq_pos) = find_top_level_eq(trimmed) {
            let lhs = LatexStream::new(trimmed[..eq_pos].trim()).parse_expr()?;
            let rhs = LatexStream::new(trimmed[eq_pos + 1..].trim()).parse_expr()?;
            return Ok(MathIR::Eq(Box::new(lhs), Box::new(rhs)));
        }

        LatexStream::new(trimmed).parse_expr()
    }

    fn format_name(&self) -> &str {
        "latex"
    }
}

// ── Token types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Cmd(String),         // \command
    Brace(String),       // { ... }
    Bracket(String),     // [ ... ]
    Char(char),          // single character (letter, digit, operator)
    Sub,                 // _
    Sup,                 // ^
    Lp,                  // (
    Rp,                  // )
    Comma,               // ,
    End,                 // end of input
}

// ── Lexer ────────────────────────────────────────────────────────────────────

struct LatexLexer {
    chars: Vec<char>,
    pos: usize,
}

impl LatexLexer {
    fn new(input: &str) -> Self {
        Self { chars: input.chars().collect(), pos: 0 }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_spacing();
        if self.pos >= self.chars.len() {
            return Token::End;
        }
        let c = self.chars[self.pos];
        match c {
            '{' => {
                self.pos += 1;
                let content = self.read_balanced('{', '}');
                Token::Brace(content)
            }
            '[' => {
                self.pos += 1;
                let content = self.read_balanced('[', ']');
                Token::Bracket(content)
            }
            '_' => { self.pos += 1; Token::Sub }
            '^' => { self.pos += 1; Token::Sup }
            '(' => { self.pos += 1; Token::Lp }
            ')' => { self.pos += 1; Token::Rp }
            ',' => { self.pos += 1; Token::Comma }
            '\\' => {
                self.pos += 1;
                if self.pos >= self.chars.len() {
                    return Token::Cmd("\\".to_string());
                }
                let next = self.chars[self.pos];
                if next == '\\' {
                    self.pos += 1;
                    return Token::Char('\\');
                }
                // Read command name
                let mut name = String::new();
                if next.is_alphabetic() {
                    while self.pos < self.chars.len() && self.chars[self.pos].is_alphabetic() {
                        name.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                } else {
                    // Single non-alpha command like \, \; \! \>
                    name.push(next);
                    self.pos += 1;
                }
                Token::Cmd(name)
            }
            _ => {
                self.pos += 1;
                Token::Char(c)
            }
        }
    }

    fn skip_whitespace_and_spacing(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.pos += 1;
            } else if c == '\\' && self.pos + 1 < self.chars.len() {
                // Skip LaTeX spacing commands: \, \; \! \> \quad \qquad \negthinspace etc.
                let next = self.chars[self.pos + 1];
                if matches!(next, ',' | ';' | '!' | '>' | ' ') {
                    self.pos += 2;
                } else if self.read_command_name_at(self.pos + 1).map_or(false, |n|
                    n == "quad" || n == "qquad" || n == "negthinspace" || n == "thinspace"
                        || n == "enspace" || n == "medspace" || n == "thickspace"
                ) {
                    let name = self.read_command_name_at(self.pos + 1).unwrap();
                    self.pos += 1 + name.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_command_name_at(&self, start: usize) -> Option<String> {
        if start >= self.chars.len() || !self.chars[start].is_alphabetic() {
            return None;
        }
        let mut name = String::new();
        let mut i = start;
        while i < self.chars.len() && self.chars[i].is_alphabetic() {
            name.push(self.chars[i]);
            i += 1;
        }
        Some(name)
    }

    fn read_balanced(&mut self, open: char, close: char) -> String {
        let mut depth = 1;
        let mut content = String::new();
        while self.pos < self.chars.len() && depth > 0 {
            let c = self.chars[self.pos];
            if c == open { depth += 1; }
            if c == close { depth -= 1; }
            if depth > 0 { content.push(c); }
            self.pos += 1;
        }
        content
    }
}

// ── Stream parser ────────────────────────────────────────────────────────────

struct LatexStream {
    lexer: LatexLexer,
    peeked: Option<Token>,
}

impl LatexStream {
    fn new(input: &str) -> Self {
        Self { lexer: LatexLexer::new(input), peeked: None }
    }

    fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        self.peeked.as_ref().unwrap()
    }

    fn advance(&mut self) -> Token {
        if let Some(t) = self.peeked.take() {
            t
        } else {
            self.lexer.next_token()
        }
    }

    fn expect_brace(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Brace(s) => Ok(s),
            t => Err(ParseError::Invalid(format!("Expected {{}}, got {:?}", t))),
        }
    }

    // ── Expression level (handles + and -) ──────────────────────────────────

    fn parse_expr(&mut self) -> Result<MathIR, ParseError> {
        let mut left = self.parse_term()?;

        loop {
            match self.peek() {
                Token::Char('+') => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = match left {
                        MathIR::Add(mut args) => { args.push(right); MathIR::Add(args) }
                        _ => MathIR::Add(vec![left, right]),
                    };
                }
                Token::Char('-') => {
                    self.advance();
                    let right = self.parse_term()?;
                    let neg_right = MathIR::Mul(vec![
                        MathIR::Const(Constant::Int(-1)),
                        right,
                    ]);
                    left = match left {
                        MathIR::Add(mut args) => { args.push(neg_right); MathIR::Add(args) }
                        _ => MathIR::Add(vec![left, neg_right]),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // ── Term level (handles implicit multiplication and \frac) ──────────────

    fn parse_term(&mut self) -> Result<MathIR, ParseError> {
        let mut factors = Vec::new();
        factors.push(self.parse_power()?);

        loop {
            let can_mul = match self.peek() {
                Token::Char(c) if c.is_alphabetic() => true,
                Token::Lp => true,
                Token::Brace(_) => true,
                Token::Cmd(_) => true,
                _ => false,
            };
            if !can_mul { break; }
            factors.push(self.parse_power()?);
        }

        if factors.len() == 1 {
            Ok(factors.into_iter().next().unwrap())
        } else {
            Ok(MathIR::Mul(factors))
        }
    }

    // ── Power level (handles ^) ────────────────────────────────────────────

    fn parse_power(&mut self) -> Result<MathIR, ParseError> {
        let base = self.parse_primary()?;

        if matches!(self.peek(), Token::Sup) {
            self.advance();
            let exp = self.parse_primary()?;
            return Ok(MathIR::Pow(Box::new(base), Box::new(exp)));
        }

        Ok(base)
    }

    // ── Primary level (atoms, functions, commands) ──────────────────────────

    fn parse_primary(&mut self) -> Result<MathIR, ParseError> {
        match self.peek().clone() {
            Token::Brace(content) => {
                self.advance();
                LatexStream::new(&content).parse_expr()
            }
            Token::Lp => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_rp()?;
                Ok(expr)
            }
            Token::Char(c) if c.is_ascii_digit() => {
                self.advance();
                let mut num = c.to_string();
                while let Token::Char(c) = self.peek() {
                    if c.is_ascii_digit() || *c == '.' {
                        num.push(*c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                if num.contains('.') {
                    Ok(MathIR::Const(Constant::Float(num.parse().unwrap_or(0.0))))
                } else {
                    Ok(MathIR::Const(Constant::Int(num.parse().unwrap_or(0))))
                }
            }
            Token::Char(c) if c.is_alphabetic() => {
                self.advance();
                let id = c.to_string();
                Ok(MathIR::Var(Box::new(Variable {
                    id,
                    domain: Domain::Real,
                    assumptions: AssumptionSet::default(),
                })))
            }
            Token::Cmd(name) => {
                self.advance();
                self.parse_command(&name)
            }
            Token::Char('\\') => {
                self.advance();
                Ok(MathIR::Const(Constant::Symbolic(SymbolicConst::Pi)))
            }
            t => Err(ParseError::Invalid(format!("Unexpected token: {:?}", t))),
        }
    }

    // ── Command dispatcher ──────────────────────────────────────────────────

    fn parse_command(&mut self, name: &str) -> Result<MathIR, ParseError> {
        match name {
            // ── Fractions ────────────────────────────────────────────────
            "frac" | "dfrac" | "tfrac" => {
                let num = LatexStream::new(&self.expect_brace()?).parse_expr()?;
                let den = LatexStream::new(&self.expect_brace()?).parse_expr()?;
                // a/b = a * b^(-1)
                Ok(MathIR::Mul(vec![
                    num,
                    MathIR::Pow(Box::new(den), Box::new(MathIR::Const(Constant::Int(-1)))),
                ]))
            }

            // ── Integral ─────────────────────────────────────────────────
            "int" | "iint" | "iiint" | "oint" => {
                self.parse_integral()
            }

            // ── Sum ──────────────────────────────────────────────────────
            "sum" => {
                self.parse_sum()
            }

            // ── Product ──────────────────────────────────────────────────
            "prod" => {
                self.parse_product()
            }

            // ── Limit ────────────────────────────────────────────────────
            "lim" | "limsup" | "liminf" => {
                self.parse_limit(name)
            }

            // ── Trig / transcendentals ───────────────────────────────────
            "sin" | "cos" | "tan" | "cot" | "sec" | "csc" |
            "sinh" | "cosh" | "tanh" | "coth" |
            "arcsin" | "arccos" | "arctan" |
            "exp" | "ln" | "log" | "det" | "dim" | "ker" | "hom" => {
                self.parse_function(name)
            }

            // ── Derivative via \frac{d}{dx} pattern ─────────────────────
            // Handled at the \frac level — but if someone writes \dfrac{d}{dx}, same thing.
            // We detect the pattern here: if the next two brace groups are "d" and "dx"/"dt" etc.
            // Actually this is handled by the \frac parser since \frac{d}{dx} goes through parse_integral
            // via the "d" detection. But standalone \derivative or \pd derivative could be added here.

            // ── Square root ──────────────────────────────────────────────
            "sqrt" => {
                let arg = LatexStream::new(&self.expect_brace()?).parse_expr()?;
                Ok(MathIR::Pow(
                    Box::new(arg),
                    Box::new(MathIR::Mul(vec![
                        MathIR::Const(Constant::Int(1)),
                        MathIR::Pow(Box::new(MathIR::Const(Constant::Int(2))), Box::new(MathIR::Const(Constant::Int(-1)))),
                    ])),
                ))
            }

            // ── Known constants ──────────────────────────────────────────
            "pi" => Ok(MathIR::Const(Constant::Symbolic(SymbolicConst::Pi))),
            "infty" | "infinity" => Ok(MathIR::Const(Constant::Symbolic(SymbolicConst::Infinity))),
            "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "theta" |
            "lambda" | "mu" | "sigma" | "phi" | "omega" | "xi" | "rho" | "tau" |
            "kappa" | "zeta" | "eta" | "nu" | "psi" | "chi" => {
                Ok(MathIR::Var(Box::new(Variable {
                    id: name.to_string(),
                    domain: Domain::Real,
                    assumptions: AssumptionSet::default(),
                })))
            }

            // ── Left/right delimiters ────────────────────────────────────
            "left" | "right" => {
                // Skip the delimiter character
                match self.advance() {
                    Token::Char(_) | Token::Cmd(_) => {}
                    _ => {}
                }
                self.parse_primary()
            }

            // ── Spacing / formatting commands (skip) ─────────────────────
            "quad" | "qquad" | "negthinspace" | "thinspace" | "enspace" | "medspace" |
            "mathrm" | "mathbf" | "mathit" | "mathsf" | "mathtt" | "mathcal" |
            "text" | "mbox" | "operatorname" => {
                // If it's \text{...} or \operatorname{...}, parse the content
                let peeked = self.peek().clone();
                if let Token::Brace(content) = peeked {
                    self.advance();
                    LatexStream::new(&content).parse_expr()
                } else {
                    // Just skip spacing commands
                    self.parse_primary()
                }
            }

            _ => {
                // Unknown command — treat as a function call
                if matches!(self.peek(), Token::Brace(_)) {
                    let arg = LatexStream::new(&self.expect_brace()?).parse_expr()?;
                    Ok(MathIR::Fn { name: name.into(), args: vec![arg] })
                } else {
                    Ok(MathIR::Var(Box::new(Variable {
                        id: format!("\\{}", name),
                        domain: Domain::Real,
                        assumptions: AssumptionSet::default(),
                    })))
                }
            }
        }
    }

    // ── Integral parser ─────────────────────────────────────────────────────

    fn parse_integral(&mut self) -> Result<MathIR, ParseError> {
        // Parse optional subscript (lower limit) and superscript (upper limit)
        let mut lower = None;
        let mut upper = None;

        if matches!(self.peek(), Token::Sub) {
            self.advance();
            lower = Some(Box::new(self.parse_primary()?));
        }
        if matches!(self.peek(), Token::Sup) {
            self.advance();
            upper = Some(Box::new(self.parse_primary()?));
        }

        // Parse the integrand
        let integrand = self.parse_term()?;

        // Try to extract the variable of integration from the integrand
        // Look for patterns like "f(t) dt", "f(x) dx", or "f(x) d x"
        let (expr, var) = extract_differential(integrand);

        let limits = match (lower, upper) {
            (Some(lo), Some(hi)) => Some((lo, hi)),
            _ => None,
        };

        Ok(MathIR::Integral {
            expr: Box::new(expr),
            var,
            limits,
        })
    }

    // ── Sum parser ──────────────────────────────────────────────────────────

    fn parse_sum(&mut self) -> Result<MathIR, ParseError> {
        let mut lower = None;
        let mut upper = None;

        if matches!(self.peek(), Token::Sub) {
            self.advance();
            lower = Some(Box::new(self.parse_primary()?));
        }
        if matches!(self.peek(), Token::Sup) {
            self.advance();
            upper = Some(Box::new(self.parse_primary()?));
        }

        let body = self.parse_term()?;

        let var = Variable { id: "i".into(), ..Default::default() };
        let limits = match (lower, upper) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (Box::new(MathIR::Const(Constant::Int(0))), Box::new(MathIR::Const(Constant::Symbolic(SymbolicConst::Infinity)))),
        };

        Ok(MathIR::Sum {
            expr: Box::new(body),
            var,
            limits,
        })
    }

    // ── Product parser ──────────────────────────────────────────────────────

    fn parse_product(&mut self) -> Result<MathIR, ParseError> {
        let mut lower = None;
        let mut upper = None;

        if matches!(self.peek(), Token::Sub) {
            self.advance();
            lower = Some(Box::new(self.parse_primary()?));
        }
        if matches!(self.peek(), Token::Sup) {
            self.advance();
            upper = Some(Box::new(self.parse_primary()?));
        }

        let body = self.parse_term()?;

        let var = Variable { id: "i".into(), ..Default::default() };
        let limits = match (lower, upper) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (Box::new(MathIR::Const(Constant::Int(1))), Box::new(MathIR::Const(Constant::Symbolic(SymbolicConst::Infinity)))),
        };

        Ok(MathIR::Product {
            expr: Box::new(body),
            var,
            limits,
        })
    }

    // ── Limit parser ────────────────────────────────────────────────────────

    fn parse_limit(&mut self, name: &str) -> Result<MathIR, ParseError> {
        // \lim_{x \to a} or \lim_{x \to \infty}
        let mut var = Variable { id: "x".into(), ..Default::default() };
        let mut target = MathIR::Const(Constant::Symbolic(SymbolicConst::Infinity));
        let dir = Dir::Both;

        if matches!(self.peek(), Token::Sub) {
            self.advance();
            // Parse the subscript: should be like "x \to a" or "x \rightarrow a"
            if let Token::Brace(content) = self.peek().clone() {
                self.advance();
                let mut sub_stream = LatexStream::new(&content);
                // Parse variable
                if let Token::Char(c) = sub_stream.peek().clone() {
                    if c.is_alphabetic() {
                        sub_stream.advance();
                        var = Variable { id: c.to_string(), ..Default::default() };
                    }
                }
                // Skip \to or \rightarrow
                if let Token::Cmd(cmd) = sub_stream.peek().clone() {
                    if cmd == "to" || cmd == "rightarrow" || cmd == "Rightarrow" {
                        sub_stream.advance();
                    }
                }
                // Parse target
                target = sub_stream.parse_expr()?;
            }
        }

        let body = self.parse_primary()?;

        let target_box = Box::new(target);
        match name {
            "limsup" => Ok(MathIR::Limit {
                expr: Box::new(MathIR::Fn { name: "sup".into(), args: vec![body] }),
                var,
                target: target_box,
                dir,
            }),
            "liminf" => Ok(MathIR::Limit {
                expr: Box::new(MathIR::Fn { name: "inf".into(), args: vec![body] }),
                var,
                target: target_box,
                dir,
            }),
            _ => Ok(MathIR::Limit {
                expr: Box::new(body),
                var,
                target: target_box,
                dir,
            }),
        }
    }

    // ── Function call parser ────────────────────────────────────────────────

    fn parse_function(&mut self, name: &str) -> Result<MathIR, ParseError> {
        let mut args = Vec::new();

        // Check for optional bracket argument (e.g., \sqrt[3]{x})
        if matches!(self.peek(), Token::Bracket(_)) {
            if let Token::Bracket(content) = self.advance() {
                args.push(LatexStream::new(&content).parse_expr()?);
            }
        }

        // Parse mandatory brace argument or parenthesized argument
        match self.peek().clone() {
            Token::Brace(_) => {
                let content = self.expect_brace()?;
                args.push(LatexStream::new(&content).parse_expr()?);
            }
            Token::Lp => {
                self.advance();
                args.push(self.parse_expr()?);
                self.expect_rp()?;
            }
            _ => {
                // No argument — e.g., \sin x (implicit)
                args.push(self.parse_primary()?);
            }
        }

        Ok(MathIR::Fn { name: name.into(), args })
    }

    fn expect_rp(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Token::Rp => Ok(()),
            t => Err(ParseError::Invalid(format!("Expected ')', got {:?}", t))),
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract differential variable from integrand.
/// Handles:
///   - Fn("d", [Var("x")]) → d(x) as a function call
///   - Mul([..., Var("d"), Var("x")]) → d x as separate factors
///   - Mul([...Fn("d", [Var("x"])])]) → f(x) d(x)
fn extract_differential(expr: MathIR) -> (MathIR, Variable) {
    let default_var = Variable { id: "t".into(), ..Default::default() };

    match &expr {
        // Case: just "d(x)" with no integrand
        MathIR::Fn { name, args } if name.name == "d" && args.len() == 1 => {
            let var = match &args[0] {
                MathIR::Var(v) => v.as_ref().clone(),
                _ => default_var,
            };
            (MathIR::Const(Constant::Int(1)), var)
        }
        MathIR::Mul(factors) => {
            // Scan from the end to find the differential pattern
            // Pattern 1: last factor is Fn("d", [Var("x")])
            if let Some(MathIR::Fn { name, args }) = factors.last() {
                if name.name == "d" && args.len() == 1 {
                    let var = match &args[0] {
                        MathIR::Var(v) => v.as_ref().clone(),
                        _ => default_var,
                    };
                    let remaining: Vec<MathIR> = factors[..factors.len() - 1].to_vec();
                    let inner = if remaining.len() == 1 {
                        remaining.into_iter().next().unwrap()
                    } else {
                        MathIR::Mul(remaining)
                    };
                    return (inner, var);
                }
            }

            // Pattern 2: last two factors are Var("d") and Var("x") where x is single-letter
            if factors.len() >= 2 {
                let second_to_last = &factors[factors.len() - 2];
                let last = &factors[factors.len() - 1];
                if let (MathIR::Var(d_var), MathIR::Var(x_var)) = (second_to_last, last) {
                    if d_var.id == "d" && x_var.id.len() == 1 && x_var.id.chars().next().unwrap().is_alphabetic() {
                        let var = x_var.as_ref().clone();
                        let remaining: Vec<MathIR> = factors[..factors.len() - 2].to_vec();
                        let inner = if remaining.is_empty() {
                            MathIR::Const(Constant::Int(1))
                        } else if remaining.len() == 1 {
                            remaining.into_iter().next().unwrap()
                        } else {
                            MathIR::Mul(remaining)
                        };
                        return (inner, var);
                    }
                }
            }

            (expr, default_var)
        }
        // No differential found
        _ => (expr, default_var),
    }
}

fn find_top_level_eq(input: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in input.chars().enumerate() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '=' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> MathIR {
        LatexParser.parse(input).unwrap()
    }

    #[test]
    fn test_integer() {
        assert_eq!(parse("42"), MathIR::Const(Constant::Int(42)));
    }

    #[test]
    fn test_variable() {
        assert!(matches!(parse("x"), MathIR::Var(_)));
    }

    #[test]
    fn test_equation() {
        let result = parse("x = 1");
        assert!(matches!(result, MathIR::Eq(_, _)));
    }

    #[test]
    fn test_addition() {
        let result = parse("x + y");
        assert!(matches!(result, MathIR::Add(_)));
    }

    #[test]
    fn test_power() {
        let result = parse("x^2");
        assert!(matches!(result, MathIR::Pow(_, _)));
    }

    #[test]
    fn test_frac() {
        let result = parse("\\frac{a}{b}");
        // Should be a * b^(-1)
        assert!(matches!(result, MathIR::Mul(_)));
    }

    #[test]
    fn test_integral() {
        let result = parse("\\int_{0}^{1} x^2 \\, dx");
        match &result {
            MathIR::Integral { expr: _, var, limits } => {
                assert_eq!(var.id, "x");
                assert!(limits.is_some());
            }
            _ => panic!("Expected Integral, got {:?}", result),
        }
    }

    #[test]
    fn test_derivative() {
        // \frac{d}{dx} x^2
        let result = parse("\\frac{d}{dx} x^2");
        // Should be: d * (dx)^(-1) * x^2
        // The \frac{d}{dx} pattern: "d" in first brace, "dx" in second
        assert!(matches!(result, MathIR::Mul(_)));
    }

    #[test]
    fn test_sin() {
        let result = parse("\\sin(x)");
        match &result {
            MathIR::Fn { name, args } => {
                assert_eq!(name.name, "sin");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Fn, got {:?}", result),
        }
    }

    #[test]
    fn test_sin_implicit() {
        let result = parse("\\sin x");
        match &result {
            MathIR::Fn { name, args } => {
                assert_eq!(name.name, "sin");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Fn, got {:?}", result),
        }
    }

    #[test]
    fn test_exp() {
        let result = parse("\\exp(x)");
        match &result {
            MathIR::Fn { name, .. } => assert_eq!(name.name, "exp"),
            _ => panic!("Expected Fn, got {:?}", result),
        }
    }

    #[test]
    fn test_ln() {
        let result = parse("\\ln(x)");
        match &result {
            MathIR::Fn { name, .. } => assert_eq!(name.name, "ln"),
            _ => panic!("Expected Fn, got {:?}", result),
        }
    }

    #[test]
    fn test_pi() {
        assert_eq!(parse("\\pi"), MathIR::Const(Constant::Symbolic(SymbolicConst::Pi)));
    }

    #[test]
    fn test_infty() {
        assert_eq!(parse("\\infty"), MathIR::Const(Constant::Symbolic(SymbolicConst::Infinity)));
    }

    #[test]
    fn test_sum() {
        let result = parse("\\sum_{i=1}^{n} i");
        assert!(matches!(result, MathIR::Sum { .. }));
    }

    #[test]
    fn test_prod() {
        let result = parse("\\prod_{i=1}^{n} i");
        assert!(matches!(result, MathIR::Product { .. }));
    }

    #[test]
    fn test_lim() {
        let result = parse("\\lim_{x \\to 0} \\frac{\\sin(x)}{x}");
        assert!(matches!(result, MathIR::Limit { .. }));
    }

    #[test]
    fn test_nested_frac_integral() {
        // \int_{0}^{1} \frac{x^2}{2} dx
        let result = parse("\\int_{0}^{1} \\frac{x^2}{2} \\, dx");
        assert!(matches!(result, MathIR::Integral { .. }));
    }
}
