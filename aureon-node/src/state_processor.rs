use crate::db::Db;
use crate::mpt::MerklePatriciaTrie;
use crate::types::Block;  // Add this import if needed
use crate::types::Transaction;

pub struct StateProcessor<'a> {
    db: &'a mut Db,
    trie: &'a mut MerklePatriciaTrie,
}

impl<'a> StateProcessor<'a> {
    pub fn new(db: &'a mut Db, trie: &'a mut MerklePatriciaTrie) -> Self {
        Self { db, trie }
    }

    pub fn apply_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            let from_balance = self.db.get(tx.from.as_bytes());
            let to_balance = self.db.get(tx.to.as_bytes());

            let from_balance = from_balance
                .map(|b| u64::from_le_bytes(b.try_into().unwrap_or_default()))
                .unwrap_or(0);
            let to_balance = to_balance
                .map(|b| u64::from_le_bytes(b.try_into().unwrap_or_default()))
                .unwrap_or(0);

            if from_balance >= tx.amount {
                let new_from = from_balance - tx.amount;
                let new_to = to_balance + tx.amount;

                self.db.put(tx.from.as_bytes(), &new_from.to_le_bytes());
                self.db.put(tx.to.as_bytes(), &new_to.to_le_bytes());

                self.trie.insert(tx.from.clone().into_bytes(), new_from.to_le_bytes().to_vec());
                self.trie.insert(tx.to.clone().into_bytes(), new_to.to_le_bytes().to_vec());

                println!("Applied: {} -> {} [{}]", tx.from, tx.to, tx.amount);
            } else {
                println!("Skipped: {} -> {} [{}] (Insufficient balance)", tx.from, tx.to, tx.amount);
            }
        }
    }

    pub fn apply_block(&mut self, block: &Block) {
        self.apply_transactions(&block.transactions);
    }

    pub fn get_root_hash(&self) -> Vec<u8> {
        self.trie.root_hash()
    }
}