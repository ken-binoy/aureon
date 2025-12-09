//! State Compression for Light Clients
//! 
//! Provides efficient compressed state representation for light client sync
//! without requiring full account state storage.

use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Compressed account state for light clients
#[derive(Debug, Clone)]
pub struct CompressedAccount {
    /// Account address
    pub address: String,
    /// Current balance
    pub balance: u64,
    /// Nonce/sequence number
    pub nonce: u64,
    /// Code hash (for smart contract accounts)
    pub code_hash: String,
    /// State root for contract state
    pub storage_root: String,
}

impl CompressedAccount {
    /// Create a new compressed account
    pub fn new(
        address: String,
        balance: u64,
        nonce: u64,
        code_hash: String,
        storage_root: String,
    ) -> Self {
        CompressedAccount {
            address,
            balance,
            nonce,
            code_hash,
            storage_root,
        }
    }

    /// Compute hash of this account's state
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        hasher.update(self.address.as_bytes());
        hasher.update(self.balance.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.code_hash.as_bytes());
        hasher.update(self.storage_root.as_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    /// Estimate the size of this account (in bytes)
    pub fn size_bytes(&self) -> usize {
        // address (32) + balance (8) + nonce (8) + code_hash (64) + storage_root (64) = ~176 bytes
        32 + 8 + 8 + 64 + 64
    }
}

/// Compressed state snapshot for a specific block height
#[derive(Debug, Clone)]
pub struct CompressedStateSnapshot {
    /// Block height this snapshot represents
    pub height: u64,
    /// Block hash
    pub block_hash: String,
    /// State root hash
    pub state_root: String,
    /// Compressed accounts (only active/modified accounts)
    pub accounts: HashMap<String, CompressedAccount>,
    /// Timestamp of snapshot
    pub timestamp: u64,
}

impl CompressedStateSnapshot {
    /// Create a new compressed state snapshot
    pub fn new(
        height: u64,
        block_hash: String,
        state_root: String,
        timestamp: u64,
    ) -> Self {
        CompressedStateSnapshot {
            height,
            block_hash,
            state_root,
            accounts: HashMap::new(),
            timestamp,
        }
    }

    /// Add an account to the snapshot
    pub fn add_account(&mut self, account: CompressedAccount) {
        self.accounts.insert(account.address.clone(), account);
    }

    /// Get an account from the snapshot
    pub fn get_account(&self, address: &str) -> Option<&CompressedAccount> {
        self.accounts.get(address)
    }

