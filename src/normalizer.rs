use crate::{MathIR, Constant, Variable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Normalizer {
    rules: Vec<RewriteRule>,
    max_iterations: usize,
}

#[derive(Debug, Clone)]
pub struct RewriteRule {
    pub name: String,
    pub pattern: MathIR,
    pub replacement: MathIR,
    pub priority: u32,
}

fn var_x() -> Variable {
    Variable { id: "__x".into(), ..Default::default() }
}

fn var_f() -> Variable {
    Variable { id: "__f".into(), ..Default::default() }
}

impl Normalizer {
    pub fn new() -> Self {
        let mut rules = Vec::new();
        Self::add_arithmetic_rules(&mut rules);
        Self::add_algebraic_rules(&mut rules);
        Self::add_transcendental_rules(&mut rules);
        Self::add_calculus_rules(&mut rules);
        Self {
            rules,
            max_iterations: 1000,
        }
    }

    fn add_arithmetic_rules(rules: &mut Vec<RewriteRule>) {
        rules.push(RewriteRule {
            name: "add_zero".to_string(),
            pattern: MathIR::Add(vec![MathIR::Var(Box::new(var_x())), MathIR::Const(Constant::Int(0))]),
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "add_zero_left".to_string(),
            pattern: MathIR::Add(vec![MathIR::Const(Constant::Int(0)), MathIR::Var(Box::new(var_x()))]),
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "mul_one".to_string(),
            pattern: MathIR::Mul(vec![MathIR::Var(Box::new(var_x())), MathIR::Const(Constant::Int(1))]),
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "mul_zero".to_string(),
            pattern: MathIR::Mul(vec![MathIR::Var(Box::new(var_x())), MathIR::Const(Constant::Int(0))]),
            replacement: MathIR::Const(Constant::Int(0)),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "pow_zero".to_string(),
            pattern: MathIR::Pow(Box::new(MathIR::Var(Box::new(var_x()))), Box::new(MathIR::Const(Constant::Int(0)))),
            replacement: MathIR::Const(Constant::Int(1)),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "pow_one".to_string(),
            pattern: MathIR::Pow(Box::new(MathIR::Var(Box::new(var_x()))), Box::new(MathIR::Const(Constant::Int(1)))),
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 100,
        });

        rules.push(RewriteRule {
            name: "one_pow".to_string(),
            pattern: MathIR::Pow(Box::new(MathIR::Const(Constant::Int(1))), Box::new(MathIR::Var(Box::new(var_x())))),
            replacement: MathIR::Const(Constant::Int(1)),
            priority: 100,
        });
    }

    fn add_algebraic_rules(rules: &mut Vec<RewriteRule>) {
        rules.push(RewriteRule {
            name: "pythagorean".to_string(),
            pattern: MathIR::Add(vec![
                MathIR::Pow(
                    Box::new(MathIR::Fn { name: "sin".into(), args: vec![MathIR::Var(Box::new(var_x()))] }),
                    Box::new(MathIR::Const(Constant::Int(2))),
                ),
                MathIR::Pow(
                    Box::new(MathIR::Fn { name: "cos".into(), args: vec![MathIR::Var(Box::new(var_x()))] }),
                    Box::new(MathIR::Const(Constant::Int(2))),
                ),
            ]),
            replacement: MathIR::Const(Constant::Int(1)),
            priority: 80,
        });
    }

    fn add_transcendental_rules(rules: &mut Vec<RewriteRule>) {
        rules.push(RewriteRule {
            name: "exp_zero".to_string(),
            pattern: MathIR::Fn { name: "exp".into(), args: vec![MathIR::Const(Constant::Int(0))] },
            replacement: MathIR::Const(Constant::Int(1)),
            priority: 95,
        });

        rules.push(RewriteRule {
            name: "ln_one".to_string(),
            pattern: MathIR::Fn { name: "ln".into(), args: vec![MathIR::Const(Constant::Int(1))] },
            replacement: MathIR::Const(Constant::Int(0)),
            priority: 95,
        });

        rules.push(RewriteRule {
            name: "exp_ln_cancel".to_string(),
            pattern: MathIR::Fn { name: "exp".into(), args: vec![
                MathIR::Fn { name: "ln".into(), args: vec![MathIR::Var(Box::new(var_x()))] }
            ] },
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 95,
        });

        rules.push(RewriteRule {
            name: "ln_exp_cancel".to_string(),
            pattern: MathIR::Fn { name: "ln".into(), args: vec![
                MathIR::Fn { name: "exp".into(), args: vec![MathIR::Var(Box::new(var_x()))] }
            ] },
            replacement: MathIR::Var(Box::new(var_x())),
            priority: 95,
        });
    }

    fn add_calculus_rules(rules: &mut Vec<RewriteRule>) {
        rules.push(RewriteRule {
            name: "deriv_var".to_string(),
            pattern: MathIR::Derivative(Box::new(MathIR::Var(Box::new(var_x()))), var_x()),
            replacement: MathIR::Const(Constant::Int(1)),
            priority: 90,
        });

        rules.push(RewriteRule {
            name: "int_deriv_cancel".to_string(),
            pattern: MathIR::Integral {
                expr: Box::new(MathIR::Derivative(Box::new(MathIR::Var(Box::new(var_f()))), var_x())),
                var: var_x(),
                limits: None,
            },
            replacement: MathIR::Var(Box::new(var_f())),
            priority: 90,
        });

        // Fundamental Theorem of Calculus Part 1:
        // d/dx [ integral_a^x f(t) dt ] = f(x)
        // Pattern: Derivative(Integral(f(t), t, a, x), x) => f(x)
        rules.push(RewriteRule {
            name: "ftc_1".to_string(),
            pattern: MathIR::Derivative(
                Box::new(MathIR::Integral {
                    expr: Box::new(MathIR::Var(Box::new(var_f()))),
                    var: Variable { id: "t".into(), ..Default::default() },
                    limits: Some((
                        Box::new(MathIR::Var(Box::new(Variable { id: "__a".into(), ..Default::default() }))),
                        Box::new(MathIR::Var(Box::new(var_x()))),
                    )),
                }),
                var_x(),
            ),
            replacement: MathIR::Var(Box::new(var_f())),
            priority: 95,
        });

        // Integral of derivative (with limits):
        // integral_a^b f'(x) dx = f(b) - f(a)
        // Simplified: just cancel derivative inside integral when no limits
        rules.push(RewriteRule {
            name: "int_of_deriv".to_string(),
            pattern: MathIR::Integral {
                expr: Box::new(MathIR::Derivative(
                    Box::new(MathIR::Var(Box::new(var_f()))),
                    var_x(),
                )),
                var: var_x(),
                limits: None,
            },
            replacement: MathIR::Var(Box::new(var_f())),
            priority: 90,
        });
    }

    pub fn normalize(&self, expr: &MathIR) -> MathIR {
        let mut current = expr.clone();
        for _ in 0..self.max_iterations {
            let mut changed = false;
            for rule in &self.rules {
                if self.matches(&rule.pattern, &current) {
                    current = self.apply_rule(&rule.pattern, &rule.replacement, &current);
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }
        current
    }

    fn matches(&self, pattern: &MathIR, expr: &MathIR) -> bool {
        match (pattern, expr) {
            (MathIR::Var(p), MathIR::Var(_)) if p.id.starts_with("__") => true,
            (MathIR::Add(pa), MathIR::Add(ea)) => {
                pa.len() == ea.len() && pa.iter().zip(ea.iter()).all(|(p, e)| self.matches(p, e))
            }
            (MathIR::Mul(pa), MathIR::Mul(ea)) => {
                pa.len() == ea.len() && pa.iter().zip(ea.iter()).all(|(p, e)| self.matches(p, e))
            }
            (MathIR::Pow(pa, pb), MathIR::Pow(ea, eb)) => {
                self.matches(pa, ea) && self.matches(pb, eb)
            }
            (MathIR::Fn { name: pn, args: pa }, MathIR::Fn { name: en, args: ea }) => {
                pn == en && pa.len() == ea.len() && pa.iter().zip(ea.iter()).all(|(p, e)| self.matches(p, e))
            }
            (MathIR::Eq(pa, pb), MathIR::Eq(ea, eb)) => {
                self.matches(pa, ea) && self.matches(pb, eb)
            }
            (MathIR::Const(pc), MathIR::Const(ec)) => pc == ec,
            (MathIR::Derivative(pa, pv), MathIR::Derivative(ea, ev)) => {
                self.matches(pa, ea) && self.matches_var(pv, ev)
            }
            (MathIR::Integral { expr: pe, var: pv, limits: pl },
             MathIR::Integral { expr: ee, var: ev, limits: el }) => {
                self.matches_var(pv, ev) && self.matches(pe, ee) && match (pl, el) {
                    (None, None) => true,
                    (Some((plo, phi)), Some((elo, ehi))) => {
                        self.matches(plo, elo) && self.matches(phi, ehi)
                    }
                    _ => false,
                }
            }
            (MathIR::Limit { expr: pe, var: pv, target: pt, dir: pd },
             MathIR::Limit { expr: ee, var: ev, target: et, dir: ed }) => {
                pv == ev && pd == ed && self.matches(pe, ee) && self.matches(pt, et)
            }
            _ => false,
        }
    }

    /// Match variables, handling wildcards (prefix "__")
    fn matches_var(&self, pattern: &Variable, expr: &Variable) -> bool {
        pattern.id.starts_with("__") || pattern == expr
    }

    fn apply_rule(&self, pattern: &MathIR, replacement: &MathIR, expr: &MathIR) -> MathIR {
        let mut bindings: HashMap<String, MathIR> = HashMap::new();
        self.collect_bindings(pattern, expr, &mut bindings);
        self.substitute(replacement, &bindings)
    }

    fn collect_bindings(&self, pattern: &MathIR, expr: &MathIR, bindings: &mut HashMap<String, MathIR>) {
        if let MathIR::Var(p) = pattern {
            if p.id.starts_with("__") {
                bindings.insert(p.id.clone(), expr.clone());
                return;
            }
        }
        // Handle Derivative: collect from expression and variable
        if let (MathIR::Derivative(p_expr, p_var), MathIR::Derivative(e_expr, e_var)) = (pattern, expr) {
            self.collect_bindings(p_expr, e_expr, bindings);
            if p_var.id.starts_with("__") {
                bindings.insert(p_var.id.clone(), MathIR::Var(Box::new(e_var.clone())));
            }
            return;
        }
        // Handle Integral: collect from expression, variable, and limits
        if let (MathIR::Integral { expr: p_expr, var: p_var, limits: p_lim },
                MathIR::Integral { expr: e_expr, var: e_var, limits: e_lim }) = (pattern, expr) {
            self.collect_bindings(p_expr, e_expr, bindings);
            if p_var.id.starts_with("__") {
                bindings.insert(p_var.id.clone(), MathIR::Var(Box::new(e_var.clone())));
            }
            if let (Some((p_lo, p_hi)), Some((e_lo, e_hi))) = (p_lim, e_lim) {
                self.collect_bindings(p_lo, e_lo, bindings);
                self.collect_bindings(p_hi, e_hi, bindings);
            }
            return;
        }
        let p_children = pattern.children();
        let e_children = expr.children();
        for (p, e) in p_children.iter().zip(e_children.iter()) {
            self.collect_bindings(p, e, bindings);
        }
    }

    fn substitute(&self, expr: &MathIR, bindings: &HashMap<String, MathIR>) -> MathIR {
        match expr {
            MathIR::Var(v) if v.id.starts_with("__") => {
                bindings.get(&v.id).cloned().unwrap_or_else(|| expr.clone())
            }
            MathIR::Add(args) => MathIR::Add(args.iter().map(|a| self.substitute(a, bindings)).collect()),
            MathIR::Mul(args) => MathIR::Mul(args.iter().map(|a| self.substitute(a, bindings)).collect()),
            MathIR::Pow(a, b) => MathIR::Pow(
                Box::new(self.substitute(a, bindings)),
                Box::new(self.substitute(b, bindings)),
            ),
            MathIR::Fn { name, args } => MathIR::Fn {
                name: name.clone(),
                args: args.iter().map(|a| self.substitute(a, bindings)).collect(),
            },
            MathIR::Eq(a, b) => MathIR::Eq(
                Box::new(self.substitute(a, bindings)),
                Box::new(self.substitute(b, bindings)),
            ),
            MathIR::Derivative(a, v) => MathIR::Derivative(
                Box::new(self.substitute(a, bindings)),
                v.clone(),
            ),
            MathIR::Integral { expr, var, limits } => MathIR::Integral {
                expr: Box::new(self.substitute(expr, bindings)),
                var: var.clone(),
                limits: limits.as_ref().map(|(lo, hi)| (
                    Box::new(self.substitute(lo, bindings)),
                    Box::new(self.substitute(hi, bindings)),
                )),
            },
            MathIR::Limit { expr, var, target, dir } => MathIR::Limit {
                expr: Box::new(self.substitute(expr, bindings)),
                var: var.clone(),
                target: Box::new(self.substitute(target, bindings)),
                dir: dir.clone(),
            },
            MathIR::Sum { expr, var, limits } => MathIR::Sum {
                expr: Box::new(self.substitute(expr, bindings)),
                var: var.clone(),
                limits: (
                    Box::new(self.substitute(&limits.0, bindings)),
                    Box::new(self.substitute(&limits.1, bindings)),
                ),
            },
            MathIR::Product { expr, var, limits } => MathIR::Product {
                expr: Box::new(self.substitute(expr, bindings)),
                var: var.clone(),
                limits: (
                    Box::new(self.substitute(&limits.0, bindings)),
                    Box::new(self.substitute(&limits.1, bindings)),
                ),
            },
            other => other.clone(),
        }
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftc_1() {
        // d/dx [ integral_a^x f(t) dt ] = f(x)
        // Build: Derivative(Integral(Var("f"), Var("t"), Some(Var("__a"), Var("x"))), Var("x"))
        let input = MathIR::Derivative(
            Box::new(MathIR::Integral {
                expr: Box::new(MathIR::Var(Box::new(Variable { id: "f".into(), ..Default::default() }))),
                var: Variable { id: "t".into(), ..Default::default() },
                limits: Some((
                    Box::new(MathIR::Var(Box::new(Variable { id: "a".into(), ..Default::default() }))),
                    Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), ..Default::default() }))),
                )),
            }),
            Variable { id: "x".into(), ..Default::default() },
        );

        let norm = Normalizer::new();
        let result = norm.normalize(&input);

        // Should reduce to f(x)
        match result {
            MathIR::Var(v) => assert_eq!(v.id, "f"),
            _ => panic!("Expected Var(\"f\"), got {:?}", result),
        }
    }

    #[test]
    fn test_pythagorean() {
        // sin^2(x) + cos^2(x) = 1
        let input = MathIR::Add(vec![
            MathIR::Pow(
                Box::new(MathIR::Fn { name: "sin".into(), args: vec![MathIR::Var(Box::new(Variable { id: "x".into(), ..Default::default() }))] }),
                Box::new(MathIR::Const(Constant::Int(2))),
            ),
            MathIR::Pow(
                Box::new(MathIR::Fn { name: "cos".into(), args: vec![MathIR::Var(Box::new(Variable { id: "x".into(), ..Default::default() }))] }),
                Box::new(MathIR::Const(Constant::Int(2))),
            ),
        ]);

        let norm = Normalizer::new();
        let result = norm.normalize(&input);

        assert_eq!(result, MathIR::Const(Constant::Int(1)));
    }
}
