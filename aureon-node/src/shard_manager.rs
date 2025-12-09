use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::shard_coordinator::{ShardId, ShardCoordinator};
use crate::types::Account;

/// Per-shard ledger containing accounts and their balances
/// Each shard maintains independent state that is processed by shard-specific validators
#[derive(Debug, Clone)]
pub struct ShardLedger {
    /// Accounts managed by this shard, keyed by address
    pub accounts: HashMap<String, Account>,
    /// Running hash of shard state for merkle proof validation
    pub state_root: String,
    /// Block number when this shard state was last updated
    pub last_updated_block: u64,
}

impl ShardLedger {
    /// Create a new empty shard ledger
    pub fn new() -> Self {
        ShardLedger {
            accounts: HashMap::new(),
            state_root: String::from("0"),
            last_updated_block: 0,
        }
    }

    /// Get an account from this shard
    pub fn get_account(&self, address: &str) -> Option<&Account> {
        self.accounts.get(address)
    }

    /// Get mutable account reference (for state updates)
    pub fn get_account_mut(&mut self, address: &str) -> Option<&mut Account> {
        self.accounts.get_mut(address)
    }

    /// Add or update an account in this shard
    pub fn set_account(&mut self, address: String, account: Account) {
        self.accounts.insert(address, account);
    }

    /// Remove an account from this shard
    pub fn remove_account(&mut self, address: &str) -> Option<Account> {
        self.accounts.remove(address)
    }

    /// Get account count in this shard
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Update the state root hash
    pub fn update_state_root(&mut self, new_root: String) {
        self.state_root = new_root;
    }

    /// Get the current state root
    pub fn get_state_root(&self) -> &str {
        &self.state_root
    }

    /// Mark shard as updated at specific block
    pub fn update_block_number(&mut self, block_num: u64) {
        self.last_updated_block = block_num;
    }
}

impl Default for ShardLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-shard state management
/// Maintains individual ledgers for each shard with atomic operations
#[derive(Debug)]
pub struct ShardManager {
    coordinator: ShardCoordinator,
    shards: Vec<Arc<RwLock<ShardLedger>>>,
}

impl ShardManager {
    /// Create a new ShardManager with default shard configuration
    pub fn new(coordinator: ShardCoordinator) -> Self {
        let num_shards = coordinator.num_shards();
        let shards = (0..num_shards)
            .map(|_| Arc::new(RwLock::new(ShardLedger::new())))
            .collect();

        ShardManager { coordinator, shards }
    }

    /// Get the shard for an account
    pub fn get_shard_id(&self, account_address: &str) -> ShardId {
        self.coordinator.get_shard(account_address)
    }

    /// Get mutable access to a shard ledger
    /// 
    /// # Panics
    /// Panics if shard ID is invalid
    fn get_shard_mut(&self, shard: ShardId) -> Arc<RwLock<ShardLedger>> {
        assert!(
            self.coordinator.is_valid_shard(shard),
            "Invalid shard ID: {}",
            shard.0
        );
        Arc::clone(&self.shards[shard.0 as usize])
    }

    /// Get read-only access to a shard ledger
    fn get_shard_read(&self, shard: ShardId) -> Arc<RwLock<ShardLedger>> {
        assert!(
            self.coordinator.is_valid_shard(shard),
            "Invalid shard ID: {}",
            shard.0
        );
        Arc::clone(&self.shards[shard.0 as usize])
    }

    /// Get or create an account in the appropriate shard
    pub fn get_or_create_account(&self, address: String) -> Account {
        let shard = self.get_shard_id(&address);
        let shard_ledger = self.get_shard_read(shard);
        
        {
            let ledger = shard_ledger.read().unwrap();
            if let Some(account) = ledger.get_account(&address) {
                return account.clone();
            }
        }
        
        // Create new account
        Account {
            address: address.clone(),
            balance: 0,
            nonce: 0,
            code: vec![],
            storage: HashMap::new(),
        }
    }

    /// Update an account in the appropriate shard
    pub fn update_account(&self, address: String, account: Account) {
        let shard = self.get_shard_id(&address);
        let shard_ledger = self.get_shard_mut(shard);
        let mut ledger = shard_ledger.write().unwrap();
        ledger.set_account(address, account);
    }

    /// Get account balance from appropriate shard
    pub fn get_balance(&self, address: &str) -> u64 {
        let shard = self.get_shard_id(address);
        let shard_ledger = self.get_shard_read(shard);
        let ledger = shard_ledger.read().unwrap();
        
        ledger
            .get_account(address)
            .map(|account| account.balance)
            .unwrap_or(0)
    }

    /// Update account balance in appropriate shard
    pub fn set_balance(&self, address: String, new_balance: u64) {
        let shard = self.get_shard_id(&address);
        let shard_ledger = self.get_shard_mut(shard);
        let mut ledger = shard_ledger.write().unwrap();
        
        if let Some(account) = ledger.get_account_mut(&address) {
            account.balance = new_balance;
        } else {
            // Create new account if it doesn't exist
            let account = Account {
                address: address.clone(),
                balance: new_balance,
                nonce: 0,
                code: vec![],
                storage: HashMap::new(),
            };
            ledger.set_account(address, account);
        }
    }

