pub mod pow;
pub mod pos;
use std::collections::HashMap;

use crate::consensus::{pow::PoWConsensus, pos::PoSConsensus};
use crate::types::{Block, Transaction};

pub trait ConsensusEngine {
    fn produce_block(&self, transactions: Vec<Transaction>, pre_state_root: Vec<u8>, post_state_root: Vec<u8>) -> Block;

    fn validate_block(
        &self,
        block: &Block,
        pre_state_root: Vec<u8>,
        actual_post_state_root: Vec<u8>,
    ) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub enum ConsensusType {
    PoW,
    PoS,
    PoA,
}

pub fn get_engine(consensus_type: ConsensusType) -> Box<dyn ConsensusEngine> {
    match consensus_type {
        ConsensusType::PoW => Box::new(PoWConsensus::new()),
        ConsensusType::PoS => {
            let mut validators = HashMap::new();
            validators.insert("Alice".to_string(), 100);
            validators.insert("Bob".to_string(), 200);
            Box::new(PoSConsensus::new(validators))
        }
        ConsensusType::PoA => {
            // PoA uses PoS engine with authority-based validator set
            // In production, validators would be loaded from config
            let mut validators = HashMap::new();
            validators.insert("alice".to_string(), 100);
            validators.insert("bob".to_string(), 100);
            validators.insert("charlie".to_string(), 100);
            Box::new(PoSConsensus::new(validators))
        }
    }
}