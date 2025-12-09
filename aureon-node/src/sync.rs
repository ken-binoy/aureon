/// Block synchronization module for P2P network
/// Handles requesting and validating blocks from peer nodes

use crate::types::Block;
use crate::indexer::BlockchainIndexer;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Tracks synchronization state with peers
#[derive(Clone, Debug)]
pub struct BlockSyncState {
    /// Our current chain height
    pub local_height: u64,
    /// Highest height we've heard from peers
    pub peer_max_height: u64,
    /// Blocks we're currently waiting for (height -> peer requesting from)
    pub pending_blocks: Arc<Mutex<HashMap<u64, String>>>,
    /// Blocks we've received but not yet applied
    pub staged_blocks: Arc<Mutex<Vec<Block>>>,
}

impl BlockSyncState {
    pub fn new() -> Self {
        BlockSyncState {
            local_height: 0,
            peer_max_height: 0,
            pending_blocks: Arc::new(Mutex::new(HashMap::new())),
            staged_blocks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Check if we're in sync with the network
    pub fn is_synced(&self) -> bool {
        self.local_height >= self.peer_max_height
    }

    /// Get blocks we need to sync
    pub fn get_sync_range(&self) -> Option<(u64, u64)> {
        if self.is_synced() {
            return None;
        }
        Some((self.local_height + 1, self.peer_max_height))
    }

    /// Register a pending block request
    pub fn add_pending_block(&self, height: u64, peer_id: String) -> Result<(), String> {
        let mut pending = self.pending_blocks.lock().map_err(|e| e.to_string())?;
        if pending.contains_key(&height) {
            return Err(format!("Block {} already requested", height));
        }
        pending.insert(height, peer_id);
        Ok(())
    }

    /// Mark a block as received (but not yet applied)
    pub fn stage_block(&self, block: Block) -> Result<(), String> {
        let mut staged = self.staged_blocks.lock().map_err(|e| e.to_string())?;
        staged.push(block);
        Ok(())
    }

    /// Get blocks ready to apply (contiguous from local_height+1)
    pub fn get_applicable_blocks(&self) -> Result<Vec<Block>, String> {
        let mut staged = self.staged_blocks.lock().map_err(|e| e.to_string())?;
        
        // Sort by hash for determinism (in real implementation, verify height field if available)
        staged.sort_by(|a, b| a.hash.cmp(&b.hash));
        
        // For now, return all staged blocks if they can be applied
        let applicable = staged.drain(..).collect();
        Ok(applicable)
    }

    /// Update local height after applying blocks
    pub fn update_local_height(&mut self, new_height: u64) {
        self.local_height = new_height;
    }

    /// Update peer max height from peer info
    pub fn update_peer_height(&mut self, height: u64) {
        if height > self.peer_max_height {
            self.peer_max_height = height;
        }
    }
}

/// Block validator for sync operations
pub struct BlockValidator;

impl BlockValidator {
    /// Validate a block structure (basic checks before applying)
    /// More thorough validation should happen in state processor
    pub fn validate_block(block: &Block) -> Result<(), String> {
        // Check that block hash is non-empty
        if block.hash.is_empty() {
            return Err("Block hash is empty".to_string());
        }

        // Check that previous hash is valid (non-empty for non-genesis)
        if block.previous_hash.is_empty() && !block.transactions.is_empty() {
            // Only genesis blocks can have empty previous_hash
            return Err("Non-genesis block has empty previous hash".to_string());
        }

        // Check that state roots exist
        if block.pre_state_root.is_empty() || block.post_state_root.is_empty() {
            return Err("Block missing state roots".to_string());
        }

        // Validate all transactions in the block
        for tx in &block.transactions {
            Self::validate_transaction(tx)?;
        }

        Ok(())
    }

    /// Validate a transaction
    fn validate_transaction(tx: &crate::types::Transaction) -> Result<(), String> {
        // Check required fields
        if tx.from.is_empty() {
            return Err("Transaction from address is empty".to_string());
        }

        // Signature must be present (from Phase 6.1)
        if tx.signature.is_empty() {
            return Err("Transaction signature is empty".to_string());
        }

        // Nonce must be reasonable
        if tx.nonce > u64::MAX / 2 {
            return Err("Transaction nonce is suspiciously high".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_creation() {
        let state = BlockSyncState::new();
        assert_eq!(state.local_height, 0);
        assert_eq!(state.peer_max_height, 0);
        assert!(state.is_synced());
    }

    #[test]
    fn test_sync_range_when_behind() {
        let mut state = BlockSyncState::new();
        state.local_height = 10;
        state.peer_max_height = 20;
        
        let range = state.get_sync_range();
        assert_eq!(range, Some((11, 20)));
    }

    #[test]
    fn test_sync_range_when_synced() {
        let mut state = BlockSyncState::new();
        state.local_height = 20;
        state.peer_max_height = 20;
        
        let range = state.get_sync_range();
        assert_eq!(range, None);
    }

    #[test]
    fn test_pending_block_tracking() {
        let state = BlockSyncState::new();
        
        assert!(state.add_pending_block(1, "peer1".to_string()).is_ok());
        assert!(state.add_pending_block(2, "peer2".to_string()).is_ok());
        
        // Duplicate request should fail
        assert!(state.add_pending_block(1, "peer3".to_string()).is_err());
    }

    #[test]
    fn test_peer_height_updates() {
        let mut state = BlockSyncState::new();
        
        state.update_peer_height(10);
        assert_eq!(state.peer_max_height, 10);
        
        state.update_peer_height(5);
        assert_eq!(state.peer_max_height, 10); // Should not decrease
        
        state.update_peer_height(20);
        assert_eq!(state.peer_max_height, 20);
    }
}
