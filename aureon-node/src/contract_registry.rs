use std::collections::HashMap;
use sha2::{Digest, Sha256};

/// Contract registry stores deployed contracts and their metadata
pub struct ContractRegistry {
    /// contract_address -> (code_hash, code_bytes)
    contracts: HashMap<String, (String, Vec<u8>)>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    /// Deploy a contract and return its address (hash of code)
    pub fn deploy(&mut self, code: Vec<u8>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&code);
        let hash = hex::encode(hasher.finalize());
        
        self.contracts.insert(hash.clone(), (hash.clone(), code));
        hash
    }

    /// Get contract code by address
    pub fn get_contract(&self, address: &str) -> Option<Vec<u8>> {
        self.contracts.get(address).map(|(_, code)| code.clone())
    }

    /// Check if contract exists
    pub fn contract_exists(&self, address: &str) -> bool {
        self.contracts.contains_key(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_and_get() {
        let mut registry = ContractRegistry::new();
        let code = vec![1, 2, 3];
        let addr = registry.deploy(code.clone());
        
        assert!(registry.contract_exists(&addr));
        assert_eq!(registry.get_contract(&addr).unwrap(), code);
    }
}
