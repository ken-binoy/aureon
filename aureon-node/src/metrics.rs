use prometheus::{
    Counter, GaugeVec, HistogramOpts, HistogramVec, IntCounter,
    IntCounterVec, IntGauge, Opts, Registry, TextEncoder, Encoder,
};
use std::sync::Arc;

/// Prometheus metrics registry and counters
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,

    // Block metrics
    pub blocks_produced: IntCounter,
    pub blocks_received: IntCounter,
    pub block_production_time: HistogramVec,

    // Transaction metrics
    pub transactions_submitted: IntCounter,
    pub transactions_processed: IntCounter,
    pub transactions_failed: IntCounter,
    pub mempool_size: IntGauge,

    // Consensus metrics
    pub consensus_rounds: IntCounter,
    pub consensus_round_time: HistogramVec,
    pub pow_difficulty: IntGauge,
    pub pos_validators: IntGauge,

    // Network metrics
    pub peers_connected: IntGauge,
    pub messages_sent: IntCounterVec,
    pub messages_received: IntCounterVec,
    pub peer_heights: GaugeVec,

    // State metrics
    pub chain_height: IntGauge,
    pub state_root_updates: IntCounter,
    pub account_count: IntGauge,

    // API metrics
    pub http_requests: IntCounterVec,
    pub http_request_duration: HistogramVec,
    pub http_errors: IntCounterVec,

    // Smart contract metrics
    pub contracts_deployed: IntCounter,
    pub contract_invocations: IntCounter,
    pub contract_execution_time: HistogramVec,
    pub contract_gas_used: Counter,

    // Database metrics
    pub db_operations: IntCounterVec,
    pub db_operation_time: HistogramVec,
    pub db_key_count: IntGauge,
}

impl Metrics {
    /// Create new metrics registry
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // Block metrics
        let blocks_produced = IntCounter::new("blocks_produced_total", "Total blocks produced")?;
        let blocks_received = IntCounter::new("blocks_received_total", "Total blocks received")?;
        let block_production_time = HistogramVec::new(
            HistogramOpts::new("block_production_time_seconds", "Block production time"),
            &["type"],
        )?;

        // Transaction metrics
        let transactions_submitted =
            IntCounter::new("transactions_submitted_total", "Total transactions submitted")?;
        let transactions_processed =
            IntCounter::new("transactions_processed_total", "Total transactions processed")?;
        let transactions_failed =
            IntCounter::new("transactions_failed_total", "Total failed transactions")?;
        let mempool_size = IntGauge::new("mempool_size", "Current mempool size")?;

        // Consensus metrics
        let consensus_rounds =
            IntCounter::new("consensus_rounds_total", "Total consensus rounds completed")?;
        let consensus_round_time = HistogramVec::new(
            HistogramOpts::new(
                "consensus_round_time_seconds",
                "Consensus round completion time",
            ),
            &["engine"],
        )?;
        let pow_difficulty = IntGauge::new("pow_difficulty", "Current PoW difficulty")?;
        let pos_validators = IntGauge::new("pos_validators_count", "Number of PoS validators")?;

        // Network metrics
        let peers_connected = IntGauge::new("peers_connected", "Number of connected peers")?;
        let messages_sent =
            IntCounterVec::new(Opts::new("messages_sent_total", "Total messages sent"), &["type"])?;
        let messages_received = IntCounterVec::new(
            Opts::new("messages_received_total", "Total messages received"),
            &["type"],
        )?;
        let peer_heights = GaugeVec::new(
            Opts::new("peer_heights", "Height of connected peers"),
            &["peer_id"],
        )?;

        // State metrics
        let chain_height = IntGauge::new("chain_height", "Current blockchain height")?;
        let state_root_updates =
            IntCounter::new("state_root_updates_total", "Total state root updates")?;
        let account_count = IntGauge::new("account_count", "Number of accounts in state")?;

