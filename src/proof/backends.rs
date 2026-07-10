use serde::{Serialize, Deserialize};
use super::core::{ProofCore, PlaceholderScan, scan_placeholders, forbidden_tokens};

/// ProofStatus — machine state. Never claim verified without checker exit 0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofStatus {
    Parsed,
    Normalized,
    ProofCoreGenerated,
    Emitted { target: String },
    PlaceholderDetected,
    CheckerPending,
    CheckerRunning,
    CheckerPassed,
    CheckerFailed,
    WORMSealed,
    Verified,
    Failed,
}

/// ProofOutput — standardized result from every backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOutput {
    pub source: String,
    pub target: String,
    pub theorem_name: String,
    pub status: ProofStatus,
    pub checker_command: Option<String>,
    pub checker_output: Option<String>,
    pub checker_exit_code: Option<i32>,
    pub placeholder_scan: PlaceholderScan,
    pub worm_receipt_hash: Option<String>,
}

impl ProofOutput {
    /// Mark as emitted, scan for placeholders, never claim verified.
    pub fn emitted(source: String, target: &str, theorem_name: &str, checker_command: Option<String>) -> Self {
        let scan = scan_placeholders(&source, &forbidden_tokens(target));
        let status = if scan.is_clean() {
            ProofStatus::Emitted { target: target.to_string() }
        } else {
            ProofStatus::PlaceholderDetected
        };
        Self {
            source,
            target: target.to_string(),
            theorem_name: theorem_name.to_string(),
            status,
            checker_command,
            checker_output: None,
            checker_exit_code: None,
            placeholder_scan: scan,
            worm_receipt_hash: None,
        }
    }

    /// Attach checker results. Only mark Verified if exit_code == 0 AND scan clean.
    pub fn with_checker_result(&mut self, output: String, exit_code: i32) {
        self.checker_output = Some(output);
        self.checker_exit_code = Some(exit_code);
        if exit_code == 0 && self.placeholder_scan.is_clean() {
            self.status = ProofStatus::CheckerPassed;
        } else {
            self.status = ProofStatus::CheckerFailed;
        }
    }

    /// Attach WORM receipt hash.
    pub fn with_worm(&mut self, hash: String) {
        self.worm_receipt_hash = Some(hash);
        if self.status == ProofStatus::CheckerPassed {
            self.status = ProofStatus::WORMSealed;
        }
    }
}

/// Backend trait — every target language implements this.
pub trait ProofBackend {
    fn emit(&self, core: &ProofCore) -> ProofOutput;
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn checker_command(&self) -> Option<String>;
}

/// Emit to all backends.
pub fn emit_all_backends(core: &ProofCore) -> Vec<ProofOutput> {
    vec![
        IsabelleBackend.emit(core),
        Lean4Backend.emit(core),
        CoqBackend.emit(core),
        IdrisBackend.emit(core),
        SmtLibBackend.emit(core),
        LatexReportBackend.emit(core),
        AplTraceBackend.emit(core),
    ]
}

// ══════════════════════════════════════════════════════════════════════════════
// Isabelle/HOL Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct IsabelleBackend;

impl ProofBackend for IsabelleBackend {
    fn name(&self) -> &str { "Isabelle/HOL" }
    fn file_extension(&self) -> &str { ".thy" }
    fn checker_command(&self) -> Option<String> {
        Some("isabelle build -D .".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_isabelle(core);
        ProofOutput::emitted(source, "isabelle", &core.theorem_name, self.checker_command())
    }
}

fn emit_isabelle(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str(&format!("theory {}\n", sanitize_thy_name(&core.theorem_name)));
    out.push_str("  imports Main\nbegin\n\n");

    for var in &core.variables {
        out.push_str(&format!("(* variable: {} : {} )\n", var.id, domain_isabelle(&var.domain)));
    }
    if !core.variables.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("theorem {}:\n", sanitize_thy_name(&core.theorem_name)));
    out.push_str(&format!("  \"{}\"\n", mathir_to_isabelle(&core.statement)));
    out.push_str("  sorry\n\nend\n");
    out
}

