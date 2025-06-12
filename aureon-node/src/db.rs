use rocksdb::{DB, Options};
use std::path::Path;

pub struct Db {
    db: DB,
}

impl Db {
    pub fn open(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, Path::new(path)).expect("Failed to open RocksDB");
        Db { db }
    }

    pub fn put(&self, key: &[u8], value: &[u8]) {
        self.db.put(key, value).expect("DB put failed");
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).expect("DB get failed")
    }

    pub fn delete(&self, key: &[u8]) {
        self.db.delete(key).expect("DB delete failed");
    }
}