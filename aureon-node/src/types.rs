use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub enum TransactionPayload {
    /// Simple transfer between accounts
    Transfer { 
        to: String, 
        amount: u64 
    },
    /// Deploy a smart contract (WASM bytecode)
    ContractDeploy {
        code: Vec<u8>,  // WASM bytecode
        gas_limit: u64,
    },
    /// Call an existing contract function
    ContractCall {
        contract_address: String,
        function: String,
        args: Vec<Vec<u8>>,  // serialized arguments
        gas_limit: u64,
    },
    /// Stake tokens for PoS participation
    Stake {
        amount: u64,
    },
    /// Unstake tokens
    Unstake {
        amount: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct Transaction {
    pub from: String,
    pub nonce: u64,
    pub gas_price: u64,
    pub payload: TransactionPayload,
    pub signature: Vec<u8>,  // Ed25519 signature (64 bytes)
    pub public_key: Vec<u8>,  // Ed25519 public key (32 bytes)
}

impl Transaction {
    /// Helper to create a simple transfer (backward compat)
    pub fn transfer(from: String, to: String, amount: u64) -> Self {
        Self {
            from,
            nonce: 0,
            gas_price: 1,
            payload: TransactionPayload::Transfer { to, amount },
            signature: vec![],
            public_key: vec![],
        }
    }

    /// Helper to create a contract deployment
    pub fn deploy_contract(from: String, code: Vec<u8>, gas_limit: u64) -> Self {
        Self {
            from,
            nonce: 0,
            gas_price: 1,
            payload: TransactionPayload::ContractDeploy { code, gas_limit },
            signature: vec![],
            public_key: vec![],
        }
    }

    /// Helper to create a contract call
    pub fn call_contract(
        from: String,
        contract_address: String,
        function: String,
        args: Vec<Vec<u8>>,
        gas_limit: u64,
    ) -> Self {
        Self {
            from,
            nonce: 0,
            gas_price: 1,
            payload: TransactionPayload::ContractCall {
                contract_address,
                function,
                args,
                gas_limit,
            },
            signature: vec![],
            public_key: vec![],
        }
    }

    /// Helper to create a stake transaction
    pub fn stake(from: String, amount: u64) -> Self {
        Self {
            from,
            nonce: 0,
            gas_price: 1,
            payload: TransactionPayload::Stake { amount },
            signature: vec![],
            public_key: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
    pub pre_state_root: Vec<u8>,
    pub post_state_root: Vec<u8>,
}

/// Represents an account in shard state
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage: std::collections::HashMap<String, Vec<u8>>,
}