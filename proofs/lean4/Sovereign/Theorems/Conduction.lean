/-- Theorem 4: Conduction Soundness -/
namespace Sovereign

theorem conductionSoundness (cs : List Clause) (h : PureFactFile cs) (hva : annotatedWithVaParams (cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none))) :
    let facts := cs.filterMap (fun c => match c with | Clause.fact f => some f | _ => none)
    let T := buildTopology facts
    ConductionSound T := by
  dsimp only at hva ⊢
  exact conductionSound_from_annotated facts hva

end Sovereign
