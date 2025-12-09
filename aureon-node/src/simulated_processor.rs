use crate::db::SnapshotDb;
use crate::mpt::MerklePatriciaTrie;
use crate::types::{Transaction, TransactionPayload};

pub struct SimulatedProcessor<'a> {
    snapshot: SnapshotDb<'a>,
    pub trie: &'a mut MerklePatriciaTrie,
}

impl<'a> SimulatedProcessor<'a> {
    pub fn new(snapshot: SnapshotDb<'a>, trie: &'a mut MerklePatriciaTrie) -> Self {
        Self { snapshot, trie }
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
                // Placeholder
            }
            TransactionPayload::ContractCall {
                contract_address: _,
                function: _,
                args: _,
                gas_limit: _,
            } => {
                // Placeholder
            }
            TransactionPayload::Stake { amount } => {
                let balance = self.get_balance(&tx.from);
                if balance >= *amount {
                    self.set_balance(&tx.from, balance - *amount);
                }
            }
            TransactionPayload::Unstake { amount } => {
                let balance = self.get_balance(&tx.from);
                self.set_balance(&tx.from, balance + *amount);
            }
        }
    }

    pub fn get_balance(&self, account: &str) -> u64 {
        if let Some(bytes) = self.snapshot.get(account.as_bytes()) {
            u64::from_le_bytes(bytes.try_into().unwrap_or_default())
        } else {
            0
        }
    }

    pub fn set_balance(&mut self, account: &str, balance: u64) {
        let key = account.as_bytes().to_vec();
        let value = balance.to_le_bytes().to_vec();
        self.trie.insert(key, value);
    }
}