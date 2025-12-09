use sha2::{Sha256, Digest};
use crate::shard_coordinator::ShardId;
use crate::shard_manager::ShardLedger;
use std::collections::HashMap;

/// Merkle proof node in a merkle tree for shard state validation
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleProofNode {
    pub hash: String,
    pub is_left: bool,
}

/// Merkle proof path from leaf to root
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf_hash: String,
    pub path: Vec<MerkleProofNode>,
    pub root_hash: String,
}

impl MerkleProof {
    /// Verify a merkle proof is valid
    pub fn verify(&self) -> bool {
        let mut current = self.leaf_hash.clone();

        for node in &self.path {
            current = if node.is_left {
                hash_pair(&node.hash, &current)
            } else {
                hash_pair(&current, &node.hash)
            };
        }

        current == self.root_hash
    }
}

/// Hash two values together (for merkle tree)
fn hash_pair(left: &str, right: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Hash a single value (for merkle tree leaves)
fn hash_value(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Shard state snapshot for synchronization
#[derive(Debug, Clone, PartialEq)]
pub struct ShardStateSnapshot {
    pub shard_id: ShardId,
    pub block_number: u64,
    pub state_root: String,
    pub account_count: usize,
    pub accounts: Vec<String>,
}

impl ShardStateSnapshot {
    /// Create a snapshot from a shard ledger
    pub fn from_ledger(
        shard_id: ShardId,
        block_number: u64,
        ledger: &ShardLedger,
    ) -> Self {
        let accounts = ledger
            .accounts
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        ShardStateSnapshot {
            shard_id,
            block_number,
            state_root: ledger.state_root.clone(),
            account_count: ledger.account_count(),
            accounts,
        }
    }

    /// Validate snapshot consistency
    pub fn validate(&self) -> bool {
        self.account_count == self.accounts.len() && !self.state_root.is_empty()
    }
}

/// Shard synchronization manager
/// Coordinates state synchronization across shard replicas using merkle proofs
#[derive(Debug)]
pub struct ShardSync {
    /// Current sync status per shard
    sync_status: HashMap<ShardId, SyncStatus>,
    /// Recent snapshots for quick access
    snapshots: HashMap<ShardId, ShardStateSnapshot>,
}

/// Status of shard synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Shard is fully synchronized
    Synced,
    /// Shard is currently syncing
    Syncing,
    /// Shard is out of sync and needs full resync
    OutOfSync,
}

impl ShardSync {
    /// Create a new shard sync manager
    pub fn new() -> Self {
        ShardSync {
            sync_status: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    /// Update sync status for a shard
    pub fn set_status(&mut self, shard: ShardId, status: SyncStatus) {
        self.sync_status.insert(shard, status);
    }

    /// Get current sync status for a shard
    pub fn get_status(&self, shard: ShardId) -> Option<SyncStatus> {
        self.sync_status.get(&shard).copied()
    }

    /// Check if a shard is fully synchronized
    pub fn is_synced(&self, shard: ShardId) -> bool {
        self.get_status(shard) == Some(SyncStatus::Synced)
    }

    /// Store a shard state snapshot
    pub fn store_snapshot(&mut self, snapshot: ShardStateSnapshot) {
        self.snapshots.insert(snapshot.shard_id, snapshot);
    }

    /// Get a stored snapshot for a shard
    pub fn get_snapshot(&self, shard: ShardId) -> Option<&ShardStateSnapshot> {
        self.snapshots.get(&shard)
    }

    /// Validate a snapshot matches expected state root
    pub fn validate_snapshot(
        &self,
        shard: ShardId,
        expected_root: &str,
    ) -> bool {
        if let Some(snapshot) = self.get_snapshot(shard) {
            snapshot.state_root == expected_root && snapshot.validate()
        } else {
            false
        }
    }

    /// Generate a merkle proof for an account in a shard
    pub fn generate_merkle_proof(
        &self,
        shard: ShardId,
        account_address: &str,
    ) -> Option<MerkleProof> {
        let snapshot = self.get_snapshot(shard)?;

        // Find account in sorted list
        let mut sorted_accounts = snapshot.accounts.clone();
        sorted_accounts.sort();

        let account_index = sorted_accounts
            .iter()
            .position(|a| a == account_address)?;

        // Build merkle proof path
        let leaf_hash = hash_value(account_address);
        let path = self.build_merkle_path(&sorted_accounts, account_index);

        Some(MerkleProof {
            leaf_hash,
            path,
            root_hash: snapshot.state_root.clone(),
        })
    }

    /// Build merkle proof path from leaf index to root
    fn build_merkle_path(&self, sorted_accounts: &[String], leaf_index: usize) -> Vec<MerkleProofNode> {
        let mut path = Vec::new();
        let mut current_level = sorted_accounts
            .iter()
            .map(|a| hash_value(a))
            .collect::<Vec<_>>();

        let mut index = leaf_index;

        while current_level.len() > 1 {
            let is_left = index % 2 == 0;
            let sibling_index = if is_left { index + 1 } else { index - 1 };

            if sibling_index < current_level.len() {
                path.push(MerkleProofNode {
                    hash: current_level[sibling_index].clone(),
                    is_left: !is_left,
                });
            }

            // Build next level
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    next_level.push(hash_pair(&current_level[i], &current_level[i + 1]));
                } else {
                    next_level.push(current_level[i].clone());
                }
            }

            current_level = next_level;
            index /= 2;
        }

        path
    }

    /// Get count of synchronized shards
    pub fn synced_count(&self) -> usize {
        self.sync_status
            .values()
            .filter(|&&status| status == SyncStatus::Synced)
            .count()
    }

    /// Get count of syncing shards
    pub fn syncing_count(&self) -> usize {
        self.sync_status
            .values()
            .filter(|&&status| status == SyncStatus::Syncing)
            .count()
    }

    /// Get count of out-of-sync shards
    pub fn out_of_sync_count(&self) -> usize {
        self.sync_status
            .values()
            .filter(|&&status| status == SyncStatus::OutOfSync)
            .count()
    }

    /// Get all synced shards
    pub fn get_synced_shards(&self) -> Vec<ShardId> {
        self.sync_status
            .iter()
            .filter(|&(_, status)| *status == SyncStatus::Synced)
            .map(|(&shard, _)| shard)
            .collect()
    }

    /// Get all out-of-sync shards
    pub fn get_out_of_sync_shards(&self) -> Vec<ShardId> {
        self.sync_status
            .iter()
            .filter(|&(_, status)| *status == SyncStatus::OutOfSync)
            .map(|(&shard, _)| shard)
            .collect()
    }
}

impl Default for ShardSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_node_creation() {
        let node = MerkleProofNode {
            hash: "abc123".to_string(),
            is_left: true,
        };
        assert_eq!(node.hash, "abc123");
        assert!(node.is_left);
    }

    #[test]
    fn test_hash_pair() {
        let hash1 = hash_pair("left", "right");
        let hash2 = hash_pair("left", "right");
        assert_eq!(hash1, hash2); // Deterministic
    }

    #[test]
    fn test_hash_value() {
        let hash1 = hash_value("test");
        let hash2 = hash_value("test");
        assert_eq!(hash1, hash2); // Deterministic
        assert_ne!(hash1, hash_value("other"));
    }

    #[test]
    fn test_shard_state_snapshot_creation() {
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "root123".to_string(),
            account_count: 5,
            accounts: vec!["alice@aureon".to_string(), "bob@aureon".to_string()],
        };

        assert_eq!(snapshot.shard_id, ShardId(0));
        assert_eq!(snapshot.block_number, 100);
    }

    #[test]
    fn test_shard_state_snapshot_validate() {
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "root123".to_string(),
            account_count: 2,
            accounts: vec!["alice@aureon".to_string(), "bob@aureon".to_string()],
        };

        assert!(snapshot.validate());
    }

    #[test]
    fn test_shard_state_snapshot_invalid() {
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "".to_string(), // Empty root
            account_count: 2,
            accounts: vec!["alice@aureon".to_string()], // Mismatch
        };

        assert!(!snapshot.validate());
    }

    #[test]
    fn test_shard_sync_creation() {
        let sync = ShardSync::new();
        assert_eq!(sync.synced_count(), 0);
    }

    #[test]
    fn test_shard_sync_set_status() {
        let mut sync = ShardSync::new();
        sync.set_status(ShardId(0), SyncStatus::Synced);
        assert_eq!(sync.get_status(ShardId(0)), Some(SyncStatus::Synced));
    }

    #[test]
    fn test_shard_sync_is_synced() {
        let mut sync = ShardSync::new();
        sync.set_status(ShardId(0), SyncStatus::Synced);
        sync.set_status(ShardId(1), SyncStatus::Syncing);

        assert!(sync.is_synced(ShardId(0)));
        assert!(!sync.is_synced(ShardId(1)));
    }

    #[test]
    fn test_shard_sync_store_snapshot() {
        let mut sync = ShardSync::new();
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "root123".to_string(),
            account_count: 2,
            accounts: vec!["alice@aureon".to_string(), "bob@aureon".to_string()],
        };

        sync.store_snapshot(snapshot.clone());
        assert_eq!(sync.get_snapshot(ShardId(0)), Some(&snapshot));
    }

    #[test]
    fn test_shard_sync_validate_snapshot() {
        let mut sync = ShardSync::new();
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "root123".to_string(),
            account_count: 2,
            accounts: vec!["alice@aureon".to_string(), "bob@aureon".to_string()],
        };

        sync.store_snapshot(snapshot);
        assert!(sync.validate_snapshot(ShardId(0), "root123"));
        assert!(!sync.validate_snapshot(ShardId(0), "wrong_root"));
    }

    #[test]
    fn test_shard_sync_counts() {
        let mut sync = ShardSync::new();
        sync.set_status(ShardId(0), SyncStatus::Synced);
        sync.set_status(ShardId(1), SyncStatus::Synced);
        sync.set_status(ShardId(2), SyncStatus::Syncing);
        sync.set_status(ShardId(3), SyncStatus::OutOfSync);

        assert_eq!(sync.synced_count(), 2);
        assert_eq!(sync.syncing_count(), 1);
        assert_eq!(sync.out_of_sync_count(), 1);
    }

    #[test]
    fn test_get_synced_shards() {
        let mut sync = ShardSync::new();
        sync.set_status(ShardId(0), SyncStatus::Synced);
        sync.set_status(ShardId(1), SyncStatus::Syncing);
        sync.set_status(ShardId(2), SyncStatus::Synced);

        let synced = sync.get_synced_shards();
        assert_eq!(synced.len(), 2);
        assert!(synced.contains(&ShardId(0)));
        assert!(synced.contains(&ShardId(2)));
    }

    #[test]
    fn test_get_out_of_sync_shards() {
        let mut sync = ShardSync::new();
        sync.set_status(ShardId(0), SyncStatus::OutOfSync);
        sync.set_status(ShardId(1), SyncStatus::Synced);
        sync.set_status(ShardId(2), SyncStatus::OutOfSync);

        let out_of_sync = sync.get_out_of_sync_shards();
        assert_eq!(out_of_sync.len(), 2);
    }

    #[test]
    fn test_merkle_proof_simple() {
        let mut sync = ShardSync::new();
        let snapshot = ShardStateSnapshot {
            shard_id: ShardId(0),
            block_number: 100,
            state_root: "root".to_string(),
            account_count: 1,
            accounts: vec!["alice@aureon".to_string()],
        };

        sync.store_snapshot(snapshot);
        let proof = sync.generate_merkle_proof(ShardId(0), "alice@aureon");
        assert!(proof.is_some());
    }
}
