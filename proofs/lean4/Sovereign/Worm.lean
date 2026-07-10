/-- Abstract deterministic WORM receipts -/
namespace Sovereign

class DeterministicSigner (Key Msg Sig : Type) where
  sign : Key → Msg → Sig
  sign_deterministic : ∀ (k : Key) (m : Msg), sign k m = sign k m

structure Receipt (Sig : Type) where
  tx : String
  hash : String
  timestamp : Nat
  signature : Sig
deriving Repr

def deterministicReceipt {Key Msg Sig : Type} [DeterministicSigner Key Msg Sig]
    (k : Key) (tx hash : String) (ts : Nat) : Receipt Sig :=
  { tx := tx, hash := hash, timestamp := ts,
    signature := DeterministicSigner.sign k (tx ++ hash ++ toString ts) }

lemma wormReceiptDeterminism {Key Msg Sig : Type} [DeterministicSigner Key Msg Sig]
    (k : Key) (tx₁ tx₂ hash₁ hash₂ : String) (ts₁ ts₂ : Nat)
    (htx : tx₁ = tx₂) (hhash : hash₁ = hash₂) (hts : ts₁ = ts₂) :
    deterministicReceipt k tx₁ hash₁ ts₁ = deterministicReceipt k tx₂ hash₂ ts₂ := by
  simp_all [deterministicReceipt]
  <;> congr 1 <;> simp_all [String.append_assoc]
  <;> aesop

end Sovereign
