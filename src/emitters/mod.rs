pub mod isabelle;
pub mod lean4;
pub mod coq;
pub mod smtlib;
pub mod latex;

use crate::MathIR;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmittedTarget {
    pub backend: ProofBackend,
    pub source: String,
    pub status: ProofStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofBackend {
    Isabelle,
    Lean4,
    Coq,
    SmtLib,
    Latex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofStatus {
    Parsed,
    Normalized,
    AstGenerated,
    EmittedIsabelle,
    EmittedLean4,
    EmittedCoq,
    EmittedSmtlib,
    LatexReportEmitted,
    ProofPending,
    Verified,
    Failed,
}

pub trait Emitter {
    fn emit(&self, expr: &MathIR, theory_name: &str) -> EmittedTarget;
    fn backend(&self) -> ProofBackend;
}

pub fn emit_all(expr: &MathIR, theory_name: &str) -> Vec<EmittedTarget> {
    vec![
        isabelle::IsabelleEmitter.emit(expr, theory_name),
        lean4::Lean4Emitter.emit(expr, theory_name),
        coq::CoqEmitter.emit(expr, theory_name),
        smtlib::SmtLibEmitter.emit(expr, theory_name),
        latex::LatexReportEmitter.emit(expr, theory_name),
    ]
}
