theory Static_Topology
  imports Graph
begin

theorem topology_preservation:
  assumes "pure_fact_file facts"
      and "T = build_topology facts"
  shows "topology_preserved T T"
proof -
  have "topology_preserved T T" by (rule topology_preserved_refl)
  thus ?thesis .
qed

theorem reachability_preservation:
  assumes "pure_fact_file facts"
      and "T1 = build_topology facts"
      and "T2 = build_topology facts"
  shows "reachability_preserved T1 T2"
proof -
  have "T1 = T2" using assms by simp
  hence "reachability_preserved T1 T2"
    by (simp add: reachability_preserved_refl)
  thus ?thesis .
qed

definition no_floating_ports :: "topology ⇒ bool" where
  "no_floating_ports T ≡
     (∀n ∈ set (nodes T).
        kind n ≠ Gnd ⟶
        (nid n ∈ set (map src (edges T)) ∪ set (map dst (edges T))))"

theorem no_floating_ports:
  assumes "pure_fact_file facts"
      and "T = build_topology facts"
      and "well_formed_netlist facts"
  shows "no_floating_ports T"
proof -
  have "well_formed_netlist facts" by fact
  have "T = build_topology facts" by fact
  have "∀n ∈ set (nodes T).
          kind n ≠ Gnd ⟶
          (nid n ∈ set (map src (edges T)) ∪ set (map dst (edges T)))"
  proof -
    have wf: "well_formed_netlist facts" by fact
    have hT: "T = build_topology facts" by fact
    have "∀n ∈ set (nodes T).
            kind n ≠ Gnd ⟶
            (nid n ∈ set (map src (edges T)) ∪ set (map dst (edges T)))"
      unfolding hT well_formed_netlist_def
      by (auto split: fact.split node.split edge.split
                simp: build_topology_def)
    thus ?thesis .
  qed
  thus ?thesis unfolding no_floating_ports_def by blast
qed

end
