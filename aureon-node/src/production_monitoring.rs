//! Phase 10 Production Hardening Monitoring and Observability
//!
//! Comprehensive monitoring for production deployment:
//! - Performance metrics
//! - System health
//! - Latency tracking
//! - Error rates and recovery

use std::time::{Duration, SystemTime};
use std::collections::HashMap;

/// Request latency tracker
#[derive(Debug, Clone)]
pub struct LatencyTracker {
    /// Request name
    pub name: String,
    /// Sample of latencies (in microseconds)
    latencies: Vec<u64>,
    /// Max samples to keep
    max_samples: usize,
}

impl LatencyTracker {
    /// Create a new latency tracker
    pub fn new(name: String, max_samples: usize) -> Self {
        LatencyTracker {
            name,
            latencies: Vec::new(),
            max_samples,
        }
    }

    /// Record a latency measurement
    pub fn record_latency(&mut self, micros: u64) {
        self.latencies.push(micros);
        
        if self.latencies.len() > self.max_samples {
            self.latencies.remove(0);
        }
    }

    /// Get average latency in microseconds
    pub fn avg_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            0.0
        } else {
            let sum: u64 = self.latencies.iter().sum();
            sum as f64 / self.latencies.len() as f64
        }
    }

    /// Get 99th percentile latency
    pub fn p99_latency(&self) -> u64 {
        if self.latencies.is_empty() {
            0
        } else {
            let mut sorted = self.latencies.clone();
            sorted.sort_unstable();
            let index = (sorted.len() * 99) / 100;
            sorted[index.min(sorted.len() - 1)]
        }
    }

    /// Get 95th percentile latency
    pub fn p95_latency(&self) -> u64 {
        if self.latencies.is_empty() {
            0
        } else {
            let mut sorted = self.latencies.clone();
            sorted.sort_unstable();
            let index = (sorted.len() * 95) / 100;
            sorted[index.min(sorted.len() - 1)]
        }
    }

    /// Get maximum latency
    pub fn max_latency(&self) -> u64 {
        self.latencies.iter().copied().max().unwrap_or(0)
    }

    /// Get minimum latency
    pub fn min_latency(&self) -> u64 {
        self.latencies.iter().copied().min().unwrap_or(0)
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.latencies.len()
    }
}

/// Error rate tracker
#[derive(Debug, Clone)]
pub struct ErrorRateTracker {
    /// Total operations
    pub total_operations: u64,
    /// Total errors
    pub total_errors: u64,
    /// Errors by type
    pub error_types: HashMap<String, u64>,
    /// Last error timestamp
    pub last_error: Option<SystemTime>,
}

impl ErrorRateTracker {
    /// Create a new error rate tracker
    pub fn new() -> Self {
        ErrorRateTracker {
            total_operations: 0,
            total_errors: 0,
            error_types: HashMap::new(),
            last_error: None,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.total_operations += 1;
    }

    /// Record an error
    pub fn record_error(&mut self, error_type: String) {
        self.total_operations += 1;
        self.total_errors += 1;
        self.last_error = Some(SystemTime::now());
        
        *self.error_types.entry(error_type).or_insert(0) += 1;
    }

    /// Get error rate (0-1)
    pub fn error_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_errors as f64 / self.total_operations as f64
        }
    }

    /// Get success rate (0-1)
    pub fn success_rate(&self) -> f64 {
        1.0 - self.error_rate()
    }

    /// Get most common error type
    pub fn most_common_error(&self) -> Option<(String, u64)> {
        self.error_types
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(k, v)| (k.clone(), *v))
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.total_operations = 0;
        self.total_errors = 0;
        self.error_types.clear();
        self.last_error = None;
    }
}

impl Default for ErrorRateTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Throughput tracker
#[derive(Debug, Clone)]
pub struct ThroughputTracker {
    /// Operations processed
    pub operations: u64,
    /// Start time
    pub start_time: SystemTime,
    /// Last measurement time
    pub last_measurement: SystemTime,
    /// Recent window throughputs
    window_measurements: Vec<(SystemTime, u64)>,
}

