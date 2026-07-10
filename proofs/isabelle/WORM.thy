theory WORM
  imports Main
begin

locale Deterministic_Signature =
  fixes sign :: "'key ⇒ 'msg ⇒ 'sig"
  assumes sign_deterministic: "sign k m = sign k m"
begin

lemma sign_deterministic': "sign k m = sign k m"
  by (rule sign_deterministic)

end

record ('key, 'msg, 'sig) worm_receipt =
  tx_id :: string
  topology_hash :: string
  timestamp :: nat
  signature :: 'sig

definition deterministic_receipt ::
  "('key ⇒ 'msg ⇒ 'sig) ⇒ 'key ⇒ string ⇒ string ⇒ nat ⇒
   (('key, 'msg, 'sig) worm_receipt)" where
  "deterministic_receipt sign k tx_id topo_hash ts ≡
     ⦇tx_id = tx_id,
       topology_hash = topo_hash,
       timestamp = ts,
       signature = sign k (tx_id @ topo_hash @ string_of_nat ts)⦇"

lemma (in Deterministic_Signature) worm_receipt_determinism:
  assumes "tx_id = tx_id'"
      and "topo_hash = topo_hash'"
      and "ts = ts'"
  shows "deterministic_receipt sign k tx_id topo_hash ts =
           deterministic_receipt sign k tx_id' topo_hash' ts'"
proof -
  have "deterministic_receipt sign k tx_id topo_hash ts =
        deterministic_receipt sign k tx_id' topo_hash' ts'"
    unfolding deterministic_receipt_def using assms by simp
  thus ?thesis .
qed

end
