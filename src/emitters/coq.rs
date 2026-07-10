use crate::MathIR;
use super::{Emitter, EmittedTarget, ProofBackend, ProofStatus};

pub struct CoqEmitter;

impl Emitter for CoqEmitter {
    fn backend(&self) -> ProofBackend {
        ProofBackend::Coq
    }

    fn emit(&self, expr: &MathIR, theory_name: &str) -> EmittedTarget {
        let source = emit_coq(expr, theory_name);
        EmittedTarget {
            backend: ProofBackend::Coq,
            source,
            status: ProofStatus::EmittedCoq,
        }
    }
}

fn emit_coq(expr: &MathIR, theory_name: &str) -> String {
    let mut out = String::new();

    let types = collect_types(expr);
    for t in &types {
        out.push_str(&format!("Parameter {} : Type.\n", t));
    }
    if !types.is_empty() {
        out.push('\n');
    }

    let consts = collect_consts(expr);
    for (name, ty) in &consts {
        out.push_str(&format!("Parameter {} : {}.\n", name, ty));
    }
    if !consts.is_empty() {
        out.push('\n');
    }

    let thm_name = sanitize_ident(theory_name);
    out.push_str(&format!("Theorem {} :\n", thm_name));
    out.push_str(&format!("  {}.\n", mathir_to_coq(expr)));
    out.push_str("Proof.\n");
    out.push_str("Admitted.\n");
    out
}

fn mathir_to_coq(expr: &MathIR) -> String {
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
                crate::SymbolicConst::Pi => "PI".to_string(),
                crate::SymbolicConst::E => "E".to_string(),
                crate::SymbolicConst::Infinity => "infty".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        MathIR::Var(v) => v.id.clone(),
        MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            format!("({})", parts.join(" + "))
        }
        MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            format!("({})", parts.join(" * "))
        }
        MathIR::Pow(base, exp) => {
            format!("({} ^ {})", mathir_to_coq(base), mathir_to_coq(exp))
        }
        MathIR::Fn { name, args } => {
            if args.is_empty() {
                name.as_str().to_string()
            } else {
                let args_str: Vec<String> = args.iter().map(mathir_to_coq).collect();
                format!("({} {})", name.as_str(), args_str.join(" "))
            }
        }
        MathIR::Eq(lhs, rhs) => {
            format!("{} = {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Neq(lhs, rhs) => {
            format!("{} <> {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Lt(lhs, rhs) => {
            format!("{} < {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Lte(lhs, rhs) => {
            format!("{} <= {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Gt(lhs, rhs) => {
            format!("{} > {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Gte(lhs, rhs) => {
            format!("{} >= {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            parts.join(" /\\ ")
        }
        MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            parts.join(" \\/ ")
        }
        MathIR::Not(inner) => {
            format!("~ {}", mathir_to_coq(inner))
        }
        MathIR::Implies(lhs, rhs) => {
            format!("{} -> {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::Iff(lhs, rhs) => {
            format!("{} <-> {}", mathir_to_coq(lhs), mathir_to_coq(rhs))
        }
        MathIR::ForAll(var, _domain, body) => {
            format!("forall ({} : {}), {}", var.id, coq_type(&var.domain), mathir_to_coq(body))
        }
        MathIR::Exists(var, _domain, body) => {
            format!("exists ({} : {}), {}", var.id, coq_type(&var.domain), mathir_to_coq(body))
        }
        MathIR::Derivative(expr, var) => {
            format!("deriv (fun {} => {}) {}", var.id, mathir_to_coq(expr), var.id)
        }
        MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!("RInt (fun {} => {}) {} {}", var.id, mathir_to_coq(expr), mathir_to_coq(lo), mathir_to_coq(hi))
                }
                None => {
                    format!("RInt (fun {} => {}) 0 0", var.id, mathir_to_coq(expr))
                }
            }
        }
        _ => "??".to_string(),
    }
}

fn coq_type(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Integer => "Z".to_string(),
        crate::Domain::Rational => "Q".to_string(),
        crate::Domain::Real => "R".to_string(),
        crate::Domain::Complex => "C".to_string(),
        crate::Domain::UserDefined(name) => name.clone(),
        _ => "Type".to_string(),
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
                let arg_tys: Vec<String> = args.iter().map(|_| "Type".to_string()).collect();
                consts.push((name.as_str().to_string(), format!("{} -> Prop", arg_tys.join(" -> "))));
            }
        }
        _ => {}
    }
    consts.sort_by(|a, b| a.0.cmp(&b.0));
    consts.dedup();
    consts
}

fn sanitize_ident(name: &str) -> String {
    name.chars().map(|c| {
        if c.is_alphanumeric() || c == '_' { c } else { '_' }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_coq_equality() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let result = emit_coq(&expr, "test_theory");
        assert!(result.contains("Theorem test_theory :"));
        assert!(result.contains("x = 1"));
        assert!(result.contains("Admitted."));
    }

    #[test]
    fn test_coq_forall() {
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
        let result = emit_coq(&expr, "topology_preservation");
        assert!(result.contains("Parameter Topology : Type."));
        assert!(result.contains("Parameter Edge : Type."));
        assert!(result.contains("Admitted."));
    }
}
