/-- Static topology syntax with meaningful pure-fact checking -/
namespace Sovereign

inductive PortKind
| sig
| portP
| portPN
| opGate
| gnd
| busIn
| busMid
| busOut
deriving DecidableEq, Repr, Inhabited

inductive Fact
| node : Nat → PortKind → Fact
| edge : Nat → Nat → Nat → Rat → Bool → Fact
deriving DecidableEq, Repr

structure Rule where
  head : Fact
  body : List Fact
deriving DecidableEq, Repr

inductive Clause
| fact : Fact → Clause
| rule : Rule → Clause
deriving DecidableEq, Repr

def PureFactFile (cs : List Clause) : Prop :=
  ∀ c ∈ cs, match c with
  | Clause.fact _ => True
  | Clause.rule _ => False

theorem pureFactFile_iff (cs : List Clause) :
    PureFactFile cs ↔ ∀ c ∈ cs, ∃ f : Fact, c = Clause.fact f := by
  constructor
  · intro h c hc
    have h₁ := h c hc
    cases c with
    | fact f => exact ⟨f, by simp_all⟩
    | rule _ => exfalso; simp_all
  · intro h c hc
    obtain ⟨f, rfl⟩ := h c hc
    trivial

end Sovereign
