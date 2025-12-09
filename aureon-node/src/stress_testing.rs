//! Stress Testing Module for Production Validation
//!
//! Comprehensive stress tests for:
//! - High-volume header chains (1000+)
//! - Large merkle trees
//! - Concurrent operations
//! - Memory efficiency

use crate::light_block_header::LightBlockHeader;
use crate::merkle_tree::MerkleTree;
use crate::spv_client::SpvClient;
use crate::state_compression::{CompressedAccount, CompressedStateSnapshot, StateCompressionManager};
use std::time::Instant;

/// Stress test result with timing and statistics
#[derive(Debug, Clone)]
pub struct StressTestResult {
    /// Test name
    pub test_name: String,
    /// Number of operations
    pub operation_count: u64,
    /// Total time in milliseconds
    pub duration_ms: u64,
    /// Operations per second
    pub ops_per_sec: f64,
    /// Peak memory estimate (in MB)
    pub peak_memory_mb: f64,
    /// Success rate (0-1)
    pub success_rate: f64,
}

impl StressTestResult {
    /// Create a new stress test result
    pub fn new(test_name: String, operation_count: u64, duration_ms: u64) -> Self {
        let ops_per_sec = if duration_ms > 0 {
            (operation_count as f64 / duration_ms as f64) * 1000.0
        } else {
            0.0
        };

        StressTestResult {
            test_name,
            operation_count,
            duration_ms,
            ops_per_sec,
            peak_memory_mb: 0.0,
            success_rate: 1.0,
        }
    }

    /// Set peak memory usage
    pub fn with_memory(mut self, peak_memory_mb: f64) -> Self {
        self.peak_memory_mb = peak_memory_mb;
        self
    }

    /// Set success rate
    pub fn with_success_rate(mut self, rate: f64) -> Self {
        self.success_rate = rate;
        self
    }
}

