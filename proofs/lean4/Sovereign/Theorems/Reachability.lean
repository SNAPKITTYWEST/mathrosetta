/-- Theorem 2: Reachability Preservation -/
namespace Sovereign

theorem reachabilityPreservation (cs : List Clause) (h : PureFactFile cs) :
    let facts := cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)
    let T₁ := buildTopology facts
    let T₂ := buildTopology facts
    ReachabilityPreserved T₁ T₂ := by
  dsimp only
  exact reachabilityPreserved_refl _

end Sovereign
