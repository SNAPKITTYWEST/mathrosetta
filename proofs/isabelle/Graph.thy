theory Graph
  imports Topology
begin

inductive reachable :: "topology ⇒ nat ⇒ nat ⇒ bool" where
  base: "⟦∃e ∈ set (edges T). src e = s ∧ dst e = d⟧ ⟹ reachable T s d"
| trans: "⟦reachable T s m; reachable T m d⟧ ⟹ reachable T s d"

definition reachability_preserved :: "topology ⇒ topology ⇒ bool" where
  "reachability_preserved T1 T2 ⟷
     (∀s d. reachable T1 s d ⟷ reachable T2 s d)"

lemma reachability_preserved_refl: "reachability_preserved T T"
  unfolding reachability_preserved_def by blast

definition topology_preserved :: "topology ⇒ topology ⇒ bool" where
  "topology_preserved T1 T2 ⟷
     (set (nodes T1) = set (nodes T2)) ∧
     (set (edges T1) = set (edges T2))"

lemma topology_preserved_refl: "topology_preserved T T"
  unfolding topology_preserved_def by simp

end
