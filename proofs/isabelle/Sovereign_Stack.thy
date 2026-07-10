theory Sovereign_Stack
  imports Conduction WORM_Receipt
begin

definition all_theorems_hold :: "topology ⇒ ('key, 'msg, 'sig) worm_receipt ⇒ bool" where
  "all_theorems_hold T R ≡
     topology_preserved T T ∧
     reachability_preserved T T ∧
     no_floating_ports T ∧
     conduction_sound T ∧
     (∃sign k tx_id topo_hash ts.
        R = deterministic_receipt sign k tx_id topo_hash ts)"

theorem sovereign_stack_correct:
  assumes "pure_fact_file facts"
      and "T = build_topology facts"
      and "well_formed_netlist facts"
      and "annotated_with_va_params facts"
      and "R = deterministic_receipt sign k ""tx-001"" (""hash"" :: string) 1234567890"
  shows "all_theorems_hold T R"
proof -
  have t1: "topology_preserved T T"
    using topology_preservation assms(1) assms(2) by blast
  have t2: "reachability_preserved T T"
    using reachability_preservation assms(1) assms(2) assms(2) by blast
  have t3: "no_floating_ports T"
    using no_floating_ports assms(1) assms(2) assms(3) by blast
  have t4: "conduction_sound T"
    using conduction_soundness assms(1) assms(2) assms(4) by blast
  have t5: "∃sign' k' tx_id topo_hash ts. R = deterministic_receipt sign' k' tx_id topo_hash ts"
    using assms(5) by blast
  thus ?thesis unfolding all_theorems_hold_def by blast
qed

end
