use crate::types::{Block, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory blockchain indexes for fast data lookups
/// Maintains mappings from block/transaction hashes to their data
#[derive(Clone, Debug)]
pub struct BlockchainIndexer {
    /// Block hash -> Block data
    blocks: Arc<Mutex<HashMap<String, BlockIndexEntry>>>,
    /// Transaction hash -> Transaction data + containing block hash
    transactions: Arc<Mutex<HashMap<String, TransactionIndexEntry>>>,
    /// Block number -> Block hash (for sequential queries)
    block_numbers: Arc<Mutex<HashMap<u64, String>>>,
}

/// Indexed block information
#[derive(Clone, Debug)]
pub struct BlockIndexEntry {
    pub block: Block,
    pub block_number: u64,
    pub timestamp: u64,
}

/// Indexed transaction information
#[derive(Clone, Debug)]
pub struct TransactionIndexEntry {
    pub transaction: Transaction,
    pub block_hash: String,
    pub block_number: u64,
    pub tx_index: usize,  // Position in block transactions
}

impl BlockchainIndexer {
    /// Create a new empty indexer
    pub fn new() -> Self {
        BlockchainIndexer {
            blocks: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            block_numbers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Index a newly produced block
    /// Should be called after block is created but before broadcast
    pub fn index_block(
        &self,
        block: Block,
        block_number: u64,
        timestamp: u64,
    ) -> Result<(), String> {
        let block_hash = block.hash.clone();

        // Index block
        let mut blocks = self.blocks.lock().map_err(|e| e.to_string())?;
        blocks.insert(
            block_hash.clone(),
            BlockIndexEntry {
                block: block.clone(),
                block_number,
                timestamp,
            },
        );

        // Index block by number
        let mut block_numbers = self.block_numbers.lock().map_err(|e| e.to_string())?;
        block_numbers.insert(block_number, block_hash.clone());

        // Index transactions within the block
        let mut transactions = self.transactions.lock().map_err(|e| e.to_string())?;
        for (tx_index, tx) in block.transactions.iter().enumerate() {
            // Compute transaction hash (simple hash of serialized tx)
            let tx_hash = self.compute_tx_hash(tx);
            transactions.insert(
                tx_hash,
                TransactionIndexEntry {
                    transaction: tx.clone(),
                    block_hash: block_hash.clone(),
                    block_number,
                    tx_index,
                },
            );
        }

        Ok(())
    }

    /// Retrieve block by hash
    pub fn get_block(&self, block_hash: &str) -> Result<Option<BlockIndexEntry>, String> {
        let blocks = self.blocks.lock().map_err(|e| e.to_string())?;
        Ok(blocks.get(block_hash).cloned())
    }

    /// Retrieve block by block number
    pub fn get_block_by_number(&self, block_number: u64) -> Result<Option<BlockIndexEntry>, String> {
        let block_numbers = self.block_numbers.lock().map_err(|e| e.to_string())?;
        if let Some(block_hash) = block_numbers.get(&block_number) {
            let blocks = self.blocks.lock().map_err(|e| e.to_string())?;
            Ok(blocks.get(block_hash).cloned())
        } else {
            Ok(None)
        }
    }

    /// Retrieve transaction by hash
    pub fn get_transaction(&self, tx_hash: &str) -> Result<Option<TransactionIndexEntry>, String> {
        let transactions = self.transactions.lock().map_err(|e| e.to_string())?;
        Ok(transactions.get(tx_hash).cloned())
    }

    /// Get all transactions in a block
    pub fn get_block_transactions(
        &self,
        block_hash: &str,
    ) -> Result<Vec<TransactionIndexEntry>, String> {
        let transactions = self.transactions.lock().map_err(|e| e.to_string())?;
        let mut block_txs: Vec<_> = transactions
            .values()
            .filter(|tx| &tx.block_hash == block_hash)
            .cloned()
            .collect();

        // Sort by transaction index to maintain order
        block_txs.sort_by_key(|tx| tx.tx_index);
        Ok(block_txs)
    }

    /// Get latest block number
    pub fn get_latest_block_number(&self) -> Result<Option<u64>, String> {
        let block_numbers = self.block_numbers.lock().map_err(|e| e.to_string())?;
        Ok(block_numbers.keys().max().copied())
    }

    /// Get latest block hash
    pub fn get_latest_block_hash(&self) -> Result<Option<String>, String> {
        let latest_num = self.get_latest_block_number()?;
        if let Some(num) = latest_num {
            let block_numbers = self.block_numbers.lock().map_err(|e| e.to_string())?;
            Ok(block_numbers.get(&num).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get transaction count
    pub fn get_transaction_count(&self) -> Result<u64, String> {
        let transactions = self.transactions.lock().map_err(|e| e.to_string())?;
        Ok(transactions.len() as u64)
    }

    /// Get block count
    pub fn get_block_count(&self) -> Result<u64, String> {
        let blocks = self.blocks.lock().map_err(|e| e.to_string())?;
        Ok(blocks.len() as u64)
    }

    /// Clear all indexes (useful for testing)
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), String> {
        self.blocks.lock().map_err(|e| e.to_string())?.clear();
        self.transactions.lock().map_err(|e| e.to_string())?.clear();
        self.block_numbers.lock().map_err(|e| e.to_string())?.clear();
        Ok(())
    }

    /// Compute hash of a transaction (simple SHA256 of debug representation)
    fn compute_tx_hash(&self, tx: &Transaction) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", tx).as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for BlockchainIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block() -> Block {
        Block {
            transactions: vec![],
            previous_hash: "genesis".to_string(),
            nonce: 0,
            hash: "test_block_hash".to_string(),
            pre_state_root: vec![],
            post_state_root: vec![],
        }
    }

    #[test]
    fn test_index_and_retrieve_block() {
        let indexer = BlockchainIndexer::new();
        let block = create_test_block();

        indexer
            .index_block(block.clone(), 0, 1000)
            .expect("Failed to index block");

        let retrieved = indexer
            .get_block(&block.hash)
            .expect("Failed to retrieve block")
            .expect("Block not found");

        assert_eq!(retrieved.block.hash, block.hash);
        assert_eq!(retrieved.block_number, 0);
        assert_eq!(retrieved.timestamp, 1000);
    }

    #[test]
    fn test_get_block_by_number() {
        let indexer = BlockchainIndexer::new();
        let block = create_test_block();

        indexer
            .index_block(block.clone(), 5, 1000)
            .expect("Failed to index block");

        let retrieved = indexer
            .get_block_by_number(5)
            .expect("Failed to retrieve block by number")
            .expect("Block not found");

        assert_eq!(retrieved.block_number, 5);
    }

    #[test]
    fn test_latest_block_number() {
        let indexer = BlockchainIndexer::new();

        let block1 = create_test_block();
        indexer
            .index_block(block1, 0, 1000)
            .expect("Failed to index block");

        let mut block2 = create_test_block();
        block2.hash = "block2_hash".to_string();
        indexer
            .index_block(block2, 1, 2000)
            .expect("Failed to index block");

        let latest = indexer
            .get_latest_block_number()
            .expect("Failed to get latest")
            .expect("No blocks");

        assert_eq!(latest, 1);
    }

    #[test]
    fn test_block_count() {
        let indexer = BlockchainIndexer::new();

        let block1 = create_test_block();
        indexer
            .index_block(block1, 0, 1000)
            .expect("Failed to index block");

        let mut block2 = create_test_block();
        block2.hash = "block2_hash".to_string();
        indexer
            .index_block(block2, 1, 2000)
            .expect("Failed to index block");

        let count = indexer.get_block_count().expect("Failed to count blocks");
        assert_eq!(count, 2);
    }
}
