use serde::{Serialize, Deserialize};
use super::core::ProofCore;
use crate::{MathIR, Domain};

/// TheoremEntry — a single theorem in a proof manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremEntry {
    pub id: String,
    pub name: String,
    pub statement: String,
    pub file_isabelle: Option<String>,
    pub file_lean4: Option<String>,
    pub status: String,
}

/// CheckerInfo — build metadata for a proof checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerInfo {
    pub version: String,
    pub build_command: Option<String>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub timestamp: Option<String>,
}

/// ScannerResults — counts of forbidden tokens found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerResults {
    pub sorry_count: u32,
    pub admit_count: u32,
    pub oops_count: u32,
    pub axiom_count: u32,
}

/// ProofManifest — the complete manifest for a theorem bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofManifest {
    pub project: String,
    pub version: String,
    pub created: String,
    pub theorems: Vec<TheoremEntry>,
    pub targets: Vec<String>,
    pub status: String,
    pub checkers: std::collections::HashMap<String, CheckerInfo>,
    pub forbidden_tokens: Vec<String>,
    pub scanner_results: std::collections::HashMap<String, ScannerResults>,
}

impl ProofManifest {
    /// Load a manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert a TheoremEntry into a ProofCore (requires parsing the statement).
    pub fn theorem_to_core(entry: &TheoremEntry) -> ProofCore {
        let statement = MathIR::Const(crate::Constant::Int(0));
        ProofCore::from_manifest(
            &entry.name,
            statement,
            vec![],
            vec![],
            Domain::Real,
        )
    }

    /// Get all theorem names.
    pub fn theorem_names(&self) -> Vec<&str> {
        self.theorems.iter().map(|t| t.name.as_str()).collect()
    }

    /// Check if all theorems are verified.
    pub fn all_verified(&self) -> bool {
        self.theorems.iter().all(|t| t.status.contains("verified") || t.status.contains("checked"))
    }

    /// Update status based on checker results.
    pub fn update_status(&mut self) {
        let all_ok = self.all_verified();
        if all_ok && !self.theorems.is_empty() {
            self.status = "machine_checked_all".to_string();
        } else if self.theorems.is_empty() {
            self.status = "empty".to_string();
        } else {
            self.status = "partial".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_json() {
        let json = r#"{
            "project": "Test",
            "version": "0.1.0",
            "created": "2026-07-05T00:00:00Z",
            "theorems": [],
            "targets": ["Isabelle/HOL"],
            "status": "empty",
            "checkers": {},
            "forbidden_tokens": ["sorry"],
            "scanner_results": {}
        }"#;
        let manifest = ProofManifest::from_json(json).unwrap();
        assert_eq!(manifest.project, "Test");
        assert!(manifest.theorems.is_empty());
    }

    #[test]
    fn test_manifest_to_json() {
        let manifest = ProofManifest {
            project: "Test".to_string(),
            version: "0.1.0".to_string(),
            created: "2026-07-05T00:00:00Z".to_string(),
            theorems: vec![],
            targets: vec!["Isabelle/HOL".to_string()],
            status: "empty".to_string(),
            checkers: std::collections::HashMap::new(),
            forbidden_tokens: vec!["sorry".to_string()],
            scanner_results: std::collections::HashMap::new(),
        };
        let json = manifest.to_json().unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_theorem_names() {
        let manifest = ProofManifest {
            project: "Test".to_string(),
            version: "0.1.0".to_string(),
            created: "2026-07-05T00:00:00Z".to_string(),
            theorems: vec![
                TheoremEntry {
                    id: "t1".to_string(),
                    name: "Theorem 1".to_string(),
                    statement: "x = 1".to_string(),
                    file_isabelle: None,
                    file_lean4: None,
                    status: "verified".to_string(),
                },
            ],
            targets: vec![],
            status: "verified".to_string(),
            checkers: std::collections::HashMap::new(),
            forbidden_tokens: vec![],
            scanner_results: std::collections::HashMap::new(),
        };
        assert_eq!(manifest.theorem_names(), vec!["Theorem 1"]);
        assert!(manifest.all_verified());
    }
}
