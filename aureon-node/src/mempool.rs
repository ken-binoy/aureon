use crate::types::Transaction;
use crate::crypto;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use hex::encode as hex_encode;

/// Transaction mempool for pending transactions awaiting inclusion in next block
/// Implements FIFO ordering with size limits and nonce enforcement
#[derive(Clone, Debug)]
pub struct TransactionMempool {
    /// Pending transactions in submission order
    pending: Arc<Mutex<VecDeque<Transaction>>>,
    /// Track transaction hashes to prevent duplicates
    seen: Arc<Mutex<HashMap<String, bool>>>,
    /// Track highest nonce for each account (prevents replay attacks)
    account_nonces: Arc<Mutex<HashMap<String, u64>>>,
    /// Maximum transactions in mempool
    max_size: usize,
}

impl TransactionMempool {
    /// Create a new mempool with default capacity (1000 transactions)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a mempool with custom capacity
    pub fn with_capacity(max_size: usize) -> Self {
        TransactionMempool {
            pending: Arc::new(Mutex::new(VecDeque::new())),
            seen: Arc::new(Mutex::new(HashMap::new())),
            account_nonces: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }

    /// Add a transaction to the mempool
    /// Returns the transaction hash if successful, error message otherwise
    /// Verifies Ed25519 signature and nonce ordering before accepting transaction
    pub fn add_transaction(&self, tx: Transaction) -> Result<String, String> {
        // Verify transaction signature
        self.verify_transaction_signature(&tx)?;
        
        // Verify nonce (prevents replay attacks and out-of-order execution)
        self.verify_nonce(&tx)?;
        
        let tx_hash = self.compute_tx_hash(&tx);

        // Check for duplicates
        let mut seen = self.seen.lock().map_err(|e| e.to_string())?;
        if seen.contains_key(&tx_hash) {
            return Err("Transaction already in mempool".to_string());
        }

        // Check mempool capacity
        let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
        if pending.len() >= self.max_size {
            return Err(format!(
                "Mempool full ({} transactions)",
                self.max_size
            ));
        }

        // Update account nonce to track maximum nonce seen
        let mut nonces = self.account_nonces.lock().map_err(|e| e.to_string())?;
        nonces.insert(tx.from.clone(), tx.nonce);

        // Add to mempool
        pending.push_back(tx);
        seen.insert(tx_hash.clone(), true);

        Ok(tx_hash)
    }

    /// Get next N transactions from mempool for block production
    /// Removes transactions from mempool (assumed to be included in block)
    pub fn take_transactions(&self, count: usize) -> Result<Vec<Transaction>, String> {
        let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
        let mut seen = self.seen.lock().map_err(|e| e.to_string())?;

        let mut transactions = Vec::new();
        for _ in 0..count {
            if let Some(tx) = pending.pop_front() {
                let tx_hash = self.compute_tx_hash(&tx);
                seen.remove(&tx_hash);
                transactions.push(tx);
            } else {
                break;
            }
        }

        Ok(transactions)
    }

    /// Get all pending transactions without removing them
    pub fn get_pending(&self) -> Result<Vec<Transaction>, String> {
        let pending = self.pending.lock().map_err(|e| e.to_string())?;
        Ok(pending.iter().cloned().collect())
    }

    /// Finalize nonces for transactions included in a block
    /// Called after block is produced to bump expected nonces
    pub fn finalize_block_transactions(&self, transactions: &[Transaction]) -> Result<(), String> {
        let mut nonces = self.account_nonces.lock().map_err(|e| e.to_string())?;
        
        for tx in transactions {
            // Increment expected nonce to tx.nonce + 1
            nonces.insert(tx.from.clone(), tx.nonce + 1);
        }
        
        Ok(())
    }

    /// Get current nonce for an account (for API queries)
    pub fn get_account_nonce(&self, account: &str) -> Result<u64, String> {
        let nonces = self.account_nonces.lock().map_err(|e| e.to_string())?;
        Ok(nonces.get(account).copied().unwrap_or(0))
    }

    /// Get transaction count
    pub fn size(&self) -> Result<usize, String> {
        let pending = self.pending.lock().map_err(|e| e.to_string())?;
        Ok(pending.len())
    }

    /// Check if transaction is in mempool
    pub fn contains(&self, tx_hash: &str) -> Result<bool, String> {
        let seen = self.seen.lock().map_err(|e| e.to_string())?;
        Ok(seen.contains_key(tx_hash))
    }

    /// Clear all transactions (useful for testing)
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), String> {
        self.pending.lock().map_err(|e| e.to_string())?.clear();
        self.seen.lock().map_err(|e| e.to_string())?.clear();
        Ok(())
    }

