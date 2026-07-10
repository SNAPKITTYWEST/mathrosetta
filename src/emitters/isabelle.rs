use crate::MathIR;
use super::{Emitter, EmittedTarget, ProofBackend, ProofStatus};

pub struct IsabelleEmitter;

impl Emitter for IsabelleEmitter {
    fn backend(&self) -> ProofBackend {
        ProofBackend::Isabelle
    }

    fn emit(&self, expr: &MathIR, theory_name: &str) -> EmittedTarget {
        let source = emit_isabelle(expr, theory_name);
        EmittedTarget {
            backend: ProofBackend::Isabelle,
            source,
            status: ProofStatus::EmittedIsabelle,
        }
    }
}

fn emit_isabelle(expr: &MathIR, theory_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("theory {}\n", theory_name));
    out.push_str("  imports Main\nbegin\n\n");

    let types = collect_types(expr);
    for t in &types {
        out.push_str(&format!("typedecl {}\n", t));
    }
    if !types.is_empty() {
        out.push('\n');
    }

    let consts = collect_consts(expr);
    for (name, ty) in &consts {
        out.push_str(&format!("consts {} :: \"{}\"\n", name, ty));
    }
    if !consts.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("theorem {}:\n", sanitize_name(theory_name)));
    out.push_str(&format!("  \"{}\"\n", mathir_to_isabelle(expr)));
    out.push_str("  sorry\n\nend\n");
    out
}

fn mathir_to_isabelle(expr: &MathIR) -> String {
    match expr {
        MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.1?}", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "pi".to_string(),
                crate::SymbolicConst::E => "exp 1".to_string(),
                crate::SymbolicConst::Infinity => "∞".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        MathIR::Var(v) => v.id.clone(),
        MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" + ")
        }
        MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" * ")
        }
        MathIR::Pow(base, exp) => {
            format!("{} ^ {}", mathir_to_isabelle(base), mathir_to_isabelle(exp))
        }
        MathIR::Fn { name, args } => {
            let args_str: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            format!("{} {}", name.as_str(), args_str.join(" "))
        }
        MathIR::Eq(lhs, rhs) => {
            format!("{} = {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Neq(lhs, rhs) => {
            format!("{} ≠ {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Lt(lhs, rhs) => {
            format!("{} < {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Lte(lhs, rhs) => {
            format!("{} ≤ {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Gt(lhs, rhs) => {
            format!("{} > {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Gte(lhs, rhs) => {
            format!("{} ≥ {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" ∧ ")
        }
        MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" ∨ ")
        }
        MathIR::Not(inner) => {
            format!("¬ {}", mathir_to_isabelle(inner))
        }
        MathIR::Implies(lhs, rhs) => {
            format!("{} ⟶ {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::Iff(lhs, rhs) => {
            format!("{} ⟷ {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        MathIR::ForAll(var, _domain, body) => {
            format!("∀{}. {}", var.id, mathir_to_isabelle(body))
        }
        MathIR::Exists(var, _domain, body) => {
            format!("∃{}. {}", var.id, mathir_to_isabelle(body))
        }
        MathIR::Derivative(expr, var) => {
            format!("DERIV (λ{}. {})", var.id, mathir_to_isabelle(expr))
        }
        MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!("integral (λ{}. {}) {} {}", var.id, mathir_to_isabelle(expr),
                        mathir_to_isabelle(lo), mathir_to_isabelle(hi))
                }
                None => {
                    format!("integral (λ{}. {}) undefined undefined", var.id, mathir_to_isabelle(expr))
                }
            }
        }
        _ => "??".to_string(),
    }
}

fn collect_types(expr: &MathIR) -> Vec<String> {
    let mut types = Vec::new();
    match expr {
        MathIR::ForAll(var, _, _) | MathIR::Exists(var, _, _) => {
            if !types.contains(&var.domain_name()) {
                types.push(var.domain_name());
            }
        }
        _ => {}
    }
    types.sort();
    types.dedup();
    types
}

fn collect_consts(expr: &MathIR) -> Vec<(String, String)> {
    let mut consts = Vec::new();
    match expr {
        MathIR::Fn { name, .. } => {
            consts.push((name.as_str().to_string(), "set ⇒ set".to_string()));
        }
        _ => {}
    }
    consts.sort_by(|a, b| a.0.cmp(&b.0));
    consts.dedup();
    consts
}

fn sanitize_name(name: &str) -> String {
    name.chars().map(|c| {
        if c.is_alphanumeric() || c == '_' { c } else { '_' }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_isabelle_simple_equality() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let result = emit_isabelle(&expr, "test_theory");
        assert!(result.contains("theory test_theory"));
        assert!(result.contains("imports Main"));
        assert!(result.contains("x = 1"));
        assert!(result.contains("sorry"));
        assert!(result.contains("emitted_pending_proof") || result.contains("sorry"));
    }

    #[test]
    fn test_isabelle_forall() {
        let expr = MathIR::ForAll(
            Variable { id: "T".into(), domain: Domain::UserDefined("topology".into()), assumptions: AssumptionSet::default() },
            Box::new(Domain::UserDefined("topology".into())),
            Box::new(MathIR::ForAll(
                Variable { id: "e".into(), domain: Domain::UserDefined("edge".into()), assumptions: AssumptionSet::default() },
                Box::new(Domain::UserDefined("edge".into())),
                Box::new(MathIR::Iff(
                    Box::new(MathIR::Fn { name: "Mem".into(), args: vec![
                        MathIR::Var(Box::new(Variable { id: "e".into(), domain: Domain::UserDefined("edge".into()), assumptions: AssumptionSet::default() })),
                        MathIR::Var(Box::new(Variable { id: "T".into(), domain: Domain::UserDefined("topology".into()), assumptions: AssumptionSet::default() })),
                    ] }),
                    Box::new(MathIR::Fn { name: "Mem".into(), args: vec![
                        MathIR::Var(Box::new(Variable { id: "e".into(), domain: Domain::UserDefined("edge".into()), assumptions: AssumptionSet::default() })),
                        MathIR::Fn { name: "Compile".into(), args: vec![
                            MathIR::Var(Box::new(Variable { id: "T".into(), domain: Domain::UserDefined("topology".into()), assumptions: AssumptionSet::default() })),
                        ] },
                    ] }),
                )),
            )),
        );
        let result = emit_isabelle(&expr, "topology_preservation");
        assert!(result.contains("∀T. ∀e."));
        assert!(result.contains("sorry"));
    }
}
