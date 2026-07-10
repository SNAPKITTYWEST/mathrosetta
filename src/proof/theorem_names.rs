use crate::MathIR;

pub struct TheoremNamer;

impl TheoremNamer {
    pub fn name_from_expr(expr: &MathIR) -> String {
        match expr {
            MathIR::Eq(lhs, rhs) => {
                let lhs_name = expr_short_name(lhs);
                let rhs_name = expr_short_name(rhs);
                format!("{}_eq_{}", lhs_name, rhs_name)
            }
            MathIR::ForAll(_, _, body) => {
                format!("forall_{}", expr_short_name(body))
            }
            MathIR::Exists(_, _, body) => {
                format!("exists_{}", expr_short_name(body))
            }
            MathIR::Iff(lhs, rhs) => {
                let lhs_name = expr_short_name(lhs);
                let rhs_name = expr_short_name(rhs);
                format!("{}_iff_{}", lhs_name, rhs_name)
            }
            MathIR::Implies(lhs, rhs) => {
                let lhs_name = expr_short_name(lhs);
                let rhs_name = expr_short_name(rhs);
                format!("{}_implies_{}", lhs_name, rhs_name)
            }
            MathIR::Integral { .. } => "integral_theorem".to_string(),
            MathIR::Derivative(_, var) => {
                format!("deriv_{}", var.id)
            }
            _ => "theorem".to_string(),
        }
    }

    pub fn name_from_description(desc: &str) -> String {
        desc.trim()
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .split('_')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    }
}

fn expr_short_name(expr: &MathIR) -> String {
    match expr {
        MathIR::Var(v) => v.id.clone(),
        MathIR::Const(c) => match c {
            crate::Constant::Int(n) => format!("n{}", n),
            crate::Constant::Float(f) => format!("f{}", *f as i64),
            crate::Constant::Symbolic(s) => format!("{:?}", s).to_lowercase(),
            _ => "c".to_string(),
        },
        MathIR::Fn { name, .. } => name.as_str().to_string(),
        MathIR::Add(_) => "sum".to_string(),
        MathIR::Mul(_) => "prod".to_string(),
        MathIR::Pow(_, _) => "pow".to_string(),
        MathIR::Eq(_, _) => "eq".to_string(),
        MathIR::Iff(_, _) => "iff".to_string(),
        MathIR::Derivative(_, var) => format!("d{}", var.id),
        MathIR::Integral { var, .. } => format!("int_{}", var.id),
        _ => "expr".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_name_from_equality() {
        let expr = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let name = TheoremNamer::name_from_expr(&expr);
        assert_eq!(name, "x_eq_n1");
    }

    #[test]
    fn test_name_from_description() {
        let name = TheoremNamer::name_from_description("Topology Preservation");
        assert_eq!(name, "topology_preservation");
    }

    #[test]
    fn test_name_from_forall() {
        let expr = MathIR::ForAll(
            Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() },
            Box::new(Domain::Real),
            Box::new(MathIR::Fn { name: "P".into(), args: vec![MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))] }),
        );
        let name = TheoremNamer::name_from_expr(&expr);
        assert_eq!(name, "forall_P");
    }
}
