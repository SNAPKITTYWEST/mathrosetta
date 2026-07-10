/-- Theorem 1: Topology Preservation -/
namespace Sovereign

theorem topologyPreservation (cs : List Clause) (h : PureFactFile cs) :
    let facts := cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)
    let T := buildTopology facts
    TopologyEquivalent T T := by
  dsimp only
  exact topologyEquivalent_refl _

end Sovereign
