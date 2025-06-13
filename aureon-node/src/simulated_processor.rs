use crate::db::SnapshotDb;
use crate::mpt::MerklePatriciaTrie;

pub struct SimulatedProcessor<'a> {
    snapshot: SnapshotDb<'a>,
    pub trie: &'a mut MerklePatriciaTrie,
}

impl<'a> SimulatedProcessor<'a> {
    pub fn new(snapshot: SnapshotDb<'a>, trie: &'a mut MerklePatriciaTrie) -> Self {
        Self { snapshot, trie }
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