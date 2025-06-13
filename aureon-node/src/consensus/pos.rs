use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::types::{Block, Transaction};
use crate::consensus::ConsensusEngine;

pub struct PoSConsensus {
    validators: HashMap<String, u64>,
}

impl PoSConsensus {
    pub fn new(validators: HashMap<String, u64>) -> Self {
        Self { validators }
    }

    fn select_validator(&self) -> String {
        self.validators
            .iter()
            .max_by_key(|&(_, stake)| stake)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "DefaultValidator".to_string())
    }

    fn hash_block_content(
        transactions: &Vec<Transaction>,
        previous_hash: &str,
        validator: &str,
        state_root: &[u8],
    ) -> String {
        let mut hasher = Sha256::new();
        let tx_string: String = transactions.iter().map(|tx| format!("{:?}", tx)).collect();
        hasher.update(tx_string.as_bytes());
        hasher.update(previous_hash.as_bytes());
        hasher.update(validator.as_bytes());
        hasher.update(state_root);
        let result = hasher.finalize();
        hex::encode(result)
    }
}

impl ConsensusEngine for PoSConsensus {
    fn produce_block(
        &self,
        transactions: Vec<Transaction>,
        pre_state_root: Vec<u8>,
        post_state_root: Vec<u8>,
    ) -> Block {
        let previous_hash = "GENESIS".to_string();
        let validator = self.select_validator();

        let hash = Self::hash_block_content(
            &transactions,
            &previous_hash,
            &validator,
            &post_state_root,
        );

        Block {
            transactions,
            previous_hash,
            nonce: 0,
            hash,
            pre_state_root,
            post_state_root,
        }
    }

    fn validate_block(
        &self,
        block: &Block,
        _pre_state_root: Vec<u8>,
        actual_post_state_root: Vec<u8>,
    ) -> bool {
        let validator = self.select_validator();

        let expected_hash = Self::hash_block_content(
            &block.transactions,
            &block.previous_hash,
            &validator,
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