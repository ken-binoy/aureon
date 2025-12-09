use sha2::{Sha256, Digest};

/// Number of shards in the system
/// This determines horizontal scalability - each shard handles independent accounts
const NUM_SHARDS: u32 = 4;

/// Represents a shard identifier (0 to NUM_SHARDS-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShardId(pub u32);

impl ShardId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// ShardCoordinator manages deterministic account-to-shard assignment
/// Uses hash-modulo function to ensure consistent account placement across all nodes
#[derive(Debug, Clone)]
pub struct ShardCoordinator {
    num_shards: u32,
}

impl ShardCoordinator {
    /// Create a new ShardCoordinator with default configuration
    pub fn new() -> Self {
        ShardCoordinator {
            num_shards: NUM_SHARDS,
        }
    }

    /// Create a new ShardCoordinator with custom shard count
    /// Useful for testing with different shard configurations
    pub fn with_shard_count(num_shards: u32) -> Self {
        assert!(num_shards > 0, "num_shards must be at least 1");
        ShardCoordinator { num_shards }
    }

    /// Get the shard ID for an account address using deterministic hash-modulo
    /// 
    /// Deterministic sharding ensures:
    /// - Same account always maps to same shard across all nodes
    /// - Uniform distribution of accounts across shards
    /// - No need for cross-shard coordination for single-account operations
    /// 
    /// # Arguments
    /// * `account_address` - The account address to shard
    /// 
    /// # Returns
    /// ShardId - The shard this account belongs to
    pub fn get_shard(&self, account_address: &str) -> ShardId {
        let mut hasher = Sha256::new();
        hasher.update(account_address.as_bytes());
        let hash = hasher.finalize();
        
        // Use first 8 bytes of hash as u64 for modulo
        let hash_value = u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ]);
        
        let shard_idx = (hash_value % (self.num_shards as u64)) as u32;
        ShardId(shard_idx)
    }

    /// Validate that a shard ID is within valid range
    pub fn is_valid_shard(&self, shard: ShardId) -> bool {
        shard.0 < self.num_shards
    }

    /// Get all valid shard IDs
    pub fn all_shards(&self) -> Vec<ShardId> {
        (0..self.num_shards)
            .map(|i| ShardId(i))
            .collect()
    }

    /// Get total number of shards
    pub fn num_shards(&self) -> u32 {
        self.num_shards
    }

    /// Check if two accounts belong to the same shard
    /// Useful for detecting cross-shard operations
    pub fn same_shard(&self, addr1: &str, addr2: &str) -> bool {
        self.get_shard(addr1) == self.get_shard(addr2)
    }

    /// Get all accounts that belong to a specific shard
    /// Requires providing all known accounts (used for queries/validation)
    pub fn get_shard_accounts(&self, shard: ShardId, accounts: &[&str]) -> Vec<String> {
        accounts
            .iter()
            .filter(|addr| self.get_shard(addr) == shard)
            .map(|addr| addr.to_string())
            .collect()
    }
}

impl Default for ShardCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_coordinator_creation() {
        let coordinator = ShardCoordinator::new();
        assert_eq!(coordinator.num_shards(), NUM_SHARDS);
    }

    #[test]
    fn test_custom_shard_count() {
        let coordinator = ShardCoordinator::with_shard_count(8);
        assert_eq!(coordinator.num_shards(), 8);
    }

    #[test]
    #[should_panic(expected = "num_shards must be at least 1")]
    fn test_zero_shards_panics() {
        ShardCoordinator::with_shard_count(0);
    }

    #[test]
    fn test_deterministic_sharding() {
        let coordinator = ShardCoordinator::new();
        let addr = "alice@aureon";
        
        // Same address should always map to same shard
        let shard1 = coordinator.get_shard(addr);
        let shard2 = coordinator.get_shard(addr);
        assert_eq!(shard1, shard2);
    }

    #[test]
    fn test_shard_distribution() {
        let coordinator = ShardCoordinator::new();
        
        // Create many accounts and verify they distribute across shards
        let accounts: Vec<String> = (0..100)
            .map(|i| format!("account_{}", i))
            .collect();
        
        let mut shard_counts = vec![0; NUM_SHARDS as usize];
        for account in &accounts {
            let shard = coordinator.get_shard(account);
            assert!(coordinator.is_valid_shard(shard));
            shard_counts[shard.0 as usize] += 1;
        }
        
        // Verify all shards are used (probabilistic test with 100 accounts)
        assert!(shard_counts.iter().all(|&count| count > 0),
            "Not all shards were used: {:?}", shard_counts);
    }

    #[test]
    fn test_is_valid_shard() {
        let coordinator = ShardCoordinator::new();
        
        for i in 0..NUM_SHARDS {
            assert!(coordinator.is_valid_shard(ShardId(i)));
        }
        assert!(!coordinator.is_valid_shard(ShardId(NUM_SHARDS)));
        assert!(!coordinator.is_valid_shard(ShardId(NUM_SHARDS + 1)));
    }

    #[test]
    fn test_all_shards() {
        let coordinator = ShardCoordinator::new();
        let shards = coordinator.all_shards();
        
        assert_eq!(shards.len(), NUM_SHARDS as usize);
        for (i, shard) in shards.iter().enumerate() {
            assert_eq!(shard.0, i as u32);
        }
    }

    #[test]
    fn test_same_shard() {
        let coordinator = ShardCoordinator::new();
        
        let addr1 = "alice@aureon";
        let addr2 = "alice@aureon"; // Same address
        
        assert!(coordinator.same_shard(addr1, addr2));
    }

    #[test]
    fn test_different_addresses_may_be_different_shards() {
        let coordinator = ShardCoordinator::with_shard_count(100);
        
        // With 100 shards and only 2 addresses, they're likely different
        let addr1 = "alice@aureon";
        let addr2 = "bob@aureon";
        
        // This is probabilistic but extremely likely with 100 shards
        // Just verify both are valid shards
        assert!(coordinator.is_valid_shard(coordinator.get_shard(addr1)));
        assert!(coordinator.is_valid_shard(coordinator.get_shard(addr2)));
    }

    #[test]
    fn test_get_shard_accounts() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let accounts = vec!["alice@aureon", "bob@aureon", "charlie@aureon", "diana@aureon"];
        
        for shard_id in coordinator.all_shards() {
            let shard_accounts = coordinator.get_shard_accounts(shard_id, &accounts);
            
            // Verify all returned accounts actually belong to this shard
            for account in &shard_accounts {
                assert_eq!(coordinator.get_shard(account), shard_id);
            }
        }
    }

    #[test]
    fn test_shard_id_equality() {
        let shard1 = ShardId(0);
        let shard2 = ShardId(0);
        let shard3 = ShardId(1);
        
        assert_eq!(shard1, shard2);
        assert_ne!(shard1, shard3);
    }

    #[test]
    fn test_shard_id_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(ShardId(0));
        set.insert(ShardId(1));
        set.insert(ShardId(0)); // Duplicate
        
        assert_eq!(set.len(), 2);
    }
}
