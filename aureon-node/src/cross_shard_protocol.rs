use std::collections::HashMap;
use crate::shard_coordinator::ShardId;

/// Receipt confirming a cross-shard transaction phase completed
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionReceipt {
    pub tx_id: String,
    pub phase: TransactionPhase,
    pub shard: ShardId,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Transaction phase in two-phase commit protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionPhase {
    /// Phase 1: Prepare - all shards validate the transaction
    Prepare,
    /// Phase 2: Commit - all shards execute the transaction
    Commit,
    /// Abort - transaction failed, rollback initiated
    Abort,
}

/// Represents a cross-shard transaction
/// May involve multiple shards, ensuring atomicity via two-phase commit
#[derive(Debug, Clone)]
pub struct CrossShardTransaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    /// Shards involved in this transaction
    pub involved_shards: Vec<ShardId>,
    /// Prepare phase receipts (one per shard)
    pub prepare_receipts: HashMap<ShardId, TransactionReceipt>,
    /// Commit phase receipts (one per shard)
    pub commit_receipts: HashMap<ShardId, TransactionReceipt>,
    /// Current state of transaction
    pub state: CrossShardState,
}

/// State of a cross-shard transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossShardState {
    /// Created, not yet prepared
    Pending,
    /// All shards have prepared successfully
    ReadyToCommit,
    /// Transaction has been committed
    Committed,
    /// Transaction was aborted
    Aborted,
}

impl CrossShardTransaction {
    /// Create a new cross-shard transaction
    pub fn new(
        id: String,
        from: String,
        to: String,
        amount: u64,
        timestamp: u64,
        involved_shards: Vec<ShardId>,
    ) -> Self {
        CrossShardTransaction {
            id,
            from,
            to,
            amount,
            timestamp,
            involved_shards,
            prepare_receipts: HashMap::new(),
            commit_receipts: HashMap::new(),
            state: CrossShardState::Pending,
        }
    }

    /// Record a prepare phase receipt
    pub fn add_prepare_receipt(&mut self, receipt: TransactionReceipt) {
        self.prepare_receipts.insert(receipt.shard, receipt);
    }

    /// Record a commit phase receipt
    pub fn add_commit_receipt(&mut self, receipt: TransactionReceipt) {
        self.commit_receipts.insert(receipt.shard, receipt);
    }

    /// Check if all shards have prepared successfully
    pub fn all_prepared(&self) -> bool {
        self.involved_shards.len() == self.prepare_receipts.len()
            && self
                .prepare_receipts
                .values()
                .all(|r| r.success && r.phase == TransactionPhase::Prepare)
    }

    /// Check if all shards have committed successfully
    pub fn all_committed(&self) -> bool {
        self.involved_shards.len() == self.commit_receipts.len()
            && self
                .commit_receipts
                .values()
                .all(|r| r.success && r.phase == TransactionPhase::Commit)
    }

    /// Check if any prepare phase failed
    pub fn prepare_failed(&self) -> bool {
        self.prepare_receipts
            .values()
            .any(|r| !r.success && r.phase == TransactionPhase::Prepare)
    }

    /// Get all shards that haven't prepared yet
    pub fn pending_prepare_shards(&self) -> Vec<ShardId> {
        self.involved_shards
            .iter()
            .filter(|shard| !self.prepare_receipts.contains_key(shard))
            .copied()
            .collect()
    }

    /// Get all shards that haven't committed yet
    pub fn pending_commit_shards(&self) -> Vec<ShardId> {
        self.involved_shards
            .iter()
            .filter(|shard| !self.commit_receipts.contains_key(shard))
            .copied()
            .collect()
    }

    /// Move to ReadyToCommit state if all shards prepared
    pub fn try_ready_to_commit(&mut self) {
        if self.all_prepared() && self.state == CrossShardState::Pending {
            self.state = CrossShardState::ReadyToCommit;
        }
    }

    /// Move to Committed state if all shards committed
    pub fn try_committed(&mut self) {
        if self.all_committed() && self.state == CrossShardState::ReadyToCommit {
            self.state = CrossShardState::Committed;
        }
    }

    /// Abort the transaction
    pub fn abort(&mut self) {
        self.state = CrossShardState::Aborted;
    }
}

/// Cross-shard protocol manager
/// Coordinates two-phase commit protocol for transactions spanning multiple shards
#[derive(Debug)]
pub struct CrossShardProtocol {
    pending_transactions: HashMap<String, CrossShardTransaction>,
}

impl CrossShardProtocol {
    /// Create a new cross-shard protocol manager
    pub fn new() -> Self {
        CrossShardProtocol {
            pending_transactions: HashMap::new(),
        }
    }

