use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
use sha2::{Sha256, Digest};
use hex::encode;

/// Represents a transaction between two accounts.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub signature: String,
}

/// Metadata for a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BlockHeader {
    pub parent_hash: String,
    pub number: u64,
    pub state_root: String,
    pub tx_root: String,
    pub timestamp: u64,
}

/// A complete block consisting of a header and a list of transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Computes the SHA-256 hash of the block header, encoded as a hex string.
    pub fn hash(&self) -> String {
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&self.header, config)
            .expect("Failed to serialize block header");
        
        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        encode(hasher.finalize())
    }
}