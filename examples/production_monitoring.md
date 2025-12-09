# Example: Production Monitoring & Observability

Learn how to monitor, track metrics, and maintain visibility into Aureon production systems.

## Overview

This example demonstrates:
1. Latency tracking with percentiles
2. Error rate monitoring
3. Throughput measurement
4. Health dashboard generation
5. Detecting degradation

## Monitoring Components

### Latency Tracking

Track operation performance with percentile metrics:

```rust
use aureon_node::production_monitoring::LatencyTracker;

fn monitor_latencies() -> Result<(), String> {
    let mut tracker = LatencyTracker::new("block_processing");
    
    // Record some operations
    tracker.record_latency_us(1500);   // 1.5ms
    tracker.record_latency_us(2300);   // 2.3ms
    tracker.record_latency_us(980);    // 0.98ms
    tracker.record_latency_us(1200);   // 1.2ms
    tracker.record_latency_us(5500);   // 5.5ms (outlier)
    
    // Get statistics
    println!("Block Processing Latency:");
    println!("  Avg:  {}µs", tracker.average_latency());
    println!("  Min:  {}µs", tracker.min_latency());
    println!("  Max:  {}µs", tracker.max_latency());
    println!("  p95:  {}µs", tracker.p95_latency());
    println!("  p99:  {}µs", tracker.p99_latency());
    println!("  Samples: {}", tracker.sample_count());
    
    // Expected output:
    // Avg:  2396µs
    // Min:  980µs
    // Max:  5500µs
    // p95:  5016µs
    // p99:  5500µs
    // Samples: 5
    
    Ok(())
}
```

### Error Rate Tracking

Monitor error patterns and frequencies:

```rust
use aureon_node::production_monitoring::ErrorRateTracker;

fn monitor_errors() -> Result<(), String> {
    let mut tracker = ErrorRateTracker::new();
    
    // Simulate operations
    tracker.record_operation();  // success
    tracker.record_operation();  // success
    tracker.record_error("NetworkTimeout");
    tracker.record_operation();  // success
    tracker.record_error("NetworkTimeout");
    tracker.record_error("DatabaseError");
    tracker.record_operation();  // success
    
    // Get statistics (7 operations, 3 errors)
    println!("Error Monitoring:");
    println!("  Total operations: {}", tracker.total_operations());
    println!("  Total errors: {}", tracker.total_errors());
    println!("  Error rate: {:.1}%", tracker.error_rate() * 100.0);
    println!("  Success rate: {:.1}%", tracker.success_rate() * 100.0);
    println!("  Most common error: {:?}", tracker.most_common_error());
    
    // Expected output:
    // Total operations: 7
    // Total errors: 3
    // Error rate: 42.9%
    // Success rate: 57.1%
    // Most common error: Some("NetworkTimeout")
    
    Ok(())
}
```

### Throughput Measurement

Measure operations per second:

```rust
use aureon_node::production_monitoring::ThroughputTracker;
use std::thread;
use std::time::Duration;

fn measure_throughput() -> Result<(), String> {
    let mut tracker = ThroughputTracker::new();
    
    // Simulate 1000 operations
    for _ in 0..1000 {
        tracker.record_operation();
    }
    
    // Get throughput
    println!("Throughput Measurement:");
    println!("  Total operations: {}", tracker.total_operations());
    println!("  Overall ops/sec: {}", tracker.ops_per_sec());
    
    // Record ops with delays
    for _ in 0..500 {
        tracker.record_operation();
        thread::sleep(Duration::from_millis(1)); // 1ms per op
    }
    
    println!("  Last 10s throughput: {} ops/sec", 
        tracker.window_ops_per_sec(10));
    
    Ok(())
}
```

### Health Dashboard

Aggregate metrics from multiple operations:

```rust
use aureon_node::production_monitoring::HealthDashboard;

fn health_dashboard() -> Result<(), String> {
    let mut dashboard = HealthDashboard::new("aureon-node");
    
    // Record header addition operations
    for i in 0..10 {
        dashboard.record_latency("header_add", 100 + i * 10);
    }
    
    // Record occasional errors
    dashboard.record_operation("block_proposal");
    dashboard.record_operation("block_proposal");
    dashboard.record_error("consensus", "TooManyValidators");
    
    // Record throughput
    for _ in 0..100 {
        dashboard.record_operation("tx_validation");
    }
    
    // Update health status
    dashboard.update_health_status();
    
    // Generate report
    println!("{}", dashboard.generate_report());
    
    // Expected output:
    // ═══════════════════════════════════════
    // Service: aureon-node
    // Status: Healthy
    // ═══════════════════════════════════════
    // 
    // Operations:
    //   header_add: avg 150µs (p95: 196µs, p99: 199µs)
    //   block_proposal: 103 ops, error rate 0.97%
    //   tx_validation: 100 ops
    // 
    // Overall Metrics:
    //   Avg Latency: 1.2ms
    //   Error Rate: 0.97%
    //   Status: Healthy (<5% errors)
    // ═══════════════════════════════════════
    
    Ok(())
}
```

## Real-World Monitoring

### Production Node Monitoring