    /// Register a new cross-shard transaction
    pub fn register_transaction(&mut self, tx: CrossShardTransaction) {
        self.pending_transactions.insert(tx.id.clone(), tx);
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&CrossShardTransaction> {
        self.pending_transactions.get(tx_id)
    }

    /// Get a mutable transaction by ID
    pub fn get_transaction_mut(&mut self, tx_id: &str) -> Option<&mut CrossShardTransaction> {
        self.pending_transactions.get_mut(tx_id)
    }

    /// Process a prepare receipt for a transaction
    /// Returns the transaction state after processing
    pub fn process_prepare_receipt(
        &mut self,
        tx_id: &str,
        receipt: TransactionReceipt,
    ) -> Option<CrossShardState> {
        if let Some(tx) = self.get_transaction_mut(tx_id) {
            tx.add_prepare_receipt(receipt);

            if tx.prepare_failed() {
                tx.abort();
                return Some(CrossShardState::Aborted);
            }

            tx.try_ready_to_commit();
            Some(tx.state)
        } else {
            None
        }
    }

    /// Process a commit receipt for a transaction
    /// Returns the transaction state after processing
    pub fn process_commit_receipt(
        &mut self,
        tx_id: &str,
        receipt: TransactionReceipt,
    ) -> Option<CrossShardState> {
        if let Some(tx) = self.get_transaction_mut(tx_id) {
            tx.add_commit_receipt(receipt);
            tx.try_committed();
            Some(tx.state)
        } else {
            None
        }
    }

    /// Finalize a transaction (remove from pending)
    pub fn finalize_transaction(&mut self, tx_id: &str) -> Option<CrossShardTransaction> {
        self.pending_transactions.remove(tx_id)
    }

    /// Get count of pending transactions
    pub fn pending_count(&self) -> usize {
        self.pending_transactions.len()
    }

    /// Get count of transactions in specific state
    pub fn count_in_state(&self, state: CrossShardState) -> usize {
        self.pending_transactions
            .values()
            .filter(|tx| tx.state == state)
            .count()
    }

    /// Get all transactions in specific state
    pub fn transactions_in_state(&self, state: CrossShardState) -> Vec<&CrossShardTransaction> {
        self.pending_transactions
            .values()
            .filter(|tx| tx.state == state)
            .collect()
    }
}

impl Default for CrossShardProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_shard_transaction_creation() {
        let tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1)],
        );

        assert_eq!(tx.id, "tx_001");
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.state, CrossShardState::Pending);
        assert_eq!(tx.involved_shards.len(), 2);
    }

    #[test]
    fn test_transaction_prepare_receipts() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1)],
        );

        let receipt = TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: true,
            error_message: None,
        };

        tx.add_prepare_receipt(receipt);
        assert_eq!(tx.prepare_receipts.len(), 1);
        assert!(!tx.all_prepared()); // Only 1 of 2 shards prepared
    }

    #[test]
    fn test_all_prepared() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1)],
        );

        tx.add_prepare_receipt(TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: true,
            error_message: None,
        });

        tx.add_prepare_receipt(TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(1),
            success: true,
            error_message: None,
        });

        assert!(tx.all_prepared());
    }

    #[test]
    fn test_prepare_failed() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1)],
        );

        tx.add_prepare_receipt(TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: false,
            error_message: Some("Insufficient balance".to_string()),
        });

        assert!(tx.prepare_failed());
    }

    #[test]
    fn test_try_ready_to_commit() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0)],
        );

        tx.add_prepare_receipt(TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: true,
            error_message: None,
        });

        tx.try_ready_to_commit();
        assert_eq!(tx.state, CrossShardState::ReadyToCommit);
    }

    #[test]
    fn test_protocol_register_transaction() {
        let mut protocol = CrossShardProtocol::new();
        let tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1)],
        );

        protocol.register_transaction(tx);
        assert_eq!(protocol.pending_count(), 1);
    }

    #[test]
    fn test_protocol_process_prepare_receipt() {
        let mut protocol = CrossShardProtocol::new();
        let tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0)],
        );

        protocol.register_transaction(tx);

        let receipt = TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: true,
            error_message: None,
        };

        let state = protocol.process_prepare_receipt("tx_001", receipt);
        assert_eq!(state, Some(CrossShardState::ReadyToCommit));
    }

    #[test]
    fn test_protocol_count_in_state() {
        let mut protocol = CrossShardProtocol::new();

        for i in 0..3 {
            let tx = CrossShardTransaction::new(
                format!("tx_{:03}", i),
                "alice@aureon".to_string(),
                "bob@aureon".to_string(),
                100,
                12345,
                vec![ShardId(0)],
            );
            protocol.register_transaction(tx);
        }

        assert_eq!(protocol.count_in_state(CrossShardState::Pending), 3);
        assert_eq!(protocol.count_in_state(CrossShardState::Committed), 0);
    }

    #[test]
    fn test_pending_shards() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0), ShardId(1), ShardId(2)],
        );

        assert_eq!(tx.pending_prepare_shards(), vec![ShardId(0), ShardId(1), ShardId(2)]);

        tx.add_prepare_receipt(TransactionReceipt {
            tx_id: "tx_001".to_string(),
            phase: TransactionPhase::Prepare,
            shard: ShardId(0),
            success: true,
            error_message: None,
        });

        assert_eq!(tx.pending_prepare_shards(), vec![ShardId(1), ShardId(2)]);
    }

    #[test]
    fn test_abort_transaction() {
        let mut tx = CrossShardTransaction::new(
            "tx_001".to_string(),
            "alice@aureon".to_string(),
            "bob@aureon".to_string(),
            100,
            12345,
            vec![ShardId(0)],
        );

        tx.abort();
        assert_eq!(tx.state, CrossShardState::Aborted);
    }
}
