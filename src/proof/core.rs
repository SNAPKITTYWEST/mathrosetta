use serde::{Serialize, Deserialize};
use crate::{MathIR, Variable, Domain};

/// ProofCore — minimal intermediate representation for theorem bundles.
/// All theorem inputs normalize into ProofCore before any backend emits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCore {
    pub theorem_name: String,
    pub statement: MathIR,
    pub assumptions: Vec<MathIR>,
    pub variables: Vec<Variable>,
    pub domain: Domain,
    pub input_latex: Option<String>,
    pub source_format: String,
}

/// Placeholder scan results — forbidden tokens that must not appear in proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaceholderScan {
    pub has_sorry: bool,
    pub has_admit: bool,
    pub has_axiom: bool,
    pub has_classical: bool,
    pub tokens_found: Vec<String>,
}

impl PlaceholderScan {
    pub fn clean() -> Self {
        Self {
            has_sorry: false,
            has_admit: false,
            has_axiom: false,
            has_classical: false,
            tokens_found: Vec::new(),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.tokens_found.is_empty()
    }
}

/// Scan source text for forbidden placeholder tokens.
pub fn scan_placeholders(source: &str, forbidden: &[&str]) -> PlaceholderScan {
    let mut scan = PlaceholderScan::clean();
    for token in forbidden {
        if source.contains(token) {
            scan.tokens_found.push(token.to_string());
            match *token {
                "sorry" => scan.has_sorry = true,
                "admit" | "Admitted" => scan.has_admit = true,
                "axiom" | "axiomatization" => scan.has_axiom = true,
                "Classical.choice" => scan.has_classical = true,
                _ => {}
            }
        }
    }
    scan
}

/// Standard forbidden tokens per backend.
pub fn forbidden_tokens(target: &str) -> Vec<&'static str> {
    match target {
        "isabelle" => vec!["sorry", "oops", "admit", "axiomatization"],
        "lean4" => vec!["sorry", "admit", "Admitted"],
        "coq" => vec!["Admitted", "admit", "Axiom"],
        "idris" => vec!["primitive", "believe_me"],
        "smtlib" => vec![],
        "latex" => vec![],
        "apl" => vec![],
        _ => vec![],
    }
}

impl ProofCore {
    /// Build a ProofCore from a MathIR expression and metadata.
    pub fn from_mathir(
        theorem_name: &str,
        statement: MathIR,
        input_latex: Option<String>,
    ) -> Self {
        let (variables, assumptions, domain) = extract_metadata(&statement);
        Self {
            theorem_name: theorem_name.to_string(),
            statement,
            assumptions,
            variables,
            domain,
            input_latex,
            source_format: "mathir".to_string(),
        }
    }

    /// Build a ProofCore from a proof manifest entry.
    pub fn from_manifest(
        theorem_name: &str,
        statement: MathIR,
        assumptions: Vec<MathIR>,
        variables: Vec<Variable>,
        domain: Domain,
    ) -> Self {
        Self {
            theorem_name: theorem_name.to_string(),
            statement,
            assumptions,
            variables,
            domain,
            input_latex: None,
            source_format: "manifest".to_string(),
        }
    }
}

/// Extract variables, assumptions, and domain from a MathIR expression.
fn extract_metadata(expr: &MathIR) -> (Vec<Variable>, Vec<MathIR>, Domain) {
    let mut variables = Vec::new();
    let mut assumptions = Vec::new();
    let mut domain = Domain::Real;

    extract_metadata_rec(expr, &mut variables, &mut assumptions, &mut domain);

    variables.sort_by(|a, b| a.id.cmp(&b.id));
    variables.dedup_by(|a, b| a.id == b.id);

    (variables, assumptions, domain)
}

