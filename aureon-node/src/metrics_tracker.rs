/// Background task for periodically updating metrics based on system state
use crate::metrics::Metrics;
use crate::mempool::TransactionMempool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct MetricsTracker;

impl MetricsTracker {
    /// Start a background task that periodically updates metrics
    pub fn start_mempool_tracker(
        metrics: Arc<Metrics>,
        mempool: Arc<TransactionMempool>,
        interval_ms: u64,
    ) {
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(interval_ms));
                
                // Update mempool size metric
                if let Ok(size) = mempool.size() {
                    metrics.mempool_size.set(size as i64);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_tracker_creation() {
        let metrics = Arc::new(Metrics::new().unwrap());
        let mempool = Arc::new(TransactionMempool::new());
        
        // Just verify we can start without panicking
        MetricsTracker::start_mempool_tracker(metrics, mempool, 1000);
    }
}
