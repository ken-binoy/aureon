use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub chain_id: String,
    pub timestamp: u64,
    pub initial_validators: Vec<String>,
    pub initial_balances: Vec<(String, u64)>,
    pub nonce: u64,
}