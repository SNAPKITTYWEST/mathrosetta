use crate::MathIR;
use super::{Emitter, EmittedTarget, ProofBackend, ProofStatus};

pub struct Lean4Emitter;

impl Emitter for Lean4Emitter {
    fn backend(&self) -> ProofBackend {
        ProofBackend::Lean4
    }

    fn emit(&self, expr: &MathIR, theory_name: &str) -> EmittedTarget {
        let source = emit_lean4(expr, theory_name);
        EmittedTarget {
            backend: ProofBackend::Lean4,
            source,
            status: ProofStatus::EmittedLean4,
        }
    }
}

fn emit_lean4(expr: &MathIR, theory_name: &str) -> String {
    let mut out = String::new();
    out.push_str("import Mathlib\n\n");

    let namespace = sanitize_namespace(theory_name);
    out.push_str(&format!("namespace {}\n\n", namespace));

    let types = collect_types(expr);
    for t in &types {
        out.push_str(&format!("axiom {} : Type\n", t));
    }
    if !types.is_empty() {
        out.push('\n');
    }

    let consts = collect_consts(expr);
    for (name, ty) in &consts {
        out.push_str(&format!("axiom {} : {}\n", name, ty));
    }
    if !consts.is_empty() {
        out.push('\n');
    }

    let thm_name = sanitize_ident(theory_name);
    out.push_str(&format!("theorem {} :\n", thm_name));
    out.push_str(&format!("  {}\n", mathir_to_lean4(expr)));
    out.push_str("  := by\n");
    out.push_str("  sorry\n\n");

    out.push_str(&format!("end {}\n", namespace));
    out
}

fn mathir_to_lean4(expr: &MathIR) -> String {
    match expr {
        MathIR::Const(c) => match c {
            crate::Constant::Int(n) => {
                if *n < 0 {
                    format!("({})", n)
                } else {
                    n.to_string()
                }
            }
            crate::Constant::Float(f) => format!("({:.1?})", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "Real.pi".to_string(),
                crate::SymbolicConst::E => "Real.exp 1".to_string(),
                crate::SymbolicConst::Infinity => "⊤".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        MathIR::Var(v) => v.id.clone(),
        MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" + ")
        }
        MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" * ")
        }
        MathIR::Pow(base, exp) => {
            format!("{} ^ {}", mathir_to_lean4(base), mathir_to_lean4(exp))
        }
        MathIR::Fn { name, args } => {
            if args.is_empty() {
                name.as_str().to_string()
            } else {
                let args_str: Vec<String> = args.iter().map(mathir_to_lean4).collect();
                format!("{} {}", name.as_str(), args_str.join(" "))
            }
        }
        MathIR::Eq(lhs, rhs) => {
            format!("{} = {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Neq(lhs, rhs) => {
            format!("{} ≠ {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Lt(lhs, rhs) => {
            format!("{} < {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Lte(lhs, rhs) => {
            format!("{} ≤ {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Gt(lhs, rhs) => {
            format!("{} > {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Gte(lhs, rhs) => {
            format!("{} ≥ {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" ∧ ")
        }
        MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" ∨ ")
        }
        MathIR::Not(inner) => {
            format!("¬ {}", mathir_to_lean4(inner))
        }
        MathIR::Implies(lhs, rhs) => {
            format!("{} → {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::Iff(lhs, rhs) => {
            format!("{} ↔ {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs))
        }
        MathIR::ForAll(var, _domain, body) => {
            format!("∀ {} : {}, {}", var.id, lean_type(&var.domain), mathir_to_lean4(body))
        }
        MathIR::Exists(var, _domain, body) => {
            format!("∃ {} : {}, {}", var.id, lean_type(&var.domain), mathir_to_lean4(body))
        }
        MathIR::Derivative(expr, var) => {
            format!("deriv (fun {} => {}) {}", var.id, mathir_to_lean4(expr), var.id)
        }
        MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!("∫ {} in {}..{}, {}", var.id, mathir_to_lean4(lo), mathir_to_lean4(hi), mathir_to_lean4(expr))
                }
                None => {
                    format!("∫ {}, {}", var.id, mathir_to_lean4(expr))
                }
            }
        }
        _ => "??".to_string(),
    }
}

fn lean_type(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Integer => "Int".to_string(),
        crate::Domain::Rational => "Rat".to_string(),
        crate::Domain::Real => "Real".to_string(),
        crate::Domain::Complex => "Complex".to_string(),
        crate::Domain::UserDefined(name) => name.clone(),
        _ => "Any".to_string(),
    }
}

fn collect_types(expr: &MathIR) -> Vec<String> {
    let mut types = Vec::new();
    collect_types_rec(expr, &mut types);
    types.sort();
    types.dedup();
    types
}

fn collect_types_rec(expr: &MathIR, types: &mut Vec<String>) {
    match expr {
        MathIR::ForAll(var, _, body) | MathIR::Exists(var, _, body) => {
            if let crate::Domain::UserDefined(name) = &var.domain {
                if !types.contains(name) {
                    types.push(name.clone());
                }
            }
            collect_types_rec(body, types);
        }
        _ => {}
    }
}

fn collect_consts(expr: &MathIR) -> Vec<(String, String)> {
    let mut consts = Vec::new();
    match expr {
        MathIR::Fn { name, args } => {
            if !args.is_empty() {
                let arg_tys: Vec<String> = args.iter().map(|_| "α".to_string()).collect();
                consts.push((name.as_str().to_string(), format!("{} → Prop", arg_tys.join(" → "))));
            }
        }
        _ => {}
    }
    consts.sort_by(|a, b| a.0.cmp(&b.0));
    consts.dedup();
    consts
}

fn sanitize_namespace(name: &str) -> String {
    let s: String = name.chars().map(|c| {
        if c.is_alphanumeric() || c == '_' { c } else { '_' }
    }).collect();
    let s = s.trim_start_matches(|c: char| c.is_ascii_digit()).to_string();
    if s.is_empty() { "Exo".to_string() } else { s }
}

fn sanitize_ident(name: &str) -> String {
    let s = sanitize_namespace(name);
    if s.is_empty() { "theorem".to_string() } else { s.to_lowercase() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_lean4_equality() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let result = emit_lean4(&expr, "test_theory");
        assert!(result.contains("import Mathlib"));
        assert!(result.contains("namespace"));
        assert!(result.contains("x = 1"));
        assert!(result.contains("sorry"));
    }

    #[test]
    fn test_lean4_forall() {
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
        let result = emit_lean4(&expr, "topology_preservation");
        assert!(result.contains("axiom Topology : Type"));
        assert!(result.contains("axiom Edge : Type"));
        assert!(result.contains("sorry"));
    }
}