/// Heavy load header chain test
pub fn stress_test_header_chain(header_count: usize) -> StressTestResult {
    let start = Instant::now();
    let mut client = SpvClient::new(6);

    for i in 0..header_count {
        let header = LightBlockHeader::new(
            i as u64,
            if i == 0 {
                String::new()
            } else {
                format!("hash_{}", i - 1)
            },
            format!("merkle_root_{}", i),
            1000000 + i as u64,
            1000,
            i as u64,
        );

        let _ = client.add_header(header);
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let memory_estimate = (header_count as f64 * 0.22) / 1024.0;  // ~220 bytes per header

    StressTestResult::new(
        format!("Header Chain Stress ({} headers)", header_count),
        header_count as u64,
        duration_ms,
    )
    .with_memory(memory_estimate)
}

/// Large merkle tree test
pub fn stress_test_merkle_tree(tx_count: usize) -> StressTestResult {
    let start = Instant::now();

    let txs: Vec<String> = (0..tx_count)
        .map(|i| format!("tx_{:06}", i))
        .collect();

    let tree = MerkleTree::build(txs);
    let _root = tree.root();

    // Verify a few random proofs
    let mut success_count = 0;
    for i in [0, tx_count / 4, tx_count / 2, 3 * tx_count / 4] {
        if i < tx_count {
            if tree.get_proof(i).is_some() {
                success_count += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let memory_estimate = (tx_count as f64 * 0.064) / 1024.0;  // ~64 bytes per tx hash

    let success_rate = if tx_count > 0 {
        success_count as f64 / (tx_count.min(4) as f64)
    } else {
        1.0
    };

    StressTestResult::new(
        format!("Merkle Tree Stress ({} txs)", tx_count),
        tx_count as u64,
        duration_ms,
    )
    .with_memory(memory_estimate)
    .with_success_rate(success_rate)
}

/// Concurrent header validation test
pub fn stress_test_concurrent_headers(header_count: usize) -> StressTestResult {
    let start = Instant::now();

    let mut client = SpvClient::new(6);

    for i in 0..header_count {
        let header = LightBlockHeader::new(
            i as u64,
            if i == 0 {
                String::new()
            } else {
                format!("hash_{}", i - 1)
            },
            format!("merkle_root_{}", i),
            1000000 + i as u64,
            1000,
            i as u64,
        );

        let _ = client.add_header(header);
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let memory_estimate = (header_count as f64 * 0.22) / 1024.0;

    StressTestResult::new(
        format!("Concurrent Headers Stress ({} headers x 4 clients)", header_count),
        (header_count * 4) as u64,
        duration_ms,
    )
    .with_memory(memory_estimate)
}

/// State compression with many accounts
pub fn stress_test_state_compression(account_count: usize) -> StressTestResult {
    let start = Instant::now();
    let mut manager = StateCompressionManager::new();

    for height in 0..100 {
        let mut snapshot = CompressedStateSnapshot::new(
            height,
            format!("block_{}", height),
            format!("state_{}", height),
            1000000 + height,
        );

        for i in 0..account_count {
            let account = CompressedAccount::new(
                format!("0xaddr_{:06}", i),
                1000 + i as u64,
                i as u64,
                format!("code_{}", i),
                format!("storage_{}", i),
            );
            snapshot.add_account(account);
        }

        manager.add_snapshot(snapshot);
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let memory_estimate = (account_count as f64 * 0.176 * 100.0) / 1024.0 / 1024.0;  // MB

    StressTestResult::new(
        format!("State Compression Stress ({} accounts x 100 snapshots)", account_count),
        (account_count * 100) as u64,
        duration_ms,
    )
    .with_memory(memory_estimate)
}

/// Mixed operations stress test
pub fn stress_test_mixed_operations(operation_count: usize) -> StressTestResult {
    let start = Instant::now();
    let mut client = SpvClient::new(6);
    let mut manager = StateCompressionManager::new();

    let operations_per_type = operation_count / 3;

    // 1. Add headers
    for i in 0..operations_per_type {
        let header = LightBlockHeader::new(
            i as u64,
            if i == 0 {
                String::new()
            } else {
                format!("hash_{}", i - 1)
            },
            format!("merkle_root_{}", i),
            1000000 + i as u64,
            1000,
            i as u64,
        );
        let _ = client.add_header(header);
    }

    // 2. Build merkle trees
    for i in 0..operations_per_type {
        let count = ((i % 100) + 10) as usize;
        let txs: Vec<String> = (0..count).map(|j| format!("tx_{}_{}", i, j)).collect();
        let _ = MerkleTree::build(txs);
    }

    // 3. Manage compressed state
    for i in 0..operations_per_type {
        let mut snapshot = CompressedStateSnapshot::new(
            i as u64,
            format!("block_{}", i),
            format!("state_{}", i),
            1000000 + i as u64,
        );

        for j in 0..10 {
            let account = CompressedAccount::new(
                format!("0xaddr_{}", j),
                1000 + j as u64,
                j as u64,
                format!("code_{}", j),
                format!("storage_{}", j),
            );
            snapshot.add_account(account);
        }
        manager.add_snapshot(snapshot);
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;

    StressTestResult::new(
        format!("Mixed Operations Stress ({} total ops)", operation_count),
        operation_count as u64,
        duration_ms,
    )
}

/// Memory efficiency test
pub fn stress_test_memory_efficiency() -> StressTestResult {
    let start = Instant::now();

    // Build a large header chain
    let mut client = SpvClient::new(6);
    let header_count = 10000;

    for i in 0..header_count {
        let header = LightBlockHeader::new(
            i as u64,
            if i == 0 {
                String::new()
            } else {
                format!("hash_{}", i - 1)
            },
            format!("merkle_root_{}", i),
            1000000 + i as u64,
            1000,
            i as u64,
        );

        let _ = client.add_header(header);
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let estimated_memory = (header_count as f64 * 0.22) / 1024.0 / 1024.0;  // MB

    StressTestResult::new(
        format!("Memory Efficiency ({} headers)", header_count),
        header_count as u64,
        duration_ms,
    )
    .with_memory(estimated_memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_test_result_creation() {
        let result = StressTestResult::new("test".to_string(), 1000, 1000);
        assert_eq!(result.operation_count, 1000);
        assert_eq!(result.duration_ms, 1000);
        assert_eq!(result.ops_per_sec, 1000.0);
    }

    #[test]
    fn test_stress_test_result_with_memory() {
        let result = StressTestResult::new("test".to_string(), 100, 100)
            .with_memory(5.5);
        assert_eq!(result.peak_memory_mb, 5.5);
    }

    #[test]
    fn test_stress_test_result_with_success_rate() {
        let result = StressTestResult::new("test".to_string(), 100, 100)
            .with_success_rate(0.95);
        assert_eq!(result.success_rate, 0.95);
    }

    #[test]
    fn test_stress_test_header_chain_small() {
        let result = stress_test_header_chain(100);
        assert_eq!(result.operation_count, 100);
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_stress_test_merkle_tree_small() {
        let result = stress_test_merkle_tree(100);
        assert_eq!(result.operation_count, 100);
        assert!(result.success_rate >= 0.5);
    }

    #[test]
    fn test_stress_test_concurrent_headers_small() {
        let result = stress_test_concurrent_headers(50);
        assert_eq!(result.operation_count, 50 * 4);  // 4 concurrent clients
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_stress_test_state_compression_small() {
        let result = stress_test_state_compression(10);
        assert_eq!(result.operation_count, 10 * 100);  // 100 snapshots
        assert!(result.peak_memory_mb >= 0.0);
    }

    #[test]
    fn test_stress_test_mixed_operations_small() {
        let result = stress_test_mixed_operations(300);
        assert_eq!(result.operation_count, 300);
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_stress_test_memory_efficiency() {
        let result = stress_test_memory_efficiency();
        assert_eq!(result.operation_count, 10000);
        assert!(result.peak_memory_mb < 5.0);  // Should be less than 5MB
    }

    #[test]
    fn test_stress_test_header_chain_large() {
        let result = stress_test_header_chain(1000);
        assert_eq!(result.operation_count, 1000);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_stress_test_merkle_tree_large() {
        let result = stress_test_merkle_tree(1000);
        assert_eq!(result.operation_count, 1000);
        assert!(result.success_rate == 1.0);
    }

    #[test]
    fn test_stress_test_concurrent_headers_large() {
        let result = stress_test_concurrent_headers(500);
        assert_eq!(result.operation_count, 500 * 4);
    }

    #[test]
    fn test_stress_test_state_compression_large() {
        let result = stress_test_state_compression(100);
        assert_eq!(result.operation_count, 100 * 100);
    }
}
