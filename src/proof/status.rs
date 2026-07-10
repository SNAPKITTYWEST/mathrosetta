use serde::{Serialize, Deserialize};
use crate::emitters::ProofStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofTracker {
    pub states: Vec<ProofState>,
    pub current: ProofStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofState {
    pub status: ProofStatus,
    pub timestamp: String,
    pub message: String,
}

impl ProofTracker {
    pub fn new() -> Self {
        let now = chrono_now();
        Self {
            states: vec![ProofState {
                status: ProofStatus::Parsed,
                timestamp: now.clone(),
                message: "Input parsed successfully".to_string(),
            }],
            current: ProofStatus::Parsed,
        }
    }

    pub fn advance(&mut self, status: ProofStatus, message: &str) {
        let now = chrono_now();
        self.states.push(ProofState {
            status: status.clone(),
            timestamp: now,
            message: message.to_string(),
        });
        self.current = status;
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.current,
            ProofStatus::EmittedIsabelle |
            ProofStatus::EmittedLean4 |
            ProofStatus::EmittedCoq |
            ProofStatus::EmittedSmtlib |
            ProofStatus::LatexReportEmitted |
            ProofStatus::ProofPending
        )
    }

    pub fn is_verified(&self) -> bool {
        self.current == ProofStatus::Verified
    }

    pub fn is_failed(&self) -> bool {
        self.current == ProofStatus::Failed
    }

    pub fn status_label(&self) -> String {
        match &self.current {
            ProofStatus::Parsed => "Parsed".to_string(),
            ProofStatus::Normalized => "Normalized".to_string(),
            ProofStatus::AstGenerated => "AST Generated".to_string(),
            ProofStatus::EmittedIsabelle => "Emitted (Isabelle/HOL) — pending proof".to_string(),
            ProofStatus::EmittedLean4 => "Emitted (Lean 4) — pending proof".to_string(),
            ProofStatus::EmittedCoq => "Emitted (Coq) — pending proof".to_string(),
            ProofStatus::EmittedSmtlib => "Emitted (SMT-LIB) — pending proof".to_string(),
            ProofStatus::LatexReportEmitted => "LaTeX report emitted — not a proof".to_string(),
            ProofStatus::ProofPending => "Proof pending — awaiting external checker".to_string(),
            ProofStatus::Verified => "Verified by external checker".to_string(),
            ProofStatus::Failed => "Proof attempt failed".to_string(),
        }
    }

    pub fn display_checklist(&self) -> Vec<(String, bool)> {
        vec![
            ("Parsed".to_string(), self.has_state(&ProofStatus::Parsed)),
            ("Normalized".to_string(), self.has_state(&ProofStatus::Normalized)),
            ("AST Generated".to_string(), self.has_state(&ProofStatus::AstGenerated)),
            ("Isabelle Emitted".to_string(), self.has_state(&ProofStatus::EmittedIsabelle)),
            ("Lean 4 Emitted".to_string(), self.has_state(&ProofStatus::EmittedLean4)),
            ("Coq Emitted".to_string(), self.has_state(&ProofStatus::EmittedCoq)),
            ("SMT-LIB Emitted".to_string(), self.has_state(&ProofStatus::EmittedSmtlib)),
            ("LaTeX Report".to_string(), self.has_state(&ProofStatus::LatexReportEmitted)),
            ("Verified".to_string(), self.has_state(&ProofStatus::Verified)),
        ]
    }

    fn has_state(&self, status: &ProofStatus) -> bool {
        self.states.iter().any(|s| s.status == *status)
    }
}

impl Default for ProofTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}-{:02}-{:02}T00:00:00Z",
        1970 + (secs / 31536000) as u32,
        ((secs % 31536000) / 2592000) as u32 + 1,
        ((secs % 2592000) / 86400) as u32 + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_starts_parsed() {
        let tracker = ProofTracker::new();
        assert_eq!(tracker.current, ProofStatus::Parsed);
        assert!(tracker.is_pending() == false);
    }

    #[test]
    fn test_tracker_advance() {
        let mut tracker = ProofTracker::new();
        tracker.advance(ProofStatus::Normalized, "Normalized successfully");
        assert_eq!(tracker.current, ProofStatus::Normalized);
        assert!(tracker.states.len() == 2);
    }

    #[test]
    fn test_tracker_checklist() {
        let mut tracker = ProofTracker::new();
        tracker.advance(ProofStatus::Normalized, "done");
        tracker.advance(ProofStatus::AstGenerated, "done");
        tracker.advance(ProofStatus::EmittedIsabelle, "done");
        let checklist = tracker.display_checklist();
        assert!(checklist[0].1); // Parsed
        assert!(checklist[1].1); // Normalized
        assert!(checklist[2].1); // AST Generated
        assert!(checklist[3].1); // Isabelle
        assert!(!checklist[4].1); // Lean 4 not yet
    }
}