fn mathir_to_isabelle(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.1?}", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "pi".to_string(),
                crate::SymbolicConst::E => "exp 1".to_string(),
                crate::SymbolicConst::Infinity => "\\infty".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" + ")
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" * ")
        }
        crate::MathIR::Pow(base, exp) => {
            format!("{} ^ {}", mathir_to_isabelle(base), mathir_to_isabelle(exp))
        }
        crate::MathIR::Fn { name, args } => {
            let args_str: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            format!("{} {}", name.as_str(), args_str.join(" "))
        }
        crate::MathIR::Eq(lhs, rhs) => {
            format!("{} = {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Neq(lhs, rhs) => {
            format!("{} \\neq {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Lt(lhs, rhs) => {
            format!("{} < {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Lte(lhs, rhs) => {
            format!("{} \\leq {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Gt(lhs, rhs) => {
            format!("{} > {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Gte(lhs, rhs) => {
            format!("{} \\geq {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" \\and ")
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_isabelle).collect();
            parts.join(" \\or ")
        }
        crate::MathIR::Not(inner) => {
            format!("\\not {}", mathir_to_isabelle(inner))
        }
        crate::MathIR::Implies(lhs, rhs) => {
            format!("{} \\longrightarrow {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::Iff(lhs, rhs) => {
            format!("{} \\longleftrightarrow {}", mathir_to_isabelle(lhs), mathir_to_isabelle(rhs))
        }
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("\\forall {}\\colon {}. {}", var.id, domain_isabelle(&var.domain), mathir_to_isabelle(body))
        }
        crate::MathIR::Exists(var, _domain, body) => {
            format!("\\exists {}\\colon {}. {}", var.id, domain_isabelle(&var.domain), mathir_to_isabelle(body))
        }
        crate::MathIR::Derivative(expr, var) => {
            format!("DERIV (\\lambda{}. {}) {}", var.id, mathir_to_isabelle(expr), var.id)
        }
        crate::MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => {
                    format!("integral (\\lambda{}. {}) {} {}", var.id, mathir_to_isabelle(expr),
                        mathir_to_isabelle(lo), mathir_to_isabelle(hi))
                }
                None => format!("integral (\\lambda{}. {}) undefined undefined", var.id, mathir_to_isabelle(expr))
            }
        }
        _ => "??".to_string(),
    }
}

fn domain_isabelle(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "real".to_string(),
        crate::Domain::Complex => "complex".to_string(),
        crate::Domain::Integer => "int".to_string(),
        crate::Domain::Rational => "rat".to_string(),
        crate::Domain::Natural => "nat".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "'a".to_string(),
    }
}

fn sanitize_thy_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

// ══════════════════════════════════════════════════════════════════════════════
// Lean 4 Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct Lean4Backend;

impl ProofBackend for Lean4Backend {
    fn name(&self) -> &str { "Lean 4" }
    fn file_extension(&self) -> &str { ".lean" }
    fn checker_command(&self) -> Option<String> {
        Some("lake build".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_lean4(core);
        ProofOutput::emitted(source, "lean4", &core.theorem_name, self.checker_command())
    }
}

fn emit_lean4(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str("import Mathlib\n\n");
    let ns = sanitize_ident(&core.theorem_name);
    out.push_str(&format!("namespace {}\n\n", ns));

    for var in &core.variables {
        out.push_str(&format!("variable ({} : {})\n", var.id, domain_lean(&var.domain)));
    }
    if !core.variables.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("theorem {} :\n", ns));
    out.push_str(&format!("  {}\n", mathir_to_lean4(&core.statement)));
    out.push_str("  := by\n");
    out.push_str("  sorry\n\n");
    out.push_str(&format!("end {}\n", ns));
    out
}

