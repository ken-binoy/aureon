use rocksdb::{DB, Options, Snapshot};
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

    pub fn snapshot(&self) -> Snapshot {
        self.db.snapshot()
    }
}

pub struct SnapshotDb<'a> {
    snapshot: Snapshot<'a>,
}

impl<'a> SnapshotDb<'a> {
    pub fn new(snapshot: Snapshot<'a>) -> Self {
        SnapshotDb { snapshot }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.snapshot.get(key).expect("Snapshot get failed")
    }
}
