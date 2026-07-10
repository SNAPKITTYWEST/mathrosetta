use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::proof::ProofBundle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormReceipt {
    pub id: String,
    pub input: String,
    pub mathir_hash: String,
    pub normalized_hash: String,
    pub emitters: Vec<String>,
    pub status: String,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormChain {
    pub receipts: Vec<WormReceipt>,
}

impl WormChain {
    pub fn new() -> Self {
        Self { receipts: Vec::new() }
    }

    pub fn genesis() -> Self {
        let mut genesis_receipt = WormReceipt {
            id: "genesis".to_string(),
            input: String::new(),
            mathir_hash: "0".to_string(),
            normalized_hash: "0".to_string(),
            emitters: Vec::new(),
            status: "genesis".to_string(),
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp: "1970-01-01T00:00:00Z".to_string(),
        };
        genesis_receipt.hash = genesis_receipt.compute_hash();
        Self { receipts: vec![genesis_receipt] }
    }

    pub fn append(&mut self, bundle: &ProofBundle) -> WormReceipt {
        let prev_hash = self.last_hash();
        let receipt = WormReceipt::from_bundle(bundle, &prev_hash);
        self.receipts.push(receipt.clone());
        receipt
    }

    pub fn last_hash(&self) -> String {
        self.receipts.last()
            .map(|r| r.hash.clone())
            .unwrap_or_else(|| "0".to_string())
    }

    pub fn verify_chain(&self) -> bool {
        if self.receipts.is_empty() {
            return true;
        }

        for i in 1..self.receipts.len() {
            if self.receipts[i].prev_hash != self.receipts[i - 1].hash {
                return false;
            }
        }

        for receipt in &self.receipts {
            let computed = receipt.compute_hash();
            if computed != receipt.hash {
                return false;
            }
        }

        true
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }
}

impl Default for WormChain {
    fn default() -> Self {
        Self::new()
    }
}

impl WormReceipt {
    pub fn from_bundle(bundle: &ProofBundle, prev_hash: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mathir_json = serde_json::to_string(&bundle.mathir).unwrap_or_default();
        let normalized_json = serde_json::to_string(&bundle.normalized).unwrap_or_default();

        let mathir_hash = hash_string(&mathir_json);
        let normalized_hash = hash_string(&normalized_json);

        let emitters: Vec<String> = bundle.targets.iter()
            .map(|t| format!("{:?}", t.backend).to_lowercase())
            .collect();

        let id = format!("proof_{}", now);

        let mut receipt = WormReceipt {
            id,
            input: bundle.input_latex.clone(),
            mathir_hash,
            normalized_hash,
            emitters,
            status: "emitted_pending_proof".to_string(),
            prev_hash: prev_hash.to_string(),
            hash: String::new(),
            timestamp: now.to_string(),
        };

        receipt.hash = receipt.compute_hash();
        receipt
    }

    pub fn compute_hash(&self) -> String {
        let payload = serde_json::json!({
            "id": self.id,
            "input": self.input,
            "mathir_hash": self.mathir_hash,
            "normalized_hash": self.normalized_hash,
            "emitters": self.emitters,
            "status": self.status,
            "prev_hash": self.prev_hash,
            "timestamp": self.timestamp,
        });
        let stable_json = serde_json::to_string(&payload).unwrap_or_default();
        let combined = format!("{}{}", self.prev_hash, stable_json);
        hash_string(&combined)
    }
}

fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MathIR, Variable, Domain, AssumptionSet};
    use crate::proof::bundle::ProofBundle;

    fn make_test_bundle() -> ProofBundle {
        let mathir = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        ProofBundle::new("x = 1", mathir.clone(), mathir, "test_theorem")
    }

    #[test]
    fn test_worm_genesis() {
        let chain = WormChain::genesis();
        assert_eq!(chain.len(), 1);
        assert!(chain.receipts[0].id == "genesis");
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_worm_append() {
        let mut chain = WormChain::genesis();
        let bundle = make_test_bundle();
        let receipt = chain.append(&bundle);
        assert_eq!(chain.len(), 2);
        assert!(receipt.prev_hash == chain.receipts[0].hash);
        assert!(receipt.hash != "0");
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_worm_hash_changes_with_input() {
        let mut chain1 = WormChain::genesis();
        let mut chain2 = WormChain::genesis();

        let mathir1 = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(1))),
        );
        let mathir2 = MathIR::Eq(
            Box::new(MathIR::Var(Box::new(Variable { id: "y".into(), domain: Domain::Real, assumptions: AssumptionSet::default() }))),
            Box::new(MathIR::Const(crate::Constant::Int(2))),
        );

        let bundle1 = ProofBundle::new("x = 1", mathir1.clone(), mathir1, "t1");
        let bundle2 = ProofBundle::new("y = 2", mathir2.clone(), mathir2, "t2");

        let receipt1 = chain1.append(&bundle1);
        let receipt2 = chain2.append(&bundle2);

        assert_ne!(receipt1.hash, receipt2.hash);
    }

    #[test]
    fn test_worm_chain_verification() {
        let mut chain = WormChain::genesis();
        let bundle = make_test_bundle();
        chain.append(&bundle);
        assert!(chain.verify_chain());
    }
}
