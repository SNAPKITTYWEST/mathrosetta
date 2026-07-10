use serde::{Serialize, Deserialize};
use crate::MathIR;
use crate::emitters::{EmittedTarget, ProofBackend, emit_all};
use super::status::ProofTracker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    pub input_latex: String,
    pub mathir: MathIR,
    pub normalized: MathIR,
    pub theory_name: String,
    pub targets: Vec<EmittedTarget>,
    pub tracker: ProofTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundleExport {
    pub version: String,
    pub theory_name: String,
    pub input_latex: String,
    pub mathir_json: String,
    pub normalized_json: String,
    pub targets: Vec<TargetExport>,
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetExport {
    pub backend: String,
    pub source: String,
    pub status: String,
}

impl ProofBundle {
    pub fn new(input_latex: &str, mathir: MathIR, normalized: MathIR, theory_name: &str) -> Self {
        let targets = emit_all(&normalized, theory_name);
        let mut tracker = ProofTracker::new();
        tracker.advance(crate::emitters::ProofStatus::Normalized, "MathIR normalized");
        tracker.advance(crate::emitters::ProofStatus::AstGenerated, "AST generated");

        for target in &targets {
            let msg = format!("{} proof target emitted", match target.backend {
                ProofBackend::Isabelle => "Isabelle/HOL",
                ProofBackend::Lean4 => "Lean 4",
                ProofBackend::Coq => "Coq",
                ProofBackend::SmtLib => "SMT-LIB",
                ProofBackend::Latex => "LaTeX report",
            });
            tracker.advance(target.status.clone(), &msg);
        }

        ProofBundle {
            input_latex: input_latex.to_string(),
            mathir,
            normalized,
            theory_name: theory_name.to_string(),
            targets,
            tracker,
        }
    }

    pub fn to_export(&self) -> ProofBundleExport {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ProofBundleExport {
            version: "0.1.0".to_string(),
            theory_name: self.theory_name.clone(),
            input_latex: self.input_latex.clone(),
            mathir_json: serde_json::to_string_pretty(&self.mathir).unwrap_or_default(),
            normalized_json: serde_json::to_string_pretty(&self.normalized).unwrap_or_default(),
            targets: self.targets.iter().map(|t| TargetExport {
                backend: format!("{:?}", t.backend),
                source: t.source.clone(),
                status: "emitted_pending_proof".to_string(),
            }).collect(),
            status: "emitted_pending_proof".to_string(),
            timestamp: format!("{}", now),
        }
    }

    pub fn get_target(&self, backend: ProofBackend) -> Option<&EmittedTarget> {
        self.targets.iter().find(|t| t.backend == backend)
    }

    pub fn status_label(&self) -> String {
        self.tracker.status_label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    #[test]
    fn test_bundle_creation() {
        let mathir = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let bundle = ProofBundle::new("x = 1", mathir.clone(), mathir, "test");
        assert_eq!(bundle.targets.len(), 5);
        assert!(bundle.get_target(ProofBackend::Isabelle).is_some());
        assert!(bundle.get_target(ProofBackend::Lean4).is_some());
        assert!(bundle.get_target(ProofBackend::Coq).is_some());
        assert!(bundle.get_target(ProofBackend::SmtLib).is_some());
        assert!(bundle.get_target(ProofBackend::Latex).is_some());
    }

    #[test]
    fn test_bundle_export() {
        let mathir = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let bundle = ProofBundle::new("x = 1", mathir.clone(), mathir, "test");
        let export = bundle.to_export();
        assert_eq!(export.version, "0.1.0");
        assert_eq!(export.status, "emitted_pending_proof");
        assert_eq!(export.targets.len(), 5);
    }

    #[test]
    fn test_bundle_status_pending() {
        let mathir = MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }));
        let bundle = ProofBundle::new("x", mathir.clone(), mathir, "test");
        assert!(bundle.tracker.is_pending());
        assert!(!bundle.tracker.is_verified());
    }
}
