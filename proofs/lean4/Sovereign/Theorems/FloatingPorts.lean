/-- Theorem 3: No Floating Ports -/
namespace Sovereign

theorem noFloatingPorts (cs : List Clause) (h : PureFactFile cs) (hwf : wellFormedNetlist (cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none))) :
    let facts := cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)
    let T := buildTopology facts
    ∀ n ∈ T.nodes, n.kind ≠ PortKind.gnd → n.id ∈ (T.edges.map (fun e => e.src) ++ T.edges.map (fun e => e.dst)).toFinset := by
  dsimp only at hwf ⊢
  exact noFloatingPorts facts hwf

end Sovereign