    /// Transfer balance between accounts (may be cross-shard)
    /// Returns true if successful
    pub fn transfer(&self, from: &str, to: &str, amount: u64) -> bool {
        let from_balance = self.get_balance(from);
        if from_balance < amount {
            return false;
        }

        self.set_balance(from.to_string(), from_balance - amount);
        
        let to_balance = self.get_balance(to);
        self.set_balance(to.to_string(), to_balance + amount);
        
        true
    }

    /// Get total account count across all shards
    pub fn total_account_count(&self) -> usize {
        self.shards
            .iter()
            .map(|shard| {
                let ledger = shard.read().unwrap();
                ledger.account_count()
            })
            .sum()
    }

    /// Get account count for a specific shard
    pub fn shard_account_count(&self, shard: ShardId) -> usize {
        let shard_ledger = self.get_shard_read(shard);
        let ledger = shard_ledger.read().unwrap();
        ledger.account_count()
    }

    /// Update state root for a shard
    pub fn update_shard_root(&self, shard: ShardId, new_root: String) {
        let shard_ledger = self.get_shard_mut(shard);
        let mut ledger = shard_ledger.write().unwrap();
        ledger.update_state_root(new_root);
    }

    /// Get state root for a shard
    pub fn get_shard_root(&self, shard: ShardId) -> String {
        let shard_ledger = self.get_shard_read(shard);
        let ledger = shard_ledger.read().unwrap();
        ledger.get_state_root().to_string()
    }

    /// Get the coordinator (for querying shard info)
    pub fn coordinator(&self) -> &ShardCoordinator {
        &self.coordinator
    }

    /// Check if two accounts are in the same shard
    pub fn same_shard(&self, addr1: &str, addr2: &str) -> bool {
        self.coordinator.same_shard(addr1, addr2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard_coordinator::ShardCoordinator;

    #[test]
    fn test_shard_ledger_creation() {
        let ledger = ShardLedger::new();
        assert_eq!(ledger.account_count(), 0);
        assert_eq!(ledger.get_state_root(), "0");
        assert_eq!(ledger.last_updated_block, 0);
    }

    #[test]
    fn test_shard_ledger_account_operations() {
        let mut ledger = ShardLedger::new();
        let account = Account {
            address: "alice@aureon".to_string(),
            balance: 100,
            nonce: 0,
            code: vec![],
            storage: HashMap::new(),
        };

        ledger.set_account("alice@aureon".to_string(), account.clone());
        assert_eq!(ledger.account_count(), 1);
        assert_eq!(ledger.get_account("alice@aureon"), Some(&account));
    }

    #[test]
    fn test_shard_manager_creation() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);
        assert_eq!(manager.total_account_count(), 0);
    }

    #[test]
    fn test_get_or_create_account() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let account = manager.get_or_create_account("alice@aureon".to_string());
        assert_eq!(account.address, "alice@aureon");
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn test_update_and_retrieve_account() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let mut account = manager.get_or_create_account("alice@aureon".to_string());
        account.balance = 500;
        manager.update_account("alice@aureon".to_string(), account);

        assert_eq!(manager.get_balance("alice@aureon"), 500);
    }

    #[test]
    fn test_transfer_balance() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let from = "alice@aureon";
        let to = "bob@aureon";

        manager.update_account(
            from.to_string(),
            Account {
                address: from.to_string(),
                balance: 1000,
                nonce: 0,
                code: vec![],
                storage: HashMap::new(),
            },
        );

        assert!(manager.transfer(from, to, 300));
        assert_eq!(manager.get_balance(from), 700);
        assert_eq!(manager.get_balance(to), 300);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let from = "alice@aureon";
        let to = "bob@aureon";

        manager.update_account(
            from.to_string(),
            Account {
                address: from.to_string(),
                balance: 100,
                nonce: 0,
                code: vec![],
                storage: HashMap::new(),
            },
        );

        assert!(!manager.transfer(from, to, 500));
        assert_eq!(manager.get_balance(from), 100);
        assert_eq!(manager.get_balance(to), 0);
    }

    #[test]
    fn test_shard_account_count() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let addr1 = "alice@aureon";
        manager.update_account(
            addr1.to_string(),
            Account {
                address: addr1.to_string(),
                balance: 100,
                nonce: 0,
                code: vec![],
                storage: HashMap::new(),
            },
        );

        let shard_id = manager.get_shard_id(addr1);
        assert_eq!(manager.shard_account_count(shard_id), 1);
    }

    #[test]
    fn test_same_shard() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let addr = "alice@aureon";
        assert!(manager.same_shard(addr, addr));
    }

    #[test]
    fn test_update_shard_root() {
        let coordinator = ShardCoordinator::with_shard_count(4);
        let manager = ShardManager::new(coordinator);

        let shard = ShardId(0);
        manager.update_shard_root(shard, "new_root_hash".to_string());
        assert_eq!(manager.get_shard_root(shard), "new_root_hash");
    }
}
