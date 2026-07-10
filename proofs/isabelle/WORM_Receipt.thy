theory WORM_Receipt
  imports WORM Static_Topology
begin

lemma (in Deterministic_Signature) worm_receipt_determinism_theorem:
  assumes "tx_id = tx_id'"
      and "topo_hash = topo_hash'"
      and "ts = ts'"
  shows "deterministic_receipt sign k tx_id topo_hash ts =
           deterministic_receipt sign k tx_id' topo_hash' ts'"
  using assms worm_receipt_determinism by blast

end
