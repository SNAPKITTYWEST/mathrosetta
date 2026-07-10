use crate::MathIR;
use super::{Emitter, EmittedTarget, ProofBackend, ProofStatus};

pub struct LatexReportEmitter;

impl Emitter for LatexReportEmitter {
    fn backend(&self) -> ProofBackend {
        ProofBackend::Latex
    }

    fn emit(&self, expr: &MathIR, theory_name: &str) -> EmittedTarget {
        let source = emit_latex_report(expr, theory_name, None, None);
        EmittedTarget {
            backend: ProofBackend::Latex,
            source,
            status: ProofStatus::LatexReportEmitted,
        }
    }
}

pub struct LatexReportMeta<'a> {
    pub input_latex: &'a str,
    pub mathir_json: &'a str,
    pub normalized_json: &'a str,
    pub isabelle_source: Option<&'a str>,
    pub lean4_source: Option<&'a str>,
    pub coq_source: Option<&'a str>,
    pub smtlib_source: Option<&'a str>,
    pub worm_hash: Option<&'a str>,
    pub status_label: &'a str,
}

pub fn emit_latex_report(
    expr: &MathIR,
    theory_name: &str,
    input_latex: Option<&str>,
    meta: Option<&LatexReportMeta<'_>>,
) -> String {
    let mut out = String::new();

    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage{amsmath, amssymb, amsthm}\n");
    out.push_str("\\usepackage{geometry}\n");
    out.push_str("\\usepackage{hyperref}\n");
    out.push_str("\\usepackage{listings}\n");
    out.push_str("\\usepackage{xcolor}\n");
    out.push_str("\\geometry{margin=1in}\n");
    out.push_str("\n");
    out.push_str("\\newtheorem{theorem}{Theorem}\n");
    out.push_str("\\newtheorem{definition}{Definition}\n");
    out.push_str("\n");
    out.push_str("\\lstset{\n");
    out.push_str("  basicstyle=\\ttfamily\\small,\n");
    out.push_str("  backgroundcolor=\\color{gray!10},\n");
    out.push_str("  frame=single,\n");
    out.push_str("  breaklines=true,\n");
    out.push_str("}\n");
    out.push_str("\n");
    out.push_str(&format!("\\title{{{}}}\n", escape_latex(theory_name)));
    out.push_str("\\author{SnapKitty MathRosetta}\n");
    out.push_str("\\date{\\today}\n");
    out.push_str("\n");
    out.push_str("\\begin{document}\n");
    out.push_str("\\maketitle\n");
    out.push_str("\\tableofcontents\n");
    out.push_str("\\newpage\n\n");

    // ── Section 1: Theorem Name ──
    out.push_str("\\section{Theorem Name}\n\n");
    out.push_str(&format!("\\textbf{{{} }}\n\n", escape_latex(theory_name)));

    // ── Section 2: Original Input ──
    out.push_str("\\section{Original Input}\n\n");
    match input_latex {
        Some(latex) => {
            out.push_str("\\begin{equation}\n");
            out.push_str(latex);
            out.push_str("\n\\end{equation}\n\n");
        }
        None => {
            out.push_str("\\begin{equation}\n");
            out.push_str(&mathir_to_latex_display(expr));
            out.push_str("\n\\end{equation}\n\n");
        }
    }

    // ── Section 3: Normalized Form ──
    out.push_str("\\section{Normalized Mathematical Form}\n\n");
    out.push_str("\\begin{equation}\n");
    out.push_str(&mathir_to_latex_display(expr));
    out.push_str("\n\\end{equation}\n\n");

    // ── Section 4: MathIR JSON ──
    out.push_str("\\section{MathIR AST}\n\n");
    let mathir_json = meta.map(|m| m.mathir_json).unwrap_or("{}");
    out.push_str("\\begin{lstlisting}[language=JSON]\n");
    out.push_str(mathir_json);
    out.push_str("\n\\end{lstlisting}\n\n");

    // ── Sections 5-8: Proof Targets ──
    if let Some(m) = meta {
        emit_proof_target_section(&mut out, "Isabelle/HOL", m.isabelle_source);
        emit_proof_target_section(&mut out, "Lean 4", m.lean4_source);
        emit_proof_target_section(&mut out, "Coq", m.coq_source);
        emit_proof_target_section(&mut out, "SMT-LIB", m.smtlib_source);
    } else {
        out.push_str("\\section{Isabelle/HOL Target}\n\n");
        out.push_str("\\textit{Not generated.}\n\n");
        out.push_str("\\section{Lean 4 Target}\n\n");
        out.push_str("\\textit{Not generated.}\n\n");
        out.push_str("\\section{Coq Target}\n\n");
        out.push_str("\\textit{Not generated.}\n\n");
        out.push_str("\\section{SMT-LIB Target}\n\n");
        out.push_str("\\textit{Not generated.}\n\n");
    }

    // ── Section 9: WORM Receipt Hash ──
    out.push_str("\\section{WORM Receipt Hash}\n\n");
    match meta.and_then(|m| m.worm_hash) {
        Some(hash) => {
            out.push_str("\\begin{verbatim}\n");
            out.push_str(hash);
            out.push_str("\n\\end{verbatim}\n\n");
        }
        None => {
            out.push_str("\\textit{No WORM receipt available.}\n\n");
        }
    }

    // ── Section 10: Proof Status ──
    out.push_str("\\section{Proof Status}\n\n");
    let status = meta.map(|m| m.status_label).unwrap_or("emitted_pending_proof");
    out.push_str(&format!(
        "\\begin{{description}}\n\\item[Status] \\texttt{{{}}}\n\\end{{description}}\n\n",
        escape_latex(status)
    ));
    out.push_str("\\begin{description}\n");
    out.push_str("\\item[Note] This is a human-readable report, not a proof certificate.\\newline\n");
    out.push_str("LaTeX output status: \\texttt{latex\\_report\\_emitted}.\n");
    out.push_str("The proof targets above must be submitted to their respective checkers for verification.\n");
    out.push_str("\\end{description}\n\n");

    out.push_str("\\end{document}\n");
    out
}

