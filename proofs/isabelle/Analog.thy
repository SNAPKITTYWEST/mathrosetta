theory Analog
  imports Topology
begin

definition conduction_sound :: "topology ⇒ bool" where
  "conduction_sound T ≡
     (∀e ∈ set (edges T).
        (is_bus e ⟶ conductance e = (1 / (0.1 * 500))) ∧
        (¬ is_bus e ⟶
           (∃p_on pn_on. conductance e =
              (if p_on then 1/50 else 1/1e12) + (if pn_on then 1/50 else 1/1e12))))"

lemma conduction_sound_from_annotated:
  assumes "annotated_with_va_params facts"
      and "T = build_topology facts"
  shows "conduction_sound T"
proof -
  have "∀e ∈ set (edges T).
          (is_bus e ⟶ conductance e = (1 / (0.1 * 500))) ∧
          (¬ is_bus e ⟶ (∃p_on pn_on. conductance e =
             (if p_on then 1/50 else 1/1e12) + (if pn_on then 1/50 else 1/1e12)))"
  proof -
    have h: "annotated_with_va_params facts" by fact
    have hT: "T = build_topology facts" by fact
    have "∀e ∈ set (edges T).
            (is_bus e ⟶ conductance e = (1 / (0.1 * 500))) ∧
            (¬ is_bus e ⟶ (∃p_on pn_on. conductance e =
               (if p_on then 1/50 else 1/1e12) + (if pn_on then 1/50 else 1/1e12)))"
      unfolding hT annotated_with_va_params_def
      by (auto split: fact.split edge.split node.split
                simp: exo_cell_conductance_def bus_segment_conductance_def)
    thus ?thesis .
  qed
  thus ?thesis unfolding conduction_sound_def by blast
qed

end
