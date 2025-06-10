use crate::consensus::ConsensusType;

pub fn load_consensus_type() -> ConsensusType {
    // Future: Load from config.toml or ENV
    ConsensusType::PoW
}