fn emit_proof_target_section(out: &mut String, name: &str, source: Option<&str>) {
    out.push_str(&format!("\\section{{{}}}\n\n", name));
    match source {
        Some(src) => {
            out.push_str("\\begin{lstlisting}\n");
            out.push_str(src);
            out.push_str("\n\\end{lstlisting}\n\n");
        }
        None => {
            out.push_str("\\textit{Not generated.}\n\n");
        }
    }
}

fn mathir_to_latex_display(expr: &MathIR) -> String {
    match expr {
        MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.4}", f),
            crate::Constant::Rational { numer, denom } => {
                format!("\\frac{{{}}}{{{}}}", numer, denom)
            }
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "\\pi".to_string(),
                crate::SymbolicConst::E => "e".to_string(),
                crate::SymbolicConst::Infinity => "\\infty".to_string(),
                crate::SymbolicConst::NegInfinity => "-\\infty".to_string(),
                crate::SymbolicConst::EulerGamma => "\\gamma".to_string(),
                crate::SymbolicConst::Catalan => "G".to_string(),
                crate::SymbolicConst::ComplexInfinity => "\\infty".to_string(),
                crate::SymbolicConst::I => "i".to_string(),
            },
            _ => "\\textit{?}".to_string(),
        },
        MathIR::Var(v) => escape_latex(&v.id),
        MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex_display).collect();
            parts.join(" + ")
        }
        MathIR::Mul(args) => {
            if args.len() == 2 {
                let a = mathir_to_latex_display(&args[0]);
                let b = mathir_to_latex_display(&args[1]);
                format!("{} \\cdot {}", a, b)
            } else {
                let parts: Vec<String> = args.iter().map(mathir_to_latex_display).collect();
                parts.join(" \\cdot ")
            }
        }
        MathIR::Pow(base, exp) => {
            let base_str = mathir_to_latex_display(base);
            let exp_str = mathir_to_latex_display(exp);
            if base.is_atomic() {
                format!("{}^{{{}}}", base_str, exp_str)
            } else {
                format!("({})^{{{}}}", base_str, exp_str)
            }
        }
        MathIR::Fn { name, args } => {
            let fn_name = match name.as_str() {
                "sin" => "\\sin",
                "cos" => "\\cos",
                "tan" => "\\tan",
                "log" => "\\log",
                "ln" => "\\ln",
                "exp" => "\\exp",
                "sqrt" => "\\sqrt",
                other => other,
            };
            if args.is_empty() {
                fn_name.to_string()
            } else {
                let args_str: Vec<String> = args.iter().map(mathir_to_latex_display).collect();
                if name.as_str() == "sqrt" {
                    format!("\\sqrt{{{}}}", args_str.join(", "))
                } else {
                    format!("{}{{{}}}", fn_name, args_str.join(", "))
                }
            }
        }
        MathIR::Eq(lhs, rhs) => {
            format!("{} = {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Neq(lhs, rhs) => {
            format!("{} \\neq {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Lt(lhs, rhs) => {
            format!("{} < {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Lte(lhs, rhs) => {
            format!("{} \\leq {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Gt(lhs, rhs) => {
            format!("{} > {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Gte(lhs, rhs) => {
            format!("{} \\geq {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex_display).collect();
            parts.join(" \\land ")
        }
        MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex_display).collect();
            parts.join(" \\lor ")
        }
        MathIR::Not(inner) => {
            format!("\\lnot {}", mathir_to_latex_display(inner))
        }
        MathIR::Implies(lhs, rhs) => {
            format!("{} \\to {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::Iff(lhs, rhs) => {
            format!("{} \\leftrightarrow {}", mathir_to_latex_display(lhs), mathir_to_latex_display(rhs))
        }
        MathIR::ForAll(var, _, body) => {
            format!(
                "\\forall {} \\in {}\\colon {}",
                escape_latex(&var.id),
                domain_to_latex(&var.domain),
                mathir_to_latex_display(body)
            )
        }
        MathIR::Exists(var, _, body) => {
            format!(
                "\\exists {} \\in {}\\colon {}",
                escape_latex(&var.id),
                domain_to_latex(&var.domain),
                mathir_to_latex_display(body)
            )
        }
        MathIR::Derivative(expr, var) => {
            format!(
                "\\frac{{d}}{{d{}}} {}",
                escape_latex(&var.id),
                mathir_to_latex_display(expr)
            )
        }
        MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!(
                        "\\int_{{ {} }}^{{ {} }} {} \\, d{}",
                        mathir_to_latex_display(lo),
                        mathir_to_latex_display(hi),
                        mathir_to_latex_display(expr),
                        escape_latex(&var.id)
                    )
                }
                None => {
                    format!(
                        "\\int {} \\, d{}",
                        mathir_to_latex_display(expr),
                        escape_latex(&var.id)
                    )
                }
            }
        }
        MathIR::Limit { expr, var, target, dir } => {
            let dir_str = match dir {
                crate::Dir::Positive => "^{+}",
                crate::Dir::Negative => "^{-}",
                crate::Dir::Both => "",
            };
            format!(
                "\\lim_{{{} \\to {}{}}} {}",
                escape_latex(&var.id),
                mathir_to_latex_display(target),
                dir_str,
                mathir_to_latex_display(expr)
            )
        }
        MathIR::Sum { expr, var, limits } => {
            format!(
                "\\sum_{{{} = {}}}^{{ {} }} {}",
                escape_latex(&var.id),
                mathir_to_latex_display(&limits.0),
                mathir_to_latex_display(&limits.1),
                mathir_to_latex_display(expr)
            )
        }
        MathIR::Product { expr, var, limits } => {
            format!(
                "\\prod_{{{} = {}}}^{{ {} }} {}",
                escape_latex(&var.id),
                mathir_to_latex_display(&limits.0),
                mathir_to_latex_display(&limits.1),
                mathir_to_latex_display(expr)
            )
        }
        MathIR::Matrix(rows) => {
            let row_strs: Vec<String> = rows.iter().map(|row| {
                let cells: Vec<String> = row.iter().map(mathir_to_latex_display).collect();
                cells.join(" & ")
            }).collect();
            format!("\\begin{{pmatrix}} {} \\end{{pmatrix}}", row_strs.join(" \\\\ "))
        }
        MathIR::Vector(elems) => {
            let elem_strs: Vec<String> = elems.iter().map(mathir_to_latex_display).collect();
            format!("\\begin{{pmatrix}} {} \\end{{pmatrix}}", elem_strs.join(" \\\\ "))
        }
        MathIR::Set(elems) => {
            let elem_strs: Vec<String> = elems.iter().map(mathir_to_latex_display).collect();
            format!("\\{{ {} \\}}", elem_strs.join(", "))
        }
        MathIR::In(elem, set) => {
            format!("{} \\in {}", mathir_to_latex_display(elem), mathir_to_latex_display(set))
        }
        MathIR::SetUnion(a, b) => {
            format!("{} \\cup {}", mathir_to_latex_display(a), mathir_to_latex_display(b))
        }
        MathIR::SetIntersect(a, b) => {
            format!("{} \\cap {}", mathir_to_latex_display(a), mathir_to_latex_display(b))
        }
        MathIR::SetDiff(a, b) => {
            format!("{} \\setminus {}", mathir_to_latex_display(a), mathir_to_latex_display(b))
        }
        MathIR::Proof { claim, .. } => {
            format!("\\text{{proof}}({})", mathir_to_latex_display(claim))
        }
        MathIR::Trusted { source, .. } => {
            format!("\\text{{trusted}}({})", escape_latex(source))
        }
        MathIR::Annotated { inner, .. } => {
            mathir_to_latex_display(inner)
        }
        MathIR::Tensor { .. } => "\\textit{{tensor}}".to_string(),
        MathIR::Geometric { .. } => "\\textit{{geometric}}".to_string(),
    }
}

