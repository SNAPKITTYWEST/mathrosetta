/-- Conduction soundness from Verilog-AMS parameters -/
namespace Sovereign

def ConductionSound (T : Topology) : Prop :=
  ∀ e ∈ T.edges,
    (e.isBus → e.conductance = busSegmentConductance) ∧
    (¬e.isBus → ∃ (pOn pnOn : Bool), e.conductance = exoCellConductance pOn pnOn)

lemma conductionSound_from_annotated (facts : List Fact) (h : annotatedWithVaParams facts) :
    ConductionSound (buildTopology facts) := by
  dsimp only [annotatedWithVaParams] at h ⊢
  intro e he
  exact h e he

end Sovereign