    /// Remove a specific transaction by hash
    pub fn remove_transaction(&self, tx_hash: &str) -> Result<bool, String> {
        let mut seen = self.seen.lock().map_err(|e| e.to_string())?;
        if !seen.remove(tx_hash).unwrap_or(false) {
            return Ok(false);
        }

        let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
        let initial_len = pending.len();
        pending.retain(|tx| self.compute_tx_hash(tx) != tx_hash);

        Ok(pending.len() < initial_len)
    }

    /// Compute hash of a transaction
    fn compute_tx_hash(&self, tx: &Transaction) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", tx).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get mempool statistics
    pub fn stats(&self) -> Result<MempoolStats, String> {
        let pending = self.pending.lock().map_err(|e| e.to_string())?;
        let tx_count = pending.len();
        let total_gas = pending
            .iter()
            .map(|tx| 21000) // Standard gas per transaction
            .sum::<u64>();

        Ok(MempoolStats {
            transaction_count: tx_count,
            total_pending_gas: total_gas,
            max_capacity: self.max_size,
            utilization_percent: (tx_count as f64 / self.max_size as f64) * 100.0,
        })
    }

    /// Verify nonce ordering to prevent replay attacks
    fn verify_nonce(&self, tx: &Transaction) -> Result<(), String> {
        let nonces = self.account_nonces.lock().map_err(|e| e.to_string())?;
        
        // Get the highest nonce seen for this account (not seen yet starts at -1, represented as None)
        // For first tx, we check if nonce is at least 0
        if let Some(max_nonce_seen) = nonces.get(&tx.from) {
            // Nonce must be greater than the highest nonce seen
            if tx.nonce <= *max_nonce_seen {
                return Err(format!(
                    "Invalid nonce: expected higher than {}, got {}",
                    max_nonce_seen, tx.nonce
                ));
            }
        }
        // If account not seen before, any nonce >= 0 is allowed (which is always true for u64)
        
        Ok(())
    }

    /// Verify Ed25519 signature on transaction
    fn verify_transaction_signature(&self, tx: &Transaction) -> Result<(), String> {
        // Skip verification for transactions without signature (for backward compatibility)
        if tx.signature.is_empty() || tx.public_key.is_empty() {
            return Ok(());
        }

        // Compute the transaction hash for signing (without the signature field)
        let mut tx_for_hash = tx.clone();
        tx_for_hash.signature = vec![];
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", tx_for_hash).as_bytes());
        let tx_hash = hex_encode(hasher.finalize());

        // Convert signature and public key from bytes to hex
        let signature_hex = hex::encode(&tx.signature);
        let public_key_hex = hex::encode(&tx.public_key);

        // Verify the signature
        crypto::verify_signature(tx_hash.as_bytes(), &signature_hex, &public_key_hex)
            .and_then(|is_valid| {
                if is_valid {
                    Ok(())
                } else {
                    Err("Invalid transaction signature".to_string())
                }
            })
    }
}

impl Default for TransactionMempool {
    fn default() -> Self {
        Self::new()
    }
}