fn domain_to_latex(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "\\mathbb{R}".to_string(),
        crate::Domain::Complex => "\\mathbb{C}".to_string(),
        crate::Domain::Integer => "\\mathbb{Z}".to_string(),
        crate::Domain::Natural => "\\mathbb{N}".to_string(),
        crate::Domain::Rational => "\\mathbb{Q}".to_string(),
        crate::Domain::Positive => "\\mathbb{R}^{+}".to_string(),
        crate::Domain::Negative => "\\mathbb{R}^{-}".to_string(),
        crate::Domain::NonZero => "\\mathbb{R} \\setminus \\{{0\\}}".to_string(),
        crate::Domain::Modulo(n) => format!("\\mathbb{{Z}}/{}\\mathbb{{Z}}", n),
        crate::Domain::UserDefined(name) => escape_latex(name),
        crate::Domain::Any => "\\top".to_string(),
        crate::Domain::Manifold(name) => format!("\\mathcal{{M}}_{{{}}}", escape_latex(name)),
        crate::Domain::FunctionSpace { .. } => "\\to".to_string(),
        crate::Domain::VectorSpace { .. } => "\\mathbb{V}".to_string(),
        crate::Domain::MatrixSpace { .. } => "\\mathbb{M}".to_string(),
    }
}

fn escape_latex(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '$' => out.push_str("\\$"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '\\' => out.push_str("\\textbackslash{}"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    fn make_var(name: &str, domain: Domain) -> Variable {
        Variable { id: name.into(), domain, assumptions: AssumptionSet::default() }
    }

    #[test]
    fn test_latex_report_contains_all_sections() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(make_var("x", Domain::Real)))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let source = emit_latex_report(&expr, "test_theorem", Some("x = 1"), None);
        assert!(source.contains("\\section{Theorem Name}"));
        assert!(source.contains("\\section{Original Input}"));
        assert!(source.contains("\\section{Normalized Mathematical Form}"));
        assert!(source.contains("\\section{MathIR AST}"));
        assert!(source.contains("\\section{Isabelle/HOL Target}"));
        assert!(source.contains("\\section{Lean 4 Target}"));
        assert!(source.contains("\\section{Coq Target}"));
        assert!(source.contains("\\section{SMT-LIB Target}"));
        assert!(source.contains("\\section{WORM Receipt Hash}"));
        assert!(source.contains("\\section{Proof Status}"));
        assert!(source.contains("latex\\_report\\_emitted"));
        assert!(source.contains("\\begin{document}"));
        assert!(source.contains("\\end{document}"));
    }

    #[test]
    fn test_latex_report_with_meta() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(make_var("x", Domain::Real)))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let meta = LatexReportMeta {
            input_latex: "x = 1",
            mathir_json: r#"{"tag":"Eq"}"#,
            normalized_json: r#"{"tag":"Eq"}"#,
            isabelle_source: Some("theorem test: \"x = 1\" sorry"),
            lean4_source: Some("theorem test : x = 1 := by sorry"),
            coq_source: Some("Theorem test : x = 1. Admitted."),
            smtlib_source: Some("(assert (= x 1))"),
            worm_hash: Some("abc123"),
            status_label: "latex_report_emitted",
        };
        let source = emit_latex_report(&expr, "test_theorem", None, Some(&meta));
        assert!(source.contains("x = 1"));
        assert!(source.contains(r#"{"tag":"Eq"}"#));
        assert!(source.contains("theorem test: \"x = 1\" sorry"));
        assert!(source.contains("theorem test : x = 1 := by sorry"));
        assert!(source.contains("Theorem test : x = 1. Admitted."));
        assert!(source.contains("(assert (= x 1))"));
        assert!(source.contains("abc123"));
        assert!(source.contains("latex\\_report\\_emitted"));
    }

    #[test]
    fn test_latex_forall_rendering() {
        let expr = MathIR::ForAll(
            make_var("x", Domain::Real),
            Box::new(Domain::Real),
            Box::new(MathIR::Gte(
                Box::new(MathIR::Pow(
                    Box::new(MathIR::Var(Box::new(make_var("x", Domain::Real)))),
                    Box::new(MathIR::Const(crate::Constant::Int(2))),
                )),
                Box::new(MathIR::Const(crate::Constant::Int(0))),
            )),
        );
        let source = emit_latex_report(&expr, "forall_test", None, None);
        assert!(source.contains("\\forall"));
        assert!(source.contains("\\mathbb{R}"));
        assert!(source.contains("\\geq"));
        assert!(source.contains("^"));
    }

    #[test]
    fn test_latex_escape_special_chars() {
        let expr = MathIR::Var(Box::new(make_var("a_b", Domain::Real)));
        let source = emit_latex_report(&expr, "test & % $ # _ { }", None, None);
        assert!(source.contains("\\&"));
        assert!(source.contains("\\%"));
        assert!(source.contains("\\$"));
        assert!(source.contains("\\#"));
        assert!(source.contains("\\_"));
    }

    #[test]
    fn test_latex_integral_rendering() {
        let expr = MathIR::Integral {
            expr: Box::new(MathIR::Var(Box::new(make_var("f", Domain::Real)))),
            var: make_var("x", Domain::Real),
            limits: Some((
                Box::new(MathIR::Const(crate::Constant::Int(0))),
                Box::new(MathIR::Var(Box::new(make_var("x", Domain::Real)))),
            )),
        };
        let source = emit_latex_report(&expr, "integral_test", None, None);
        assert!(source.contains("\\int"));
        assert!(source.contains("\\, d"));
    }

    #[test]
    fn test_latex_derivative_rendering() {
        let expr = MathIR::Derivative(
            Box::new(MathIR::Var(Box::new(make_var("f", Domain::Real)))),
            make_var("x", Domain::Real),
        );
        let source = emit_latex_report(&expr, "deriv_test", None, None);
        assert!(source.contains("\\frac{d}{dx}"));
    }

    #[test]
    fn test_latex_report_status_not_verified() {
        let expr = MathIR::Const(crate::Constant::Int(42));
        let source = emit_latex_report(&expr, "status_test", None, None);
        assert!(source.contains("latex\\_report\\_emitted"));
        assert!(!source.contains("\\texttt{verified}"));
        assert!(source.contains("not a proof certificate"));
    }

    #[test]
    fn test_latex_sum_rendering() {
        let expr = MathIR::Sum {
            expr: Box::new(MathIR::Var(Box::new(make_var("i", Domain::Integer)))),
            var: make_var("i", Domain::Integer),
            limits: (
                Box::new(MathIR::Const(crate::Constant::Int(1))),
                Box::new(MathIR::Var(Box::new(make_var("n", Domain::Integer)))),
            ),
        };
        let source = emit_latex_report(&expr, "sum_test", None, None);
        assert!(source.contains("\\sum"));
    }
}
