use crate::types::{Block, Transaction};
use crate::consensus::ConsensusEngine;
use sha2::{Sha256, Digest};

pub struct PoWConsensus;

impl PoWConsensus {
    pub fn new() -> Self {
        PoWConsensus
    }

    // Hash the block data: transactions, previous_hash, nonce
    fn hash_block(block: &Block) -> String {
        let data = format!("{:?}{:?}{}", block.transactions, block.previous_hash, block.nonce);
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // Proof-of-Work mining: find nonce that produces hash with leading "0000"
    fn mine_block(&self, mut block: Block) -> Block {
        let mut nonce = 0;
        loop {
            block.nonce = nonce;
            let hash = Self::hash_block(&block);
            if hash.starts_with("0000") {
                block.hash = hash;
                return block;
            }
            nonce += 1;
        }
    }
}

impl ConsensusEngine for PoWConsensus {
    fn validate_block(&self, block: &Block) -> bool {
        // Valid if hash starts with "0000"
        block.hash.starts_with("0000")
    }

    fn produce_block(&self, txs: Vec<Transaction>) -> Block {
        let block = Block {
            transactions: txs,
            previous_hash: "GENESIS".to_string(),
            nonce: 0,
            hash: "".to_string(),
        };
        self.mine_block(block)
    }
}