        // API metrics
        let http_requests = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"],
        )?;
        let http_request_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration"),
            &["method", "path"],
        )?;
        let http_errors = IntCounterVec::new(
            Opts::new("http_errors_total", "Total HTTP errors"),
            &["path", "status"],
        )?;

        // Smart contract metrics
        let contracts_deployed =
            IntCounter::new("contracts_deployed_total", "Total contracts deployed")?;
        let contract_invocations =
            IntCounter::new("contract_invocations_total", "Total contract invocations")?;
        let contract_execution_time = HistogramVec::new(
            HistogramOpts::new("contract_execution_time_seconds", "Contract execution time"),
            &["contract"],
        )?;
        let contract_gas_used =
            Counter::new("contract_gas_used_total", "Total gas used by contracts")?;

        // Database metrics
        let db_operations = IntCounterVec::new(
            Opts::new("db_operations_total", "Total database operations"),
            &["type"],
        )?;
        let db_operation_time = HistogramVec::new(
            HistogramOpts::new("db_operation_time_seconds", "Database operation time"),
            &["type"],
        )?;
        let db_key_count = IntGauge::new("db_key_count", "Number of keys in database")?;

        // Register all metrics
        registry.register(Box::new(blocks_produced.clone()))?;
        registry.register(Box::new(blocks_received.clone()))?;
        registry.register(Box::new(block_production_time.clone()))?;

        registry.register(Box::new(transactions_submitted.clone()))?;
        registry.register(Box::new(transactions_processed.clone()))?;
        registry.register(Box::new(transactions_failed.clone()))?;
        registry.register(Box::new(mempool_size.clone()))?;

        registry.register(Box::new(consensus_rounds.clone()))?;
        registry.register(Box::new(consensus_round_time.clone()))?;
        registry.register(Box::new(pow_difficulty.clone()))?;
        registry.register(Box::new(pos_validators.clone()))?;

        registry.register(Box::new(peers_connected.clone()))?;
        registry.register(Box::new(messages_sent.clone()))?;
        registry.register(Box::new(messages_received.clone()))?;
        registry.register(Box::new(peer_heights.clone()))?;

        registry.register(Box::new(chain_height.clone()))?;
        registry.register(Box::new(state_root_updates.clone()))?;
        registry.register(Box::new(account_count.clone()))?;

        registry.register(Box::new(http_requests.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_errors.clone()))?;

        registry.register(Box::new(contracts_deployed.clone()))?;
        registry.register(Box::new(contract_invocations.clone()))?;
        registry.register(Box::new(contract_execution_time.clone()))?;
        registry.register(Box::new(contract_gas_used.clone()))?;

        registry.register(Box::new(db_operations.clone()))?;
        registry.register(Box::new(db_operation_time.clone()))?;
        registry.register(Box::new(db_key_count.clone()))?;

        Ok(Metrics {
            registry: Arc::new(registry),
            blocks_produced,
            blocks_received,
            block_production_time,
            transactions_submitted,
            transactions_processed,
            transactions_failed,
            mempool_size,
            consensus_rounds,
            consensus_round_time,
            pow_difficulty,
            pos_validators,
            peers_connected,
            messages_sent,
            messages_received,
            peer_heights,
            chain_height,
            state_root_updates,
            account_count,
            http_requests,
            http_request_duration,
            http_errors,
            contracts_deployed,
            contract_invocations,
            contract_execution_time,
            contract_gas_used,
            db_operations,
            db_operation_time,
            db_key_count,
        })
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap_or_default())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();
        assert!(metrics.registry.gather().len() > 0);
    }

    #[test]
    fn test_metrics_export() {
        let metrics = Metrics::new().unwrap();
        metrics.blocks_produced.inc();
        let output = metrics.export().unwrap();
        assert!(output.contains("blocks_produced_total"));
    }

    #[test]
    fn test_counter_increment() {
        let metrics = Metrics::new().unwrap();
        metrics.blocks_produced.inc();
        metrics.blocks_produced.inc_by(5);
        // Verify by exporting
        let output = metrics.export().unwrap();
        assert!(output.contains("blocks_produced_total 6"));
    }

    #[test]
    fn test_gauge_set() {
        let metrics = Metrics::new().unwrap();
        metrics.chain_height.set(42);
        let output = metrics.export().unwrap();
        assert!(output.contains("chain_height 42"));
    }

    #[test]
    fn test_histogram_observe() {
        let metrics = Metrics::new().unwrap();
        let timer = metrics.block_production_time.with_label_values(&["pow"]).start_timer();
        timer.observe_duration();
        let output = metrics.export().unwrap();
        assert!(output.contains("block_production_time_seconds_bucket"));
    }
}
