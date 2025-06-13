use crate::db::{Db, SnapshotDb};
use crate::mpt::MerklePatriciaTrie;
use crate::types::{Block, Transaction};
use crate::simulated_processor::SimulatedProcessor;

pub struct StateProcessor<'a> {
    pub db: &'a Db,
    pub trie: &'a mut MerklePatriciaTrie,
}

impl<'a> StateProcessor<'a> {
    pub fn new(db: &'a Db, trie: &'a mut MerklePatriciaTrie) -> Self {
        Self { db, trie }
    }

    pub fn apply_block(&mut self, block: &Block) -> Vec<u8> {
        for tx in &block.transactions {
            let from_balance = self.get_balance(&tx.from);
            if from_balance < tx.amount {
                continue;
            }
            let to_balance = self.get_balance(&tx.to);
            self.set_balance(&tx.from, from_balance - tx.amount);
            self.set_balance(&tx.to, to_balance + tx.amount);
        }
        self.trie.root_hash()
    }

    pub fn simulate_block(&self, transactions: &[Transaction]) -> Vec<u8> {
        let snapshot = self.db.snapshot();
        let snapshot_db = SnapshotDb::new(snapshot);
        let mut temp_trie = self.trie.clone();
        let mut temp_processor = SimulatedProcessor::new(snapshot_db, &mut temp_trie);

        for tx in transactions {
            let from_balance = temp_processor.get_balance(&tx.from);
            if from_balance < tx.amount {
                continue;
            }
            let to_balance = temp_processor.get_balance(&tx.to);
            temp_processor.set_balance(&tx.from, from_balance - tx.amount);
            temp_processor.set_balance(&tx.to, to_balance + tx.amount);
        }

        temp_processor.trie.root_hash()
    }

    pub fn get_balance(&self, account: &str) -> u64 {
        if let Some(bytes) = self.db.get(account.as_bytes()) {
            u64::from_le_bytes(bytes.try_into().unwrap_or_default())
        } else {
            0
        }
    }

    pub fn set_balance(&mut self, account: &str, balance: u64) {
        let key = account.as_bytes().to_vec();
        let value = balance.to_le_bytes().to_vec();
        self.db.put(&key, &value);
        self.trie.insert(key, value);
    }
}