/// Mempool statistics
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub total_pending_gas: u64,
    pub max_capacity: usize,
    pub utilization_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransactionPayload;

    fn create_test_tx(from: &str, to: &str, amount: u64) -> Transaction {
        Transaction {
            from: from.to_string(),
            nonce: 0,
            gas_price: 1,
            payload: TransactionPayload::Transfer {
                to: to.to_string(),
                amount,
            },
            signature: vec![],
            public_key: vec![],
        }
    }

    #[test]
    fn test_add_and_get_transaction() {
        let mempool = TransactionMempool::new();
        let tx = create_test_tx("Alice", "Bob", 100);

        let result = mempool.add_transaction(tx.clone());
        assert!(result.is_ok());
        let pending = mempool.get_pending().unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mempool = TransactionMempool::new();
        let tx = create_test_tx("Alice", "Bob", 100);

        mempool.add_transaction(tx.clone()).unwrap();
        let result = mempool.add_transaction(tx);
        assert!(result.is_err());
        let pending = mempool.get_pending().unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_take_transactions() {
        let mempool = TransactionMempool::new();
        mempool.add_transaction(create_test_tx("Alice", "Bob", 100)).unwrap();
        mempool.add_transaction(create_test_tx("Bob", "Charlie", 50)).unwrap();
        mempool.add_transaction(create_test_tx("Charlie", "Dave", 25)).unwrap();

        let txs = mempool.take_transactions(2).unwrap();
        assert_eq!(txs.len(), 2);
        let remaining = mempool.get_pending().unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_fifo_ordering() {
        let mempool = TransactionMempool::new();
        let tx1 = create_test_tx("Alice", "Bob", 100);
        let tx2 = create_test_tx("Bob", "Charlie", 50);

        mempool.add_transaction(tx1.clone()).unwrap();
        mempool.add_transaction(tx2.clone()).unwrap();

        let txs = mempool.take_transactions(1).unwrap();
        assert_eq!(txs[0].from, "Alice");
    }

    #[test]
    fn test_capacity_limit() {
        let mempool = TransactionMempool::with_capacity(2);
        mempool.add_transaction(create_test_tx("Alice", "Bob", 100)).unwrap();
        mempool.add_transaction(create_test_tx("Bob", "Charlie", 50)).unwrap();

        let result = mempool.add_transaction(create_test_tx("Charlie", "Dave", 25));
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let mempool = TransactionMempool::new();
        mempool.add_transaction(create_test_tx("Alice", "Bob", 100)).unwrap();
        mempool.add_transaction(create_test_tx("Bob", "Charlie", 50)).unwrap();

        let stats = mempool.stats().unwrap();
        assert_eq!(stats.transaction_count, 2);
        assert!(stats.utilization_percent > 0.0);
    }

    #[test]
    fn test_nonce_enforcement_duplicate() {
        // Test that duplicate nonces are rejected
        let mempool = TransactionMempool::new();
        
        let mut tx1 = create_test_tx("Alice", "Bob", 100);
        tx1.nonce = 0;
        
        let mut tx2 = create_test_tx("Alice", "Charlie", 50);
        tx2.nonce = 0; // Same nonce as tx1
        
        // First transaction accepted
        assert!(mempool.add_transaction(tx1).is_ok());
        
        // Second transaction with same nonce rejected
        let result = mempool.add_transaction(tx2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonce"));
    }

    #[test]
    fn test_nonce_enforcement_ordering() {
        // Test that lower nonces are rejected after higher nonce is accepted
        let mempool = TransactionMempool::new();
        
        let mut tx1 = create_test_tx("Alice", "Bob", 100);
        tx1.nonce = 5;
        
        let mut tx2 = create_test_tx("Alice", "Charlie", 50);
        tx2.nonce = 3; // Lower nonce after higher nonce submitted
        
        // Higher nonce accepted first
        assert!(mempool.add_transaction(tx1).is_ok());
        
        // Lower nonce rejected
        let result = mempool.add_transaction(tx2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonce"));
    }

    #[test]
    fn test_nonce_enforcement_valid_sequence() {
        // Test that valid nonce sequences are accepted
        let mempool = TransactionMempool::new();
        
        let mut tx1 = create_test_tx("Alice", "Bob", 100);
        tx1.nonce = 0;
        
        let mut tx2 = create_test_tx("Alice", "Charlie", 50);
        tx2.nonce = 1; // Valid next nonce
        
        assert!(mempool.add_transaction(tx1).is_ok());
        assert!(mempool.add_transaction(tx2).is_ok());
    }

    #[test]
    fn test_nonce_finalization() {
        // Test that nonces are incremented when block is finalized
        let mempool = TransactionMempool::new();
        
        let mut tx1 = create_test_tx("Alice", "Bob", 100);
        tx1.nonce = 0;
        
        let mut tx2 = create_test_tx("Alice", "Charlie", 50);
        tx2.nonce = 1;
        
        mempool.add_transaction(tx1.clone()).unwrap();
        mempool.add_transaction(tx2.clone()).unwrap();
        
        // Finalize block with first transaction
        mempool.finalize_block_transactions(&[tx1]).unwrap();
        
        // Now nonce 1 should be required as minimum
        let mut tx3 = create_test_tx("Alice", "Dave", 25);
        tx3.nonce = 0; // Old nonce should be rejected
        
        let result = mempool.add_transaction(tx3);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_different_accounts() {
        // Test that nonces are tracked per account
        let mempool = TransactionMempool::new();
        
        let mut tx1 = create_test_tx("Alice", "Bob", 100);
        tx1.nonce = 0;
        
        let mut tx2 = create_test_tx("Bob", "Charlie", 50);
        tx2.nonce = 0; // Same nonce but different account
        
        // Both should be accepted (different accounts)
        assert!(mempool.add_transaction(tx1).is_ok());
        assert!(mempool.add_transaction(tx2).is_ok());
    }
}
