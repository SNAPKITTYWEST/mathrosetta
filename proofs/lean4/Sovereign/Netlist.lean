/-- Netlist well-formedness lemmas -/
namespace Sovereign

lemma wellFormedNetlist_iff (facts : List Fact) :
    wellFormedNetlist facts ↔
    let T := buildTopology facts
    let nonGnd := T.nodes.filter (fun n => !(n.kind == PortKind.gnd))
    let connected : Finset Nat :=
      (T.edges.map (fun e => e.src) ++ T.edges.map (fun e => e.dst)).toFinset
    ∀ n ∈ nonGnd, n.id ∈ connected := by
  rfl

lemma noFloatingPorts (facts : List Fact) (h : wellFormedNetlist facts) :
    let T := buildTopology facts
    ∀ n ∈ T.nodes, n.kind ≠ PortKind.gnd → n.id ∈ (T.edges.map (fun e => e.src) ++ T.edges.map (fun e => e.dst)).toFinset := by
  dsimp only [wellFormedNetlist] at h ⊢
  intro n hn hgn
  have h₁ : n ∈ T.nodes.filter (fun n => !(n.kind == PortKind.gnd)) := by
    simp_all [List.mem_filter, List.mem_map]
    <;> aesop
  have h₂ : n.id ∈ (T.edges.map (fun e => e.src) ++ T.edges.map (fun e => e.dst)).toFinset := h n h₁
  exact h₂

end Sovereign
