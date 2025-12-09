use crate::types::Transaction;
use crate::db::Db;
use crate::mempool::TransactionMempool;
use crate::indexer::BlockchainIndexer;
use crate::metrics::Metrics;
use crate::network::Network;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Background task that produces blocks from mempool transactions at regular intervals
pub struct BlockProducer {
    mempool: Arc<TransactionMempool>,
    db: Arc<Db>,
    indexer: Arc<BlockchainIndexer>,
    metrics: Arc<Metrics>,
    block_interval_ms: u64,
}

impl BlockProducer {
    /// Create a new block producer
    pub fn new(
        mempool: Arc<TransactionMempool>,
        db: Arc<Db>,
        indexer: Arc<BlockchainIndexer>,
        metrics: Arc<Metrics>,
        block_interval_ms: u64,
    ) -> Self {
        BlockProducer {
            mempool,
            db,
            indexer,
            metrics,
            block_interval_ms,
        }
    }

    /// Start the block producer in a background thread
    pub fn start(self) {
        thread::spawn(move || {
            self.run();
        });
    }

    /// Main loop: periodically produce blocks from mempool transactions
    fn run(&self) {
        let mut block_number = 1u64;

        loop {
            thread::sleep(Duration::from_millis(self.block_interval_ms));

            // Try to get pending transactions from mempool
            match self.mempool.get_pending() {
                Ok(pending_txs) => {
                    if pending_txs.is_empty() {
                        // No transactions, skip this block
                        continue;
                    }

                    // Take up to 100 transactions from mempool for this block
                    match self.mempool.take_transactions(100) {
                        Ok(transactions) => {
                            if !transactions.is_empty() {
                                // Finalize nonces for transactions included in block
                                if let Err(e) = self.mempool.finalize_block_transactions(&transactions) {
                                    eprintln!("Failed to finalize block transactions: {}", e);
                                }
                                
                                self.produce_block_info(transactions, block_number);
                                block_number += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to take transactions from mempool: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get pending transactions: {}", e);
                }
            }
        }
    }

    /// Log block production information (simplified version for demo)
    fn produce_block_info(&self, transactions: Vec<Transaction>, block_number: u64) {
        println!("\n--- Block #{} Produced from Mempool ---", block_number);
        println!("Transactions included: {}", transactions.len());
        
        // Update metrics
        self.metrics.blocks_produced.inc();
        self.metrics.transactions_processed.inc_by(transactions.len() as u64);
        
        // Calculate total gas
        let total_gas: u64 = transactions.iter().map(|_tx| 21000).sum();
        println!("Total gas: {}", total_gas);

        // Simulate block hash (would normally be computed from block data)
        let block_hash = format!(
            "{:064x}",
            block_number as u128 * 12345 // Simplified hash
        );
        println!("Block hash: {}", block_hash);
        println!("✅ Block #{} produced", block_number);
    }

    /// Get block by number from indexer (for P2P sync)
    pub fn get_block_by_number(&self, block_number: u64) -> Result<Option<crate::types::Block>, String> {
        match self.indexer.get_block_by_number(block_number)? {
            Some(entry) => Ok(Some(entry.block)),
            None => Ok(None),
        }
    }

    /// Get blocks in range from indexer (for P2P batch sync)
    pub fn get_blocks_in_range(&self, from: u64, to: u64) -> Result<Vec<crate::types::Block>, String> {
        let mut blocks = Vec::new();
        for height in from..=to {
            if let Ok(Some(block)) = self.get_block_by_number(height) {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    /// Broadcast a block to all peers (called when block is produced)
    pub fn broadcast_block(&self, network: &Network, block: &crate::types::Block) {
        network.broadcast_block(block);
    }

    /// Handle incoming GetBlock request from peer
    pub fn handle_get_block_request(&self, network: &Network, height: u64) {
        match self.get_block_by_number(height) {
            Ok(block_opt) => {
                let response = crate::network::Message::GetBlockResponse(block_opt);
                network.broadcast(&response);
            }
            Err(e) => {
                eprintln!("[BlockProducer] Error retrieving block #{}: {}", height, e);
            }
        }
    }

    /// Handle incoming SyncRequest from peer
    pub fn handle_sync_request(&self, network: &Network, from_height: u64, to_height: u64) {
        match self.get_blocks_in_range(from_height, to_height) {
            Ok(blocks) => {
                println!("[BlockProducer] Responding with {} blocks for sync", blocks.len());
                let response = crate::network::Message::SyncResponse { blocks };
                network.broadcast(&response);
            }
            Err(e) => {
                eprintln!("[BlockProducer] Error retrieving blocks {}-{}: {}", from_height, to_height, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_producer_creation() {
        // Just verify we can create a block producer without panicking
        let metrics = Arc::new(Metrics::new().unwrap());
        let _producer = BlockProducer::new(
            Arc::new(TransactionMempool::new()),
            Arc::new(Db::open("test_db")),
            Arc::new(BlockchainIndexer::new()),
            metrics,
            1000,
        );
        // Cleanup
        let _ = std::fs::remove_dir_all("test_db");
    }
}