fn mathir_to_lean4(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => if *n < 0 { format!("({})", n) } else { n.to_string() },
            crate::Constant::Float(f) => format!("({:.1?})", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "Real.pi".to_string(),
                crate::SymbolicConst::E => "Real.exp 1".to_string(),
                crate::SymbolicConst::Infinity => "\\top".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" + ")
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" * ")
        }
        crate::MathIR::Pow(base, exp) => {
            format!("{} ^ {}", mathir_to_lean4(base), mathir_to_lean4(exp))
        }
        crate::MathIR::Fn { name, args } => {
            let args_str: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            format!("{} {}", name.as_str(), args_str.join(" "))
        }
        crate::MathIR::Eq(lhs, rhs) => format!("{} = {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Neq(lhs, rhs) => format!("{} \\neq {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Lt(lhs, rhs) => format!("{} < {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Lte(lhs, rhs) => format!("{} \\leq {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Gt(lhs, rhs) => format!("{} > {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Gte(lhs, rhs) => format!("{} \\geq {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" \\and ")
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_lean4).collect();
            parts.join(" \\or ")
        }
        crate::MathIR::Not(inner) => format!("\\neg {}", mathir_to_lean4(inner)),
        crate::MathIR::Implies(lhs, rhs) => format!("{} \\to {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::Iff(lhs, rhs) => format!("{} \\leftrightarrow {}", mathir_to_lean4(lhs), mathir_to_lean4(rhs)),
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("\\forall ({} : {}), {}", var.id, domain_lean(&var.domain), mathir_to_lean4(body))
        }
        crate::MathIR::Exists(var, _domain, body) => {
            format!("\\exists ({} : {}), {}", var.id, domain_lean(&var.domain), mathir_to_lean4(body))
        }
        crate::MathIR::Derivative(expr, var) => {
            format!("deriv (fun {} => {}) {}", var.id, mathir_to_lean4(expr), var.id)
        }
        crate::MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => format!("\\int_{}^{} {} \\, d{}", mathir_to_lean4(lo), mathir_to_lean4(hi), mathir_to_lean4(expr), var.id),
                None => format!("\\int {} \\, d{}", mathir_to_lean4(expr), var.id),
            }
        }
        _ => "??".to_string(),
    }
}

fn domain_lean(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "Real".to_string(),
        crate::Domain::Complex => "Complex".to_string(),
        crate::Domain::Integer => "Int".to_string(),
        crate::Domain::Rational => "Rat".to_string(),
        crate::Domain::Natural => "Nat".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "Type".to_string(),
    }
}

fn sanitize_ident(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect();
    let s = s.trim_start_matches(|c: char| c.is_ascii_digit()).to_string();
    if s.is_empty() { "theorem".to_string() } else { s.to_lowercase() }
}

// ══════════════════════════════════════════════════════════════════════════════
// Coq Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct CoqBackend;

