theory Conduction
  imports Analog Static_Topology
begin

theorem conduction_soundness:
  assumes "pure_fact_file facts"
      and "T = build_topology facts"
      and "annotated_with_va_params facts"
  shows "conduction_sound T"
proof -
  have "conduction_sound T"
    using conduction_sound_from_annotated assms(3) assms(2) by blast
  thus ?thesis .
qed

end
