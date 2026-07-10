use serde::{Serialize, Deserialize};
use super::core::ProofCore;
use super::backends::ProofOutput;

/// CheckerOutput — raw output from running a proof checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerOutput {
    pub target: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timestamp: String,
}

/// ProofSource — where a theorem came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofSource {
    Latex(String),
    LeanFile(String),
    IsabelleFile(String),
    MathIR(String),
    Manifest(String),
}

/// IngestedProof — a theorem bundle ready for backend emission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestedProof {
    pub core: ProofCore,
    pub source: ProofSource,
    pub outputs: Vec<ProofOutput>,
    pub checker_results: Vec<CheckerOutput>,
}

impl IngestedProof {
    pub fn new(core: ProofCore, source: ProofSource) -> Self {
        Self {
            core,
            source,
            outputs: Vec::new(),
            checker_results: Vec::new(),
        }
    }

    /// Emit to all backends.
    pub fn emit_all(&mut self) {
        self.outputs = super::backends::emit_all_backends(&self.core);
    }

    /// Attach a checker result to the matching backend output.
    pub fn attach_checker_result(&mut self, result: CheckerOutput) {
        for output in &mut self.outputs {
            if output.target == result.target {
                if let Some(code) = result.exit_code {
                    output.with_checker_result(result.stdout.clone(), code);
                }
            }
        }
        self.checker_results.push(result);
    }

    /// Attach WORM receipt hash to all outputs.
    pub fn attach_worm(&mut self, hash: &str) {
        for output in &mut self.outputs {
            output.with_worm(hash.to_string());
        }
    }
}