impl ProofBackend for CoqBackend {
    fn name(&self) -> &str { "Coq" }
    fn file_extension(&self) -> &str { ".v" }
    fn checker_command(&self) -> Option<String> {
        Some("coqc".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_coq(core);
        ProofOutput::emitted(source, "coq", &core.theorem_name, self.checker_command())
    }
}

fn emit_coq(core: &ProofCore) -> String {
    let mut out = String::new();
    for var in &core.variables {
        out.push_str(&format!("Variable {} : {}.\n", var.id, domain_coq(&var.domain)));
    }
    if !core.variables.is_empty() {
        out.push('\n');
    }

    let thm_name = sanitize_ident(&core.theorem_name);
    out.push_str(&format!("Theorem {} :\n", thm_name));
    out.push_str(&format!("  {}.\n", mathir_to_coq(&core.statement)));
    out.push_str("Proof.\n");
    out.push_str("Admitted.\n");
    out
}

fn mathir_to_coq(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => if *n < 0 { format!("({})", n) } else { n.to_string() },
            crate::Constant::Float(f) => format!("({:.1?})", f),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "PI".to_string(),
                crate::SymbolicConst::E => "E".to_string(),
                crate::SymbolicConst::Infinity => "infty".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            format!("({})", parts.join(" + "))
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            format!("({})", parts.join(" * "))
        }
        crate::MathIR::Pow(base, exp) => format!("({} ^ {})", mathir_to_coq(base), mathir_to_coq(exp)),
        crate::MathIR::Fn { name, args } => {
            let args_str: Vec<String> = args.iter().map(mathir_to_coq).collect();
            format!("({} {})", name.as_str(), args_str.join(" "))
        }
        crate::MathIR::Eq(lhs, rhs) => format!("{} = {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Neq(lhs, rhs) => format!("{} <> {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Lt(lhs, rhs) => format!("{} < {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Lte(lhs, rhs) => format!("{} <= {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Gt(lhs, rhs) => format!("{} > {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Gte(lhs, rhs) => format!("{} >= {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            parts.join(" /\\ ")
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_coq).collect();
            parts.join(" \\/ ")
        }
        crate::MathIR::Not(inner) => format!("~ {}", mathir_to_coq(inner)),
        crate::MathIR::Implies(lhs, rhs) => format!("{} -> {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::Iff(lhs, rhs) => format!("{} <-> {}", mathir_to_coq(lhs), mathir_to_coq(rhs)),
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("forall ({} : {}), {}", var.id, domain_coq(&var.domain), mathir_to_coq(body))
        }
        crate::MathIR::Exists(var, _domain, body) => {
            format!("exists ({} : {}), {}", var.id, domain_coq(&var.domain), mathir_to_coq(body))
        }
        _ => "??".to_string(),
    }
}

fn domain_coq(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "R".to_string(),
        crate::Domain::Complex => "C".to_string(),
        crate::Domain::Integer => "Z".to_string(),
        crate::Domain::Rational => "Q".to_string(),
        crate::Domain::Natural => "nat".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "Type".to_string(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Idris Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct IdrisBackend;

impl ProofBackend for IdrisBackend {
    fn name(&self) -> &str { "Idris 2" }
    fn file_extension(&self) -> &str { ".idr" }
    fn checker_command(&self) -> Option<String> {
        Some("idris2 --check".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_idris(core);
        ProofOutput::emitted(source, "idris", &core.theorem_name, self.checker_command())
    }
}

fn emit_idris(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str("module Theorem\n\n");
    out.push_str("%default total\n\n");

    for var in &core.variables {
        out.push_str(&format!("{} : {}\n", var.id, domain_idris(&var.domain)));
    }
    if !core.variables.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("{} : {}\n", sanitize_ident(&core.theorem_name), mathir_to_idris(&core.statement)));
    out.push_str(&format!("{} = ?{}\n", sanitize_ident(&core.theorem_name), sanitize_ident(&core.theorem_name)));
    out
}

fn mathir_to_idris(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.1}", f),
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_idris).collect();
            parts.join(" + ")
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_idris).collect();
            parts.join(" * ")
        }
        crate::MathIR::Pow(base, exp) => format!("pow {} {}", mathir_to_idris(base), mathir_to_idris(exp)),
        crate::MathIR::Eq(lhs, rhs) => format!("{} = {}", mathir_to_idris(lhs), mathir_to_idris(rhs)),
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("({} : {}) -> {}", var.id, domain_idris(&var.domain), mathir_to_idris(body))
        }
        _ => "??".to_string(),
    }
}