fn extract_metadata_rec(
    expr: &MathIR,
    variables: &mut Vec<Variable>,
    assumptions: &mut Vec<MathIR>,
    domain: &mut Domain,
) {
    match expr {
        MathIR::ForAll(var, dom, body) => {
            variables.push(var.clone());
            *domain = dom.as_ref().clone();
            extract_metadata_rec(body, variables, assumptions, domain);
        }
        MathIR::Exists(var, dom, body) => {
            variables.push(var.clone());
            *domain = dom.as_ref().clone();
            extract_metadata_rec(body, variables, assumptions, domain);
        }
        MathIR::Implies(hyp, body) => {
            assumptions.push(*hyp.clone());
            extract_metadata_rec(body, variables, assumptions, domain);
        }
        MathIR::Eq(lhs, rhs) => {
            extract_metadata_rec(lhs, variables, assumptions, domain);
            extract_metadata_rec(rhs, variables, assumptions, domain);
        }
        MathIR::Iff(lhs, rhs) => {
            extract_metadata_rec(lhs, variables, assumptions, domain);
            extract_metadata_rec(rhs, variables, assumptions, domain);
        }
        MathIR::And(args) | MathIR::Or(args) => {
            for arg in args {
                extract_metadata_rec(arg, variables, assumptions, domain);
            }
        }
        MathIR::Not(inner) => {
            extract_metadata_rec(inner, variables, assumptions, domain);
        }
        MathIR::Derivative(expr, var) => {
            variables.push(var.clone());
            extract_metadata_rec(expr, variables, assumptions, domain);
        }
        MathIR::Integral { expr, var, limits } => {
            variables.push(var.clone());
            extract_metadata_rec(expr, variables, assumptions, domain);
            if let Some((lo, hi)) = limits {
                extract_metadata_rec(lo, variables, assumptions, domain);
                extract_metadata_rec(hi, variables, assumptions, domain);
            }
        }
        MathIR::Sum { expr, var, limits } | MathIR::Product { expr, var, limits } => {
            variables.push(var.clone());
            extract_metadata_rec(expr, variables, assumptions, domain);
            extract_metadata_rec(&limits.0, variables, assumptions, domain);
            extract_metadata_rec(&limits.1, variables, assumptions, domain);
        }
        MathIR::Limit { expr, var, target, .. } => {
            variables.push(var.clone());
            extract_metadata_rec(expr, variables, assumptions, domain);
            extract_metadata_rec(target, variables, assumptions, domain);
        }
        MathIR::Add(args) | MathIR::Mul(args) => {
            for arg in args {
                extract_metadata_rec(arg, variables, assumptions, domain);
            }
        }
        MathIR::Pow(base, exp) => {
            extract_metadata_rec(base, variables, assumptions, domain);
            extract_metadata_rec(exp, variables, assumptions, domain);
        }
        MathIR::Fn { args, .. } => {
            for arg in args {
                extract_metadata_rec(arg, variables, assumptions, domain);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Constant, AssumptionSet};

    fn var(name: &str) -> Variable {
        Variable { id: name.into(), domain: Domain::Real, assumptions: AssumptionSet::default() }
    }

    #[test]
    fn test_proofcore_from_forall() {
        let expr = MathIR::ForAll(
            var("x"),
            Box::new(Domain::Real),
            Box::new(MathIR::Gte(
                Box::new(MathIR::Pow(
                    Box::new(MathIR::Var(Box::new(var("x")))),
                    Box::new(MathIR::Const(Constant::Int(2))),
                )),
                Box::new(MathIR::Const(Constant::Int(0))),
            )),
        );
        let core = ProofCore::from_mathir("nonneg_square", expr, None);
        assert_eq!(core.theorem_name, "nonneg_square");
        assert_eq!(core.variables.len(), 1);
        assert_eq!(core.variables[0].id, "x");
        assert_eq!(core.domain, Domain::Real);
    }

    #[test]
    fn test_proofcore_from_implies() {
        let expr = MathIR::Implies(
            Box::new(MathIR::Var(Box::new(var("P")))),
            Box::new(MathIR::Var(Box::new(var("Q")))),
        );
        let core = ProofCore::from_mathir("implication", expr, None);
        assert_eq!(core.assumptions.len(), 1);
    }

    #[test]
    fn test_scan_placeholders_clean() {
        let scan = scan_placeholders("theorem foo : True := by trivial", &["sorry", "admit"]);
        assert!(scan.is_clean());
    }

    #[test]
    fn test_scan_placeholders_sorry() {
        let scan = scan_placeholders("theorem foo : True := sorry", &["sorry", "admit"]);
        assert!(!scan.is_clean());
        assert!(scan.has_sorry);
    }

    #[test]
    fn test_forbidden_tokens() {
        let tokens = forbidden_tokens("lean4");
        assert!(tokens.contains(&"sorry"));
        assert!(tokens.contains(&"Admitted"));
    }

    #[test]
    fn test_proofcore_nested_forall() {
        let expr = MathIR::ForAll(
            var("x"),
            Box::new(Domain::Real),
            Box::new(MathIR::ForAll(
                var("y"),
                Box::new(Domain::Real),
                Box::new(MathIR::Eq(
                    Box::new(MathIR::Add(vec![
                        MathIR::Var(Box::new(var("x"))),
                        MathIR::Var(Box::new(var("y"))),
                    ])),
                    Box::new(MathIR::Add(vec![
                        MathIR::Var(Box::new(var("y"))),
                        MathIR::Var(Box::new(var("x"))),
                    ])),
                )),
            )),
        );
        let core = ProofCore::from_mathir("add_comm", expr, None);
        assert_eq!(core.variables.len(), 2);
        assert!(core.variables.iter().any(|v| v.id == "x"));
        assert!(core.variables.iter().any(|v| v.id == "y"));
    }
}
