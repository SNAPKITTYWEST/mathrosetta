/-- Example usage with concrete facts -/
namespace Sovereign

def exampleClauses : List Clause :=
  [ Clause.fact (Fact.node 0 PortKind.sig),
    Clause.fact (Fact.node 1 PortKind.portP),
    Clause.fact (Fact.node 2 PortKind.portPN),
    Clause.fact (Fact.node 3 PortKind.opGate),
    Clause.fact (Fact.node 4 PortKind.gnd),
    Clause.fact (Fact.edge 0 0 3 (exoCellConductance true false) false),
    Clause.fact (Fact.edge 1 1 4 (1 / 50 : Rat) false),
    Clause.fact (Fact.edge 2 0 3 (exoCellConductance false true) false),
    Clause.fact (Fact.edge 3 2 4 (1 / 50 : Rat) false),
    Clause.fact (Fact.edge 4 5 6 (busSegmentConductance : Rat) true),
    Clause.fact (Fact.edge 5 6 5 (busSegmentConductance : Rat) true) ]

theorem example_pure_facts : PureFactFile exampleClauses := by
  intro c hc
  simp [exampleClauses, PureFactFile] at hc ⊢
  <;> rcases c with (_ | _) <;> simp_all (config := {decide := true})

theorem example_well_formed : wellFormedNetlist (exampleClauses.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)) := by
  dsimp [exampleClauses, buildTopology, wellFormedNetlist, Fact.node, Fact.edge, PortKind]
  <;> norm_num [List.filter, List.map, List.filterMap, Finset.mem_insert, Finset.mem_singleton]
  <;> rfl

theorem example_annotated : annotatedWithVaParams (exampleClauses.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)) := by
  dsimp [exampleClauses, buildTopology, annotatedWithVaParams, Fact.node, Fact.edge, PortKind,
    exoCellConductance, busSegmentConductance] at *
  <;> norm_num [List.filterMap, List.map, List.filter]
  <;>
  (try decide) <;>
  (try
    {
      intros
      rcases _ with _ | _ <;> simp_all (config := {decide := true})
      <;>
      (try { use true, false <;> norm_num }) <;>
      (try { use false, true <;> norm_num }) <;>
      (try { use false, false <;> norm_num }) <;>
      (try { use true, true <;> norm_num })
    }) <;>
  (try aesop)

theorem example_all_theorems :
    let facts := exampleClauses.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)
    let T := buildTopology facts
    let R : Receipt Unit := { tx := "tx-001", hash := "hash", timestamp := 1234567890, signature := () }
    AllTheoremsHold T R := by
  dsimp only
  apply sovereignStackCorrect example_pure_facts example_well_formed example_annotated

end Sovereign