fn domain_idris(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "Double".to_string(),
        crate::Domain::Integer => "Integer".to_string(),
        crate::Domain::Natural => "Nat".to_string(),
        crate::Domain::Rational => "Rational".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "Type".to_string(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// SMT-LIB Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct SmtLibBackend;

impl ProofBackend for SmtLibBackend {
    fn name(&self) -> &str { "SMT-LIB" }
    fn file_extension(&self) -> &str { ".smt2" }
    fn checker_command(&self) -> Option<String> {
        Some("z3".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_smtlib(core);
        ProofOutput::emitted(source, "smtlib", &core.theorem_name, self.checker_command())
    }
}

fn emit_smtlib(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str("(set-logic ALL)\n\n");

    for var in &core.variables {
        out.push_str(&format!("(declare-const {} {})\n", var.id, smt_sort(&var.domain)));
    }
    if !core.variables.is_empty() {
        out.push('\n');
    }

    out.push_str("(assert\n");
    out.push_str(&format!("  {}\n", mathir_to_smtlib(&core.statement, 2)));
    out.push_str(")\n\n");
    out.push_str("(check-sat)\n");
    out
}

fn mathir_to_smtlib(expr: &crate::MathIR, indent: usize) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.6e}", f),
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(+ {})", parts.join(" "))
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(* {})", parts.join(" "))
        }
        crate::MathIR::Pow(base, exp) => format!("(^ {} {})", mathir_to_smtlib(base, indent), mathir_to_smtlib(exp, indent)),
        crate::MathIR::Eq(lhs, rhs) => format!("(= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::Lt(lhs, rhs) => format!("(< {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::Lte(lhs, rhs) => format!("(<= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::Gt(lhs, rhs) => format!("(> {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::Gte(lhs, rhs) => format!("(>= {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(and {})", parts.join(" "))
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(|a| mathir_to_smtlib(a, indent)).collect();
            format!("(or {})", parts.join(" "))
        }
        crate::MathIR::Not(inner) => format!("(not {})", mathir_to_smtlib(inner, indent)),
        crate::MathIR::Implies(lhs, rhs) => format!("(=> {} {})", mathir_to_smtlib(lhs, indent), mathir_to_smtlib(rhs, indent)),
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("(forall (({} {}))\n{}{})", var.id, smt_sort(&var.domain), "  ".repeat(indent + 1), mathir_to_smtlib(body, indent + 1))
        }
        crate::MathIR::Exists(var, _domain, body) => {
            format!("(exists (({} {}))\n{}{})", var.id, smt_sort(&var.domain), "  ".repeat(indent + 1), mathir_to_smtlib(body, indent + 1))
        }
        _ => "??".to_string(),
    }
}

fn smt_sort(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "Real".to_string(),
        crate::Domain::Integer => "Int".to_string(),
        crate::Domain::Rational => "Real".to_string(),
        crate::Domain::Natural => "Int".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "Any".to_string(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// LaTeX Report Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct LatexReportBackend;

impl ProofBackend for LatexReportBackend {
    fn name(&self) -> &str { "LaTeX" }
    fn file_extension(&self) -> &str { ".tex" }
    fn checker_command(&self) -> Option<String> {
        Some("pdflatex".to_string())
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_latex(core);
        ProofOutput::emitted(source, "latex", &core.theorem_name, self.checker_command())
    }
}

fn emit_latex(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage{amsmath, amssymb, amsthm}\n");
    out.push_str("\\usepackage{geometry}\n");
    out.push_str("\\geometry{margin=1in}\n");
    out.push_str("\\newtheorem{theorem}{Theorem}\n\n");
    out.push_str(&format!("\\title{{{}}}\n", core.theorem_name.replace('_', " ")));
    out.push_str("\\author{MathRosetta}\n");
    out.push_str("\\date{\\today}\n\n");
    out.push_str("\\begin{document}\n");
    out.push_str("\\maketitle\n\n");

    out.push_str("\\begin{theorem}\n");
    out.push_str(&mathir_to_latex(&core.statement));
    out.push_str("\n\\end{theorem}\n\n");

    if !core.assumptions.is_empty() {
        out.push_str("\\section*{Assumptions}\n");
        out.push_str("\\begin{itemize}\n");
        for a in &core.assumptions {
            out.push_str(&format!("  \\item {}\n", mathir_to_latex(a)));
        }
        out.push_str("\\end{itemize}\n\n");
    }

    out.push_str("\\section*{Proof Status}\n");
    out.push_str("\\texttt{latex\\_report\\_emitted} — not a proof certificate.\n\n");
    out.push_str("\\end{document}\n");
    out
}

fn mathir_to_latex(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.4}", f),
            crate::Constant::Rational { numer, denom } => format!("\\frac{{{}}}{{{}}}", numer, denom),
            crate::Constant::Symbolic(s) => match s {
                crate::SymbolicConst::Pi => "\\pi".to_string(),
                crate::SymbolicConst::E => "e".to_string(),
                crate::SymbolicConst::Infinity => "\\infty".to_string(),
                _ => "?".to_string(),
            },
            _ => "?".to_string(),
        },
        crate::MathIR::Var(v) => v.id.clone(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex).collect();
            parts.join(" + ")
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex).collect();
            if args.len() == 2 { format!("{} \\cdot {}", parts[0], parts[1]) }
            else { parts.join(" \\cdot ") }
        }
        crate::MathIR::Pow(base, exp) => format!("{}^{{{}}}", mathir_to_latex(base), mathir_to_latex(exp)),
        crate::MathIR::Fn { name, args } => {
            let fn_name = match name.as_str() {
                "sin" => "\\sin", "cos" => "\\cos", "tan" => "\\tan",
                "log" => "\\log", "ln" => "\\ln", "exp" => "\\exp",
                other => other,
            };
            let args_str: Vec<String> = args.iter().map(mathir_to_latex).collect();
            format!("{}{{{}}}", fn_name, args_str.join(", "))
        }
        crate::MathIR::Eq(lhs, rhs) => format!("{} = {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Neq(lhs, rhs) => format!("{} \\neq {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Lt(lhs, rhs) => format!("{} < {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Lte(lhs, rhs) => format!("{} \\leq {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Gt(lhs, rhs) => format!("{} > {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Gte(lhs, rhs) => format!("{} \\geq {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex).collect();
            parts.join(" \\land ")
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_latex).collect();
            parts.join(" \\lor ")
        }
        crate::MathIR::Not(inner) => format!("\\lnot {}", mathir_to_latex(inner)),
        crate::MathIR::Implies(lhs, rhs) => format!("{} \\to {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::Iff(lhs, rhs) => format!("{} \\leftrightarrow {}", mathir_to_latex(lhs), mathir_to_latex(rhs)),
        crate::MathIR::ForAll(var, _domain, body) => {
            format!("\\forall {} \\in {}\\colon {}", var.id, domain_latex(&var.domain), mathir_to_latex(body))
        }
        crate::MathIR::Exists(var, _domain, body) => {
            format!("\\exists {} \\in {}\\colon {}", var.id, domain_latex(&var.domain), mathir_to_latex(body))
        }
        crate::MathIR::Derivative(expr, var) => {
            format!("\\frac{{d}}{{d{}}} {}", var.id, mathir_to_latex(expr))
        }
        crate::MathIR::Integral { expr, var, limits } => {
            match limits {
                Some((lo, hi)) => format!("\\int_{{{}}}^{{{}}} {} \\, d{}", mathir_to_latex(lo), mathir_to_latex(hi), mathir_to_latex(expr), var.id),
                None => format!("\\int {} \\, d{}", mathir_to_latex(expr), var.id),
            }
        }
        _ => "\\dots".to_string(),
    }
}

fn domain_latex(domain: &crate::Domain) -> String {
    match domain {
        crate::Domain::Real => "\\mathbb{R}".to_string(),
        crate::Domain::Complex => "\\mathbb{C}".to_string(),
        crate::Domain::Integer => "\\mathbb{Z}".to_string(),
        crate::Domain::Rational => "\\mathbb{Q}".to_string(),
        crate::Domain::Natural => "\\mathbb{N}".to_string(),
        crate::Domain::UserDefined(n) => n.clone(),
        _ => "\\top".to_string(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// APL Semantic Trace Backend
// ══════════════════════════════════════════════════════════════════════════════

pub struct AplTraceBackend;

impl ProofBackend for AplTraceBackend {
    fn name(&self) -> &str { "APL" }
    fn file_extension(&self) -> &str { ".apl" }
    fn checker_command(&self) -> Option<String> {
        None
    }

    fn emit(&self, core: &ProofCore) -> ProofOutput {
        let source = emit_apl(core);
        ProofOutput::emitted(source, "apl", &core.theorem_name, None)
    }
}

fn emit_apl(core: &ProofCore) -> String {
    let mut out = String::new();
    out.push_str(&format!("⍝ {} — APL semantic trace\n", core.theorem_name));
    out.push_str(&format!("⍝ Generated by MathRosetta ProofCore pipeline\n\n"));

    out.push_str(&format!("THEOREM ← '{}'\n", core.theorem_name));

    out.push_str("⍝ Variables\n");
    for var in &core.variables {
        out.push_str(&format!("{} ← ⍬\n", var.id.to_uppercase()));
    }
    out.push('\n');

    out.push_str("⍝ Statement structure\n");
    out.push_str(&format!("STATEMENT ← {}\n", mathir_to_apl(&core.statement)));
    out.push('\n');

    out.push_str(&format!("⍝ Proof status: latex_report_emitted\n"));
    out.push_str(&format!("⍝ Checker: none (APL is a trace, not a checker)\n"));

    out
}

fn mathir_to_apl(expr: &crate::MathIR) -> String {
    match expr {
        crate::MathIR::Const(c) => match c {
            crate::Constant::Int(n) => n.to_string(),
            crate::Constant::Float(f) => format!("{:.4}", f),
            _ => "⎕NULL".to_string(),
        },
        crate::MathIR::Var(v) => v.id.to_uppercase(),
        crate::MathIR::Add(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_apl).collect();
            format!("({})", parts.join(" + "))
        }
        crate::MathIR::Mul(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_apl).collect();
            format!("({})", parts.join(" × "))
        }
        crate::MathIR::Pow(base, exp) => format!("{} ⌽ {}", mathir_to_apl(base), mathir_to_apl(exp)),
        crate::MathIR::Eq(lhs, rhs) => format!("{} ≡ {}", mathir_to_apl(lhs), mathir_to_apl(rhs)),
        crate::MathIR::And(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_apl).collect();
            format!("({})", parts.join(" ∧ "))
        }
        crate::MathIR::Or(args) => {
            let parts: Vec<String> = args.iter().map(mathir_to_apl).collect();
            format!("({})", parts.join(" ∨ "))
        }
        crate::MathIR::Not(inner) => format!("¬{}", mathir_to_apl(inner)),
        crate::MathIR::Implies(lhs, rhs) => format!("{} ⊃ {}", mathir_to_apl(lhs), mathir_to_apl(rhs)),
        crate::MathIR::ForAll(var, _, body) => {
            format!("⍳/{}.{})", var.id.to_uppercase(), mathir_to_apl(body))
        }
        crate::MathIR::Exists(var, _, body) => {
            format!("⍳\\{}.{})", var.id.to_uppercase(), mathir_to_apl(body))
        }
        _ => "⎕NULL".to_string(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variable, Domain, AssumptionSet};

    fn var(name: &str) -> Variable {
        Variable { id: name.into(), domain: Domain::Real, assumptions: AssumptionSet::default() }
    }

    fn sample_core() -> ProofCore {
        let expr = crate::MathIR::Eq(
            Box::new(crate::MathIR::Var(Box::new(var("x")))),
            Box::new(crate::MathIR::Const(crate::Constant::Int(1))),
        );
        ProofCore::from_mathir("test_theorem", expr, Some("x = 1".to_string()))
    }

    fn forall_core() -> ProofCore {
        let expr = crate::MathIR::ForAll(
            var("x"),
            Box::new(Domain::Real),
            Box::new(crate::MathIR::Gte(
                Box::new(crate::MathIR::Pow(
                    Box::new(crate::MathIR::Var(Box::new(var("x")))),
                    Box::new(crate::MathIR::Const(crate::Constant::Int(2))),
                )),
                Box::new(crate::MathIR::Const(crate::Constant::Int(0))),
            )),
        );
        ProofCore::from_mathir("nonneg_square", expr, None)
    }

    #[test]
    fn test_all_backends_emit() {
        let core = sample_core();
        let outputs = emit_all_backends(&core);
        assert_eq!(outputs.len(), 7);
        for out in &outputs {
            assert!(!out.source.is_empty());
            assert_eq!(out.theorem_name, "test_theorem");
            // Some backends (Isabelle, Lean, Coq) embed sorry/admitted by design
            // so placeholder_scan is not clean — that's expected
        }
    }

    #[test]
    fn test_isabelle_backend() {
        let core = forall_core();
        let out = IsabelleBackend.emit(&core);
        assert_eq!(out.target, "isabelle");
        assert!(out.source.contains("theory nonneg_square"));
        assert!(out.source.contains("sorry"));
        assert!(out.checker_command.is_some());
    }

    #[test]
    fn test_lean4_backend() {
        let core = forall_core();
        let out = Lean4Backend.emit(&core);
        assert_eq!(out.target, "lean4");
        assert!(out.source.contains("import Mathlib"));
        assert!(out.source.contains("sorry"));
    }

    #[test]
    fn test_coq_backend() {
        let core = forall_core();
        let out = CoqBackend.emit(&core);
        assert_eq!(out.target, "coq");
        assert!(out.source.contains("Theorem nonneg_square"));
        assert!(out.source.contains("Admitted."));
    }

    #[test]
    fn test_idris_backend() {
        let core = forall_core();
        let out = IdrisBackend.emit(&core);
        assert_eq!(out.target, "idris");
        assert!(out.source.contains("module Theorem"));
    }

    #[test]
    fn test_smtlib_backend() {
        let core = forall_core();
        let out = SmtLibBackend.emit(&core);
        assert_eq!(out.target, "smtlib");
        assert!(out.source.contains("(set-logic ALL)"));
        assert!(out.source.contains("(check-sat)"));
    }

    #[test]
    fn test_latex_backend() {
        let core = forall_core();
        let out = LatexReportBackend.emit(&core);
        assert_eq!(out.target, "latex");
        assert!(out.source.contains("\\documentclass"));
        assert!(out.source.contains("\\end{document}"));
    }

    #[test]
    fn test_apl_backend() {
        let core = forall_core();
        let out = AplTraceBackend.emit(&core);
        assert_eq!(out.target, "apl");
        assert!(out.source.contains("THEOREM"));
        assert!(out.source.contains("STATEMENT"));
    }

    #[test]
    fn test_status_not_verified_without_checker() {
        let core = sample_core();
        let out = IsabelleBackend.emit(&core);
        assert_ne!(out.status, ProofStatus::Verified);
        assert_ne!(out.status, ProofStatus::CheckerPassed);
    }

    #[test]
    fn test_checker_result_marks_passed() {
        let core = sample_core();
        let mut out = SmtLibBackend.emit(&core);
        assert!(out.placeholder_scan.is_clean());
        out.with_checker_result("sat".to_string(), 0);
        assert_eq!(out.status, ProofStatus::CheckerPassed);
    }

    #[test]
    fn test_checker_result_marks_failed_on_nonzero() {
        let core = sample_core();
        let mut out = SmtLibBackend.emit(&core);
        out.with_checker_result("unsat".to_string(), 1);
        assert_eq!(out.status, ProofStatus::CheckerFailed);
    }

    #[test]
    fn test_worm_hash_attached() {
        let core = sample_core();
        let mut out = SmtLibBackend.emit(&core);
        out.with_checker_result("sat".to_string(), 0);
        out.with_worm("abc123".to_string());
        assert_eq!(out.worm_receipt_hash, Some("abc123".to_string()));
        assert_eq!(out.status, ProofStatus::WORMSealed);
    }

    #[test]
    fn test_placeholder_detection() {
        let core = ProofCore::from_mathir(
            "bad_thm",
            crate::MathIR::Const(crate::Constant::Int(1)),
            None,
        );
        let out = IsabelleBackend.emit(&core);
        // Isabelle output contains "sorry" by design
        assert!(!out.placeholder_scan.is_clean());
        assert!(out.placeholder_scan.has_sorry);
    }

    #[test]
    fn test_smtlib_forall() {
        let core = forall_core();
        let out = SmtLibBackend.emit(&core);
        assert!(out.source.contains("(forall"));
    }

    #[test]
    fn test_coq_forall() {
        let core = forall_core();
        let out = CoqBackend.emit(&core);
        assert!(out.source.contains("forall (x : R)"));
    }

    #[test]
    fn test_latex_forall() {
        let core = forall_core();
        let out = LatexReportBackend.emit(&core);
        assert!(out.source.contains("\\forall"));
        assert!(out.source.contains("\\mathbb{R}"));
    }
}