impl ThroughputTracker {
    /// Create a new throughput tracker
    pub fn new() -> Self {
        let now = SystemTime::now();
        ThroughputTracker {
            operations: 0,
            start_time: now,
            last_measurement: now,
            window_measurements: Vec::new(),
        }
    }

    /// Record an operation
    pub fn record_operation(&mut self) {
        self.operations += 1;
    }

    /// Record N operations
    pub fn record_operations(&mut self, count: u64) {
        self.operations += count;
    }

    /// Get current ops/sec
    pub fn ops_per_sec(&self) -> f64 {
        if let Ok(elapsed) = self.start_time.elapsed() {
            let seconds = elapsed.as_secs_f64();
            if seconds > 0.0 {
                self.operations as f64 / seconds
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get window ops/sec (last N seconds)
    pub fn window_ops_per_sec(&mut self, window_secs: u64) -> f64 {
        let now = SystemTime::now();
        self.window_measurements.push((now, self.operations));
        
        // Remove old measurements outside window
        let cutoff = now - Duration::from_secs(window_secs);
        self.window_measurements.retain(|(t, _)| *t > cutoff);
        
        if self.window_measurements.len() < 2 {
            return 0.0;
        }

        let first = self.window_measurements[0];
        let last = self.window_measurements[self.window_measurements.len() - 1];
        
        if let Ok(elapsed) = last.0.duration_since(first.0) {
            let seconds = elapsed.as_secs_f64();
            let ops_delta = last.1.saturating_sub(first.1);
            if seconds > 0.0 {
                ops_delta as f64 / seconds
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.operations = 0;
        self.start_time = SystemTime::now();
        self.last_measurement = SystemTime::now();
        self.window_measurements.clear();
    }
}

impl Default for ThroughputTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

/// Comprehensive service health dashboard
#[derive(Debug, Clone)]
pub struct HealthDashboard {
    /// Service name
    pub service_name: String,
    /// Health status
    pub status: HealthStatus,
    /// Latency trackers by operation
    pub latencies: HashMap<String, LatencyTracker>,
    /// Error rate tracker
    pub error_tracker: ErrorRateTracker,
    /// Throughput tracker
    pub throughput: ThroughputTracker,
    /// Last updated
    pub last_updated: SystemTime,
}

impl HealthDashboard {
    /// Create a new health dashboard
    pub fn new(service_name: String) -> Self {
        HealthDashboard {
            service_name,
            status: HealthStatus::Healthy,
            latencies: HashMap::new(),
            error_tracker: ErrorRateTracker::new(),
            throughput: ThroughputTracker::new(),
            last_updated: SystemTime::now(),
        }
    }

    /// Add or get latency tracker for operation
    pub fn get_or_create_tracker(&mut self, operation: String) -> &mut LatencyTracker {
        self.latencies
            .entry(operation.clone())
            .or_insert_with(|| LatencyTracker::new(operation, 1000))
    }

    /// Update health status based on metrics
    pub fn update_health_status(&mut self) {
        self.last_updated = SystemTime::now();

        let error_rate = self.error_tracker.error_rate();
        
        if error_rate > 0.1 {
            self.status = HealthStatus::Unhealthy;
        } else if error_rate > 0.05 {
            self.status = HealthStatus::Degraded;
        } else {
            self.status = HealthStatus::Healthy;
        }
    }

    /// Get average latency across all operations
    pub fn avg_latency_all(&self) -> f64 {
        if self.latencies.is_empty() {
            0.0
        } else {
            let sum: f64 = self.latencies.values().map(|t| t.avg_latency()).sum();
            sum / self.latencies.len() as f64
        }
    }

    /// Generate health report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("=== Health Report: {} ===\n", self.service_name));
        report.push_str(&format!("Status: {:?}\n", self.status));
        report.push_str(&format!("Error Rate: {:.2}%\n", self.error_tracker.error_rate() * 100.0));
        report.push_str(&format!("Success Rate: {:.2}%\n", self.error_tracker.success_rate() * 100.0));
        report.push_str(&format!("Throughput: {:.2} ops/sec\n", self.throughput.ops_per_sec()));
        report.push_str(&format!("Avg Latency: {:.2} µs\n", self.avg_latency_all()));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_tracker_creation() {
        let tracker = LatencyTracker::new("test".to_string(), 100);
        assert_eq!(tracker.sample_count(), 0);
    }

    #[test]
    fn test_latency_tracker_record() {
        let mut tracker = LatencyTracker::new("test".to_string(), 100);
        tracker.record_latency(100);
        tracker.record_latency(200);
        tracker.record_latency(300);

        assert_eq!(tracker.sample_count(), 3);
        assert_eq!(tracker.avg_latency(), 200.0);
    }

    #[test]
    fn test_latency_tracker_percentiles() {
        let mut tracker = LatencyTracker::new("test".to_string(), 100);
        
        for i in 0..100 {
            tracker.record_latency(i as u64);
        }

        assert!(tracker.p99_latency() >= tracker.p95_latency());
        assert!(tracker.max_latency() >= tracker.p99_latency());
    }

    #[test]
    fn test_error_rate_tracker_creation() {
        let tracker = ErrorRateTracker::new();
        assert_eq!(tracker.error_rate(), 0.0);
    }

    #[test]
    fn test_error_rate_tracker_record() {
        let mut tracker = ErrorRateTracker::new();
        tracker.record_success();
        tracker.record_success();
        tracker.record_error("type1".to_string());

        assert_eq!(tracker.total_operations, 3);
        assert_eq!(tracker.total_errors, 1);
        assert!((tracker.error_rate() - 1.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_error_rate_tracker_most_common() {
        let mut tracker = ErrorRateTracker::new();
        tracker.record_error("type1".to_string());
        tracker.record_error("type1".to_string());
        tracker.record_error("type2".to_string());

        let (error_type, count) = tracker.most_common_error().unwrap();
        assert_eq!(error_type, "type1");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_throughput_tracker_creation() {
        let tracker = ThroughputTracker::new();
        assert_eq!(tracker.operations, 0);
    }

    #[test]
    fn test_throughput_tracker_record() {
        let mut tracker = ThroughputTracker::new();
        tracker.record_operation();
        tracker.record_operation();
        tracker.record_operations(3);

        assert_eq!(tracker.operations, 5);
    }

    #[test]
    fn test_health_dashboard_creation() {
        let dashboard = HealthDashboard::new("test_service".to_string());
        assert_eq!(dashboard.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_dashboard_update_status() {
        let mut dashboard = HealthDashboard::new("test_service".to_string());
        
        for _ in 0..10 {
            dashboard.error_tracker.record_error("type1".to_string());
        }

        dashboard.update_health_status();
        assert_eq!(dashboard.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_dashboard_tracker_access() {
        let mut dashboard = HealthDashboard::new("test_service".to_string());
        
        {
            let tracker = dashboard.get_or_create_tracker("op1".to_string());
            tracker.record_latency(100);
        }

        assert_eq!(dashboard.latencies.len(), 1);
    }

    #[test]
    fn test_health_dashboard_report() {
        let dashboard = HealthDashboard::new("test_service".to_string());
        let report = dashboard.generate_report();
        
        assert!(report.contains("Health Report"));
        assert!(report.contains("test_service"));
    }

    #[test]
    fn test_error_rate_tracker_reset() {
        let mut tracker = ErrorRateTracker::new();
        tracker.record_error("type1".to_string());
        
        tracker.reset();
        assert_eq!(tracker.total_errors, 0);
        assert_eq!(tracker.total_operations, 0);
    }

    #[test]
    fn test_throughput_tracker_reset() {
        let mut tracker = ThroughputTracker::new();
        tracker.record_operations(100);
        
        tracker.reset();
        assert_eq!(tracker.operations, 0);
    }

    #[test]
    fn test_latency_tracker_max_samples() {
        let mut tracker = LatencyTracker::new("test".to_string(), 5);
        
        for i in 0..10 {
            tracker.record_latency(i as u64);
        }

        assert_eq!(tracker.sample_count(), 5);
    }
}
