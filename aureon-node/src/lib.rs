use aureon_core::types::{Block, Transaction, BlockHeader};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub state: HashMap<String, u64>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Self::create_genesis_block();
        Blockchain {
            blocks: vec![genesis_block],
            state: HashMap::new(),
        }
    }

    fn create_genesis_block() -> Block {
        let header = BlockHeader {
            parent_hash: "0x0".to_string(),
            number: 0,
            state_root: "0x0".to_string(),
            tx_root: "0x0".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Block {
        let last_block = self.blocks.last().unwrap();
        let header = BlockHeader {
            parent_hash: last_block.hash(),
            number: last_block.header.number + 1,
            state_root: "0xSTUB".to_string(),  // placeholder for now
            tx_root: "0xTXROOT".to_string(),   // placeholder
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let new_block = Block { header, transactions };
        self.blocks.push(new_block.clone());
        new_block
    }
}