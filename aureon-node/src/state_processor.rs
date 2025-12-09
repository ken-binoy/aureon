use crate::db::{Db, SnapshotDb};
use crate::mpt::MerklePatriciaTrie;
use crate::types::{Block, Transaction, TransactionPayload};
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
            self.apply_transaction(tx);
        }
        self.trie.root_hash()
    }

    /// Apply a single transaction to state
    pub fn apply_transaction(&mut self, tx: &Transaction) {
        match &tx.payload {
            TransactionPayload::Transfer { to, amount } => {
                let from_balance = self.get_balance(&tx.from);
                if from_balance >= *amount {
                    let to_balance = self.get_balance(to);
                    self.set_balance(&tx.from, from_balance - *amount);
                    self.set_balance(to, to_balance + *amount);
                }
            }
            TransactionPayload::ContractDeploy { code: _, gas_limit: _ } => {
                // Contract deployment will be handled by upper layer
                // This is a placeholder for now
            }
            TransactionPayload::ContractCall {
                contract_address: _,
                function: _,
                args: _,
                gas_limit: _,
            } => {
                // Contract execution will be handled by upper layer
                // This is a placeholder for now
            }
            TransactionPayload::Stake { amount } => {
                let balance = self.get_balance(&tx.from);
                if balance >= *amount {
                    // In a full implementation, this would transfer to staking pool
                    self.set_balance(&tx.from, balance - *amount);
                }
            }
            TransactionPayload::Unstake { amount } => {
                // In a full implementation, this would check staked amount
                let balance = self.get_balance(&tx.from);
                self.set_balance(&tx.from, balance + *amount);
            }
        }
    }

    pub fn simulate_block(&self, transactions: &[Transaction]) -> Vec<u8> {
        let snapshot = self.db.snapshot();
        let snapshot_db = SnapshotDb::new(snapshot);
        let mut temp_trie = self.trie.clone();
        let mut temp_processor = SimulatedProcessor::new(snapshot_db, &mut temp_trie);

        for tx in transactions {
            temp_processor.apply_transaction(tx);
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