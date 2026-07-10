use crate::MathIR;
use super::{Emitter, EmittedTarget, ProofBackend, ProofStatus};

pub struct SmtLibEmitter;

impl Emitter for SmtLibEmitter {
    fn backend(&self) -> ProofBackend {
        ProofBackend::SmtLib
    }

    fn emit(&self, expr: &MathIR, _theory_name: &str) -> EmittedTarget {
        let source = emit_smtlib(expr);
        EmittedTarget {
            backend: ProofBackend::SmtLib,
            source,
            status: ProofStatus::EmittedSmtlib,
        }
    }
}

fn emit_smtlib(expr: &MathIR) -> String {
    let mut out = String::new();
    out.push_str("(set-logic ALL)\n");

    let sorts = collect_sorts(expr);
    for s in &sorts {
        out.push_str(&format!("(declare-sort {} 0)\n", s));
    }
    if !sorts.is_empty() {
        out.push('\n');
    }

    let funs = collect_funs(expr);
    for (name, sig) in &funs {
        out.push_str(&format!("(declare-fun {} ({}) {})\n", name, sig.args, sig.ret));
    }
    if !funs.is_empty() {
        out.push('\n');
    }

    out.push_str("(assert\n");
    out.push_str(&format!("  {}\n", mathir_to_smtlib(expr, 2)));
    out.push_str(")\n\n");
    out.push_str("(check-sat)\n");
    out
}

fn mathir_to_smtlib(expr: &MathIR, indent: usize) -> String {
    let _pad = "  ".repeat(indent);
    match expr {
        MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.6e}", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "pi".to_string(),
                crate::SymbolicConst::E => "e".to_string(),
                crate::SymbolicConst::Infinity => "infinity".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        MathIR::Var(v) => v.id.clone(),
        MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(+ {})", parts.join(" "))
        }
        MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(* {})", parts.join(" "))
        }
        MathIR::Pow(base, exp) => {
            format!("(^ {} {})", mathir_to_smtlib(base, indent), mathir_to_smtlib(exp, indent))
        }
        MathIR::Fn { name, args } => {
            let args_str: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("({} {})", name.as_str(), args_str.join(" "))
        }
        MathIR::Eq(lhs, rhs) => {
            format!("(= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Neq(lhs, rhs) => {
            format!("(not (= {} {}))", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Lt(lhs, rhs) => {
            format!("(< {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Lte(lhs, rhs) => {
            format!("(<= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Gt(lhs, rhs) => {
            format!("(> {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Gte(lhs, rhs) => {
            format!("(>= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(and {})", parts.join(" "))
        }
        MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(or {})", parts.join(" "))
        }
        MathIR::Not(inner) => {
            format!("(not {})", mathir_to_smtlib(inner, indent))
        }
        MathIR::Implies(lhs, rhs) => {
            format!("(=> {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::Iff(lhs, rhs) => {
            format!("(= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent))
        }
        MathIR::ForAll(var, _domain, body) => {
            format!("(forall (({} {}))\n{}{})",
                var.id, smt_sort(&var.domain),
                "  ".repeat(indent + 1), mathir_to_smtlib(body, indent + 1))
        }
        MathIR::Exists(var, _domain, body) => {
            format!("(exists (({} {}))\n{}{})",
                var.id, smt_sort(&var.domain),
                "  ".repeat(indent + 1), mathir_to_smtlib(body, indent + 1))
        }
        MathIR::Derivative(expr, var) => {
            format!("(deriv (lambda (({} Real)) {}))", var.id, mathir_to_smtlib(expr, indent))
        }
        MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!("(integral (lambda (({} Real)) {}) {} {})",
                        var.id, mathir_to_smtlib(expr, indent),
                        mathir_to_smtlib(lo, indent), mathir_to_smtlib(hi, indent))
                }
                None => {
                    format!("(integral (lambda (({} Real)) {}) 0 0)",
                        var.id, mathir_to_smtlib(expr, indent))
                }
            }
        }
        _ => "??".to_string(),
    }
}

fn smt_sort(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Integer => "Int".to_string(),
        crate::Domain::Rational => "Real".to_string(),
        crate::Domain::Real => "Real".to_string(),
        crate::Domain::Complex => "Complex".to_string(),
        crate::Domain::UserDefined(name) => name.clone(),
        _ => "Any".to_string(),
    }
}

fn collect_sorts(expr: &MathIR) -> Vec<String> {
    let mut sorts = Vec::new();
    collect_sorts_rec(expr, &mut sorts);
    sorts.sort();
    sorts.dedup();
    sorts
}

fn collect_sorts_rec(expr: &MathIR, sorts: &mut Vec<String>) {
    match expr {
        MathIR::ForAll(var, _, body) | MathIR::Exists(var, _, body) => {
            if let crate::Domain::UserDefined(name) = &var.domain {
                if !sorts.contains(name) {
                    sorts.push(name.clone());
                }
            }
            collect_sorts_rec(body, sorts);
        }
        _ => {}
    }
}

#[derive(PartialEq)]
struct FunInfo {
    args: String,
    ret: String,
}

fn collect_funs(expr: &MathIR) -> Vec<(String, FunInfo)> {
    let mut funs = Vec::new();
    match expr {
        MathIR::Fn { name, args } => {
            if !args.is_empty() {
                let arg_sorts: Vec<String> = args.iter().map(|_| "Any".to_string()).collect();
                funs.push((name.as_str().to_string(), FunInfo {
                    args: arg_sorts.join(" "),
                    ret: "Bool".to_string(),
                }));
            }
        }
        _ => {}
    }
    funs.sort_by(|a, b| a.0.cmp(&b.0));
    funs.dedup();
    funs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_smtlib_equality() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let result = emit_smtlib(&expr);
        assert!(result.contains("(set-logic ALL)"));
        assert!(result.contains("(= x 1)"));
        assert!(result.contains("(check-sat)"));
    }

    #[test]
    fn test_smtlib_forall() {
        let expr = MathIR::ForAll(
            Variable { id: "T".into(), domain: Domain::UserDefined("Topology".into()), assumptions: AssumptionSet::default() },
            Box::new(Domain::UserDefined("Topology".into())),
            Box::new(MathIR::ForAll(
                Variable { id: "e".into(), domain: Domain::UserDefined("Edge".into()), assumptions: AssumptionSet::default() },
                Box::new(Domain::UserDefined("Edge".into())),
                Box::new(MathIR::Iff(
                    Box::new(MathIR::Fn { name: "Mem".into(), args: vec![
                        MathIR::Var(Box::new(Variable { id: "e".into(), domain: Domain::UserDefined("Edge".into()), assumptions: AssumptionSet::default() })),
                        MathIR::Var(Box::new(Variable { id: "T".into(), domain: Domain::UserDefined("Topology".into()), assumptions: AssumptionSet::default() })),
                    ] }),
                    Box::new(MathIR::Fn { name: "Mem".into(), args: vec![
                        MathIR::Var(Box::new(Variable { id: "e".into(), domain: Domain::UserDefined("Edge".into()), assumptions: AssumptionSet::default() })),
                        MathIR::Fn { name: "Compile".into(), args: vec![
                            MathIR::Var(Box::new(Variable { id: "T".into(), domain: Domain::UserDefined("Topology".into()), assumptions: AssumptionSet::default() })),
                        ] },
                    ] }),
                )),
            )),
        );
        let result = emit_smtlib(&expr);
        assert!(result.contains("(declare-sort Topology 0)"));
        assert!(result.contains("(declare-sort Edge 0)"));
        assert!(result.contains("(forall ((T Topology))"));
        assert!(result.contains("(check-sat)"));
    }
}