    /// Get number of accounts in snapshot
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Compute the state root based on accounts
    pub fn compute_state_root(&self) -> String {
        if self.accounts.is_empty() {
            return String::from("empty_state");
        }

        let mut hasher = Sha256::new();
        
        // Sort accounts by address for deterministic hashing
        let mut addresses: Vec<_> = self.accounts.keys().collect();
        addresses.sort();
        
        for address in addresses {
            if let Some(account) = self.accounts.get(address) {
                hasher.update(account.compute_hash().as_bytes());
            }
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// Verify that the state root matches the accounts
    pub fn verify_state_root(&self) -> bool {
        let computed = self.compute_state_root();
        computed == self.state_root
    }

    /// Get total size of snapshot (in bytes)
    pub fn size_bytes(&self) -> usize {
        let header_size = 8 + 64 + 64 + 8;  // height, block_hash, state_root, timestamp
        let accounts_size: usize = self.accounts.values()
            .map(|a| a.size_bytes())
            .sum();
        
        header_size + accounts_size
    }

    /// Get the compressed size as percentage of full node equivalent
    /// Assumes full node stores all historical state (~1MB per snapshot)
    pub fn compression_ratio(&self) -> f64 {
        let compressed = self.size_bytes() as f64;
        let full_node_estimate = 1_000_000.0;  // ~1MB per snapshot
        
        (compressed / full_node_estimate) * 100.0
    }
}

/// State compression manager for managing multiple snapshots
pub struct StateCompressionManager {
    /// Current active snapshots (keyed by height)
    snapshots: HashMap<u64, CompressedStateSnapshot>,
    /// Keep track of latest compressed height
    latest_height: u64,
}

impl StateCompressionManager {
    /// Create a new state compression manager
    pub fn new() -> Self {
        StateCompressionManager {
            snapshots: HashMap::new(),
            latest_height: 0,
        }
    }

    /// Add a state snapshot
    pub fn add_snapshot(&mut self, snapshot: CompressedStateSnapshot) {
        self.latest_height = self.latest_height.max(snapshot.height);
        self.snapshots.insert(snapshot.height, snapshot);
    }

    /// Get a snapshot by height
    pub fn get_snapshot(&self, height: u64) -> Option<&CompressedStateSnapshot> {
        self.snapshots.get(&height)
    }

    /// Get the latest snapshot
    pub fn get_latest_snapshot(&self) -> Option<&CompressedStateSnapshot> {
        self.snapshots.get(&self.latest_height)
    }

    /// Get snapshots in a height range
    pub fn get_snapshots_in_range(&self, start_height: u64, end_height: u64) -> Vec<&CompressedStateSnapshot> {
        let mut results = Vec::new();
        for height in start_height..=end_height {
            if let Some(snapshot) = self.snapshots.get(&height) {
                results.push(snapshot);
            }
        }
        results
    }

    /// Remove old snapshots to save space (keep only recent N)
    pub fn prune_old_snapshots(&mut self, keep_count: usize) {
        if self.snapshots.len() <= keep_count {
            return;
        }

        let mut heights: Vec<_> = self.snapshots.keys().copied().collect();
        heights.sort();

        let to_remove = heights.len() - keep_count;
        for i in 0..to_remove {
            self.snapshots.remove(&heights[i]);
        }
    }

    /// Get total compression size (all snapshots)
    pub fn total_size_bytes(&self) -> usize {
        self.snapshots.values()
            .map(|s| s.size_bytes())
            .sum()
    }

    /// Get number of snapshots stored
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get average compression ratio
    pub fn average_compression_ratio(&self) -> f64 {
        if self.snapshots.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.snapshots.values()
            .map(|s| s.compression_ratio())
            .sum();
        
        sum / self.snapshots.len() as f64
    }
}

impl Default for StateCompressionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_account_creation() {
        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_123".to_string(),
            "storage_root_456".to_string(),
        );

        assert_eq!(account.address, "0xabcd");
        assert_eq!(account.balance, 1000);
        assert_eq!(account.nonce, 5);
    }

    #[test]
    fn test_compressed_account_hash() {
        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_123".to_string(),
            "storage_root_456".to_string(),
        );

