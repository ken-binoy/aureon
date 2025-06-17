use crate::types::{Block, Transaction};

pub fn produce_block(transactions: Vec<Transaction>, pre_root: Vec<u8>, post_root: Vec<u8>) -> Block {
    Block {
        transactions,
        previous_hash: "0000000000000000".to_string(), // dummy
        nonce: 0,
        hash: "deadbeef".to_string(), // dummy
        pre_state_root: pre_root,
        post_state_root: post_root,
    }
}