```rust
struct ProductionMonitor {
    dashboard: HealthDashboard,
    alert_threshold: f64,  // 5% error rate
}

impl ProductionMonitor {
    fn new(name: &str) -> Self {
        Self {
            dashboard: HealthDashboard::new(name),
            alert_threshold: 0.05,
        }
    }
    
    fn process_block(&mut self, block_height: u32) -> Result<(), String> {
        // Start timing
        let start = std::time::Instant::now();
        
        // Process block
        // ... block processing logic ...
        
        // Record latency
        let elapsed_us = start.elapsed().as_micros() as u64;
        self.dashboard.record_latency("block_process", elapsed_us);
        self.dashboard.record_operation("block_process");
        
        Ok(())
    }
    
    fn check_health(&mut self) -> Result<(), String> {
        // Update status
        self.dashboard.update_health_status();
        
        // Get error rate
        let error_rate = self.dashboard
            .error_tracker
            .error_rate();
        
        // Alert if degraded
        if error_rate > self.alert_threshold {
            println!("⚠️  ALERT: Error rate {}% exceeds {}%",
                error_rate * 100.0,
                self.alert_threshold * 100.0
            );
            
            // Take action
            // - Notify ops team
            // - Reduce load
            // - Enable circuit breaker
        }
        
        println!("{}", self.dashboard.generate_report());
        
        Ok(())
    }
}
```

### Stress Test Monitoring

```rust
use aureon_node::stress_testing::{stress_test_header_chain, StressTestResult};

fn monitor_stress_test() -> Result<(), String> {
    let result = stress_test_header_chain(1000)?;
    
    println!("Stress Test Results:");
    println!("  Operations: {}", result.operation_count);
    println!("  Duration: {}ms", result.duration_ms);
    println!("  Throughput: {:.0} ops/sec", result.ops_per_sec);
    println!("  Peak memory: {:.1}MB", result.peak_memory_mb);
    println!("  Success rate: {:.1}%", result.success_rate * 100.0);
    
    // Validate performance
    assert!(result.ops_per_sec > 10000.0, "Throughput too low");
    assert!(result.peak_memory_mb < 5.0, "Memory usage too high");
    assert!(result.success_rate > 0.99, "Success rate too low");
    
    println!("✓ Stress test passed!");
    
    Ok(())
}
```

## Monitoring in Production

### Metrics to Track

| Metric | Target | Alert |
|--------|--------|-------|
| Latency p95 | <5ms | >10ms |
| Latency p99 | <10ms | >20ms |
| Error rate | <1% | >5% |
| Throughput | >100 ops/sec | <50 ops/sec |
| Memory | <300MB | >500MB |

### Alert Examples

```rust
fn should_alert(dashboard: &HealthDashboard) -> bool {
    let error_rate = dashboard.error_tracker.error_rate();
    let avg_latency = dashboard.calculate_avg_latency();
    
    // Alert conditions
    error_rate > 0.05 ||              // >5% errors
    avg_latency > 10000.0 ||          // >10ms avg
    dashboard.status == HealthStatus::Unhealthy
}
```

### Dashboard Updates

```rust
fn update_dashboard_periodic(monitor: &mut ProductionMonitor) {
    loop {
        // Check health every 30 seconds
        monitor.check_health();
        
        // Print report
        // Notify monitoring system (Prometheus, Grafana)
        // Store metrics in time-series DB
        
        std::thread::sleep(Duration::from_secs(30));
    }
}
```

## Integration with Monitoring Systems

### Prometheus Export

```rust
fn prometheus_metrics(dashboard: &HealthDashboard) -> String {
    format!(
        "aureon_latency_avg_us {}\n",
        "aureon_error_rate {}\n",
        "aureon_throughput_ops_sec {}\n",
        "aureon_health_status {}\n",
    )
}
```

### Grafana Dashboards

```json
{
  "dashboard": {
    "title": "Aureon Node Metrics",
    "panels": [
      {
        "title": "Latency Percentiles",
        "targets": [
          "latency_p95_us",
          "latency_p99_us"
        ]
      },
      {
        "title": "Error Rate",
        "targets": ["error_rate"]
      },
      {
        "title": "Health Status",
        "targets": ["health_status"]
      }
    ]
  }
}
```

## Running Examples

```bash
# Test latency tracking
cargo test production_monitoring::tests::test_latency_tracker -- --nocapture

# Test error tracking
cargo test production_monitoring::tests::test_error_rate_tracker -- --nocapture

# Test health dashboard
cargo test production_monitoring::tests::test_health_dashboard -- --nocapture

# Run all monitoring tests
cargo test production_monitoring -- --nocapture
```

## Best Practices

### ✓ Do
- Track all critical operations
- Monitor error rates continuously
- Alert on degradation
- Use percentiles (p95, p99) not just averages
- Store historical metrics
- Review dashboards regularly
- Set realistic thresholds

### ✗ Don't
- Only monitor happy path
- Ignore small error rates
- Set alerts too tight (false alarms)
- Delete metrics too quickly
- Rely on single metrics
- Ignore latency outliers
- Skip stress testing in production

## Related Examples

- `token_transfer.md` - Monitor token operations
- `smart_contract.md` - Monitor contract execution
- `spv_light_client.md` - Monitor light client sync

## References

- **Latency Tracker**: `src/production_monitoring/latency_tracker.rs`
- **Error Tracker**: `src/production_monitoring/error_tracker.rs`
- **Throughput Tracker**: `src/production_monitoring/throughput_tracker.rs`
- **Health Dashboard**: `src/production_monitoring/health_dashboard.rs`
- **Stress Testing**: `src/stress_testing.rs`

## Monitoring Resources

- **Prometheus**: https://prometheus.io/
- **Grafana**: https://grafana.com/
- **OpenTelemetry**: https://opentelemetry.io/
- **Observability Best Practices**: https://www.oreilly.com/library/view/observability-engineering/
