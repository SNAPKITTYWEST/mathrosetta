/-- Theorem 5: WORM Receipt Determinism -/
namespace Sovereign

theorem wormReceiptDeterminismTheorem {Key Msg Sig : Type} [DeterministicSigner Key Msg Sig]
    (k : Key) (tx₁ tx₂ hash₁ hash₂ : String) (ts₁ ts₂ : Nat)
    (htx : tx₁ = tx₂) (hhash : hash₁ = hash₂) (hts : ts₁ = ts₂) :
    deterministicReceipt k tx₁ hash₁ ts₁ = deterministicReceipt k tx₂ hash₂ ts₂ := by
  apply wormReceiptDeterminism

end Sovereign
