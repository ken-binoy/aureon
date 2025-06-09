use crate::types::{Block, Transaction};
use crate::consensus::ConsensusEngine;
use rand::prelude::*;
use std::collections::HashMap;

pub struct PoSConsensus {
    // Mapping validator address => stake amount
    pub validators: HashMap<String, u64>,
    pub previous_hash: String,
}

impl PoSConsensus {
    pub fn new(validators: HashMap<String, u64>) -> Self {
        PoSConsensus {
            validators,
            previous_hash: "GENESIS".to_string(),
        }
    }

    // Select validator randomly weighted by stake
    fn select_validator(&self) -> Option<String> {
        let total_stake: u64 = self.validators.values().sum();
        if total_stake == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut threshold = rng.gen_range(0..total_stake);

        for (validator, stake) in &self.validators {
            if *stake > threshold {
                return Some(validator.clone());
            }
            threshold -= *stake;
        }

        None
    }
}

impl ConsensusEngine for PoSConsensus {
    fn validate_block(&self, block: &Block) -> bool {
        // Validate block hash is not empty
        if block.hash.is_empty() {
            return false;
        }

        // Extract validator address from hash string "POS-validator-txcount"
        if !block.hash.starts_with("POS-") {
            return false;
        }

        let parts: Vec<&str> = block.hash.split('-').collect();
        if parts.len() < 3 {
            return false;
        }

        let validator = parts[1];

        // Validator must be in current validators list
        self.validators.contains_key(validator)
    }

    fn produce_block(&self, txs: Vec<Transaction>) -> Block {
        // Select validator weighted by stake
        let validator = match self.select_validator() {
            Some(v) => v,
            None => "no-validator".to_string(),
        };

        let block_hash = format!("POS-{}-{}", validator, txs.len());

        // Create block with previous hash tracking (could be improved with chain state)
        Block {
            transactions: txs,
            previous_hash: self.previous_hash.clone(),
            nonce: 0, // PoS doesn't use nonce like PoW
            hash: block_hash,
        }
    }
}