        let hash = account.compute_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);  // SHA256 hex string
    }

    #[test]
    fn test_compressed_account_size() {
        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_123".to_string(),
            "storage_root_456".to_string(),
        );

        let size = account.size_bytes();
        assert_eq!(size, 176);  // Fixed size
    }

    #[test]
    fn test_compressed_state_snapshot_creation() {
        let snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            "state_root_abc".to_string(),
            1234567890,
        );

        assert_eq!(snapshot.height, 100);
        assert_eq!(snapshot.account_count(), 0);
    }

    #[test]
    fn test_compressed_state_snapshot_add_account() {
        let mut snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            "state_root_abc".to_string(),
            1234567890,
        );

        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_123".to_string(),
            "storage_root_456".to_string(),
        );

        snapshot.add_account(account);
        assert_eq!(snapshot.account_count(), 1);
    }

    #[test]
    fn test_compressed_state_snapshot_compute_state_root() {
        let mut snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            "state_root_abc".to_string(),
            1234567890,
        );

        let account1 = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_1".to_string(),
            "storage_root_1".to_string(),
        );

        let account2 = CompressedAccount::new(
            "0xdef0".to_string(),
            2000,
            10,
            "code_hash_2".to_string(),
            "storage_root_2".to_string(),
        );

        snapshot.add_account(account1);
        snapshot.add_account(account2);

        let computed = snapshot.compute_state_root();
        assert!(!computed.is_empty());
        assert_eq!(computed.len(), 64);
    }

    #[test]
    fn test_compressed_state_snapshot_verify_state_root() {
        let mut snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            String::new(),
            1234567890,
        );

        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash_123".to_string(),
            "storage_root_456".to_string(),
        );

        snapshot.add_account(account);
        let correct_root = snapshot.compute_state_root();
        snapshot.state_root = correct_root;

        assert!(snapshot.verify_state_root());
    }

    #[test]
    fn test_state_compression_manager_creation() {
        let manager = StateCompressionManager::new();
        assert_eq!(manager.snapshot_count(), 0);
    }

    #[test]
    fn test_state_compression_manager_add_snapshot() {
        let mut manager = StateCompressionManager::new();
        let snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            "state_root_abc".to_string(),
            1234567890,
        );

        manager.add_snapshot(snapshot);
        assert_eq!(manager.snapshot_count(), 1);
    }

    #[test]
    fn test_state_compression_manager_get_latest() {
        let mut manager = StateCompressionManager::new();
        
        let snapshot1 = CompressedStateSnapshot::new(
            100,
            "block_hash_1".to_string(),
            "state_root_1".to_string(),
            1234567890,
        );
        
        let snapshot2 = CompressedStateSnapshot::new(
            200,
            "block_hash_2".to_string(),
            "state_root_2".to_string(),
            1234567900,
        );

        manager.add_snapshot(snapshot1);
        manager.add_snapshot(snapshot2);

        let latest = manager.get_latest_snapshot().unwrap();
        assert_eq!(latest.height, 200);
    }

    #[test]
    fn test_state_compression_manager_get_range() {
        let mut manager = StateCompressionManager::new();

        for height in 100..=110 {
            let snapshot = CompressedStateSnapshot::new(
                height,
                format!("block_{}", height),
                format!("state_{}", height),
                1234567890 + height,
            );
            manager.add_snapshot(snapshot);
        }

        let range = manager.get_snapshots_in_range(102, 105);
        assert_eq!(range.len(), 4);
    }

    #[test]
    fn test_state_compression_manager_prune() {
        let mut manager = StateCompressionManager::new();

        for height in 100..=110 {
            let snapshot = CompressedStateSnapshot::new(
                height,
                format!("block_{}", height),
                format!("state_{}", height),
                1234567890 + height,
            );
            manager.add_snapshot(snapshot);
        }

        assert_eq!(manager.snapshot_count(), 11);
        manager.prune_old_snapshots(5);
        assert_eq!(manager.snapshot_count(), 5);
    }

    #[test]
    fn test_state_compression_manager_total_size() {
        let mut manager = StateCompressionManager::new();

        for height in 100..=102 {
            let mut snapshot = CompressedStateSnapshot::new(
                height,
                format!("block_{}", height),
                format!("state_{}", height),
                1234567890 + height,
            );
            
            let account = CompressedAccount::new(
                "0xabcd".to_string(),
                1000,
                5,
                "code_hash".to_string(),
                "storage_root".to_string(),
            );
            snapshot.add_account(account);
            
            manager.add_snapshot(snapshot);
        }

        let total = manager.total_size_bytes();
        assert!(total > 0);
    }

    #[test]
    fn test_state_compression_ratio() {
        let mut snapshot = CompressedStateSnapshot::new(
            100,
            "block_hash_123".to_string(),
            "state_root_abc".to_string(),
            1234567890,
        );

        let account = CompressedAccount::new(
            "0xabcd".to_string(),
            1000,
            5,
            "code_hash".to_string(),
            "storage_root".to_string(),
        );
        snapshot.add_account(account);

        let ratio = snapshot.compression_ratio();
        // Should be much less than 1% since we only have one account
        assert!(ratio < 0.1);
    }
}
