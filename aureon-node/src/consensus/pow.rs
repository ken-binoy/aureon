use sha2::{Digest, Sha256};
use crate::types::{Block, Transaction};
use crate::consensus::ConsensusEngine;

pub struct PoWConsensus;

impl PoWConsensus {
    pub fn new() -> Self {
        Self
    }

    fn hash_block_content(
        transactions: &Vec<Transaction>,
        previous_hash: &str,
        nonce: u64,
        state_root: &[u8],
    ) -> String {
        let mut hasher = Sha256::new();
        let tx_string: String = transactions.iter().map(|tx| format!("{:?}", tx)).collect();
        hasher.update(tx_string.as_bytes());
        hasher.update(previous_hash.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        hasher.update(state_root);
        let result = hasher.finalize();
        hex::encode(result)
    }
}

impl ConsensusEngine for PoWConsensus {
    fn produce_block(
        &self,
        transactions: Vec<Transaction>,
        pre_state_root: Vec<u8>,
        post_state_root: Vec<u8>,
    ) -> Block {
        let previous_hash = "GENESIS".to_string();
        let mut nonce = 0;

        loop {
            let hash = Self::hash_block_content(&transactions, &previous_hash, nonce, &post_state_root);
            if hash.starts_with("0000") {
                return Block {
                    transactions,
                    previous_hash,
                    nonce,
                    hash,
                    pre_state_root,
                    post_state_root,
                };
            }
            nonce += 1;
        }
    }

    fn validate_block(
        &self,
        block: &Block,
        _pre_state_root: Vec<u8>,
        actual_post_state_root: Vec<u8>,
    ) -> bool {
        if !block.hash.starts_with("0000") {
            return false;
        }

        let expected_hash = Self::hash_block_content(
            &block.transactions,
            &block.previous_hash,
            block.nonce,
            &actual_post_state_root,
        );

        if expected_hash != block.hash {
            return false;
        }

        if block.post_state_root != actual_post_state_root {
            return false;
        }

        true
    }
}