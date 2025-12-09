# Monitoring & Observability Guide

Complete guide to monitoring Aureon blockchain nodes in production.

## Table of Contents

1. [Monitoring Architecture](#monitoring-architecture)
2. [Health Checks](#health-checks)
3. [Metrics Collection](#metrics-collection)
4. [Logging](#logging)
5. [Alerting](#alerting)
6. [Dashboards](#dashboards)
7. [Troubleshooting](#troubleshooting)

## Monitoring Architecture

```
Aureon Nodes (3 instances)
    ↓
Metrics Collection (Prometheus)
    ↓
Time-Series Database
    ↓
Visualization (Grafana)
    ↓
Alerting (AlertManager)
```

## Health Checks

### Built-in Health Endpoint

```bash
# Check node status
curl http://localhost:8080/chain/head

# Response format
{
  "height": 100,
  "hash": "0xabc123...",
  "timestamp": 1702000000,
  "validator": "0x...",
  "state_root": "0x..."
}
```

### Health Check Script

```bash
#!/bin/bash
# health_check.sh

NODE_URL="http://localhost:8080"
HEALTH_LOG="/var/aureon/logs/health.log"

# Function to check node
check_node() {
    RESPONSE=$(curl -s "${NODE_URL}/chain/head")
    
    if [ $? -ne 0 ]; then
        echo "[ERROR] Node unreachable" | tee -a "${HEALTH_LOG}"
        return 1
    fi
    
    # Extract height
    HEIGHT=$(echo "$RESPONSE" | jq -r '.height')
    
    if [ "$HEIGHT" -lt 0 ]; then
        echo "[ERROR] Invalid height: $HEIGHT" | tee -a "${HEALTH_LOG}"
        return 1
    fi
    
    echo "[OK] Block height: $HEIGHT" | tee -a "${HEALTH_LOG}"
    return 0
}

# Run check every 30 seconds
while true; do
    check_node
    sleep 30
done
```

### Systemd Health Check

```ini
[Unit]
Description=Aureon Blockchain Node
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/aureon-node --config /etc/aureon/config.toml
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Health check
ExecStartPost=/usr/local/bin/aureon-healthcheck
ExecReload=/bin/kill -HUP $MAINPID

[Install]
WantedBy=multi-user.target
```

## Metrics Collection

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'mainnet'

scrape_configs:
  # Aureon nodes
  - job_name: 'aureon-nodes'
    static_configs:
      - targets:
        - 'node0:8080'
        - 'node1:8080'
        - 'node2:8080'
        labels:
          environment: 'production'
    
    # Custom metrics
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s
    
    # Relabel
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
```

### Metrics Available

From Aureon Health Dashboard:

```
aureon_block_height           # Current block height
aureon_block_time             # Block creation timestamp
aureon_state_root             # State tree root hash
aureon_validator_count        # Active validators
aureon_pending_transactions   # Mempool size
aureon_latency_ms_avg         # Average latency
aureon_latency_ms_p95         # 95th percentile latency
aureon_latency_ms_p99         # 99th percentile latency
aureon_error_rate             # Current error rate
aureon_error_count            # Total errors
aureon_throughput_ops_sec     # Operations per second
aureon_health_status          # 1=Healthy, 0=Degraded, -1=Unhealthy
aureon_circuit_breaker_state  # 1=Closed, 0=Open, -1=HalfOpen
```

### Export Metrics

```rust
// src/metrics.rs
use std::collections::HashMap;

pub struct MetricsExporter {
    metrics: HashMap<String, f64>,
}

impl MetricsExporter {
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        for (key, value) in &self.metrics {
            output.push_str(&format!("{} {}\n", key, value));
        }
        
        output
    }
}
```

## Logging

### Log Levels

```
TRACE - Very detailed debugging
DEBUG - Detailed debugging info
INFO  - Important operational info
WARN  - Warning conditions
ERROR - Error conditions
```

### Logging Configuration

```toml
# config.toml
[logging]
level = "info"
format = "json"  # json or text
log_file = "/var/aureon/logs/node.log"
max_file_size = "100MB"
max_backups = 10
max_age_days = 30
```

### Enable Logging

```bash
# Run with debug logging
RUST_LOG=debug ./aureon-node --config config.toml

# Specific module
RUST_LOG=aureon_node::consensus=debug ./aureon-node

# Multiple modules
RUST_LOG=aureon_node=info,aureon_node::wasm=debug ./aureon-node
```

### Log Parsing

```bash
# Count log levels
cat /var/aureon/logs/node.log | grep -o '\[.*\]' | sort | uniq -c

# Find errors
grep ERROR /var/aureon/logs/node.log

# Find warnings
grep WARN /var/aureon/logs/node.log

# Time-based search
grep "2024-01-15" /var/aureon/logs/node.log

# Extract metrics from logs
grep "block_height" /var/aureon/logs/node.log
```

## Alerting

### Alert Rules

```yaml
# alerting_rules.yml
groups:
  - name: aureon
    interval: 30s
    rules:
      # Node down
      - alert: NodeDown
        expr: up{job="aureon-nodes"} == 0
        for: 2m
        annotations:
          summary: "Aureon node {{ $labels.instance }} is down"
          
      # High error rate
      - alert: HighErrorRate
        expr: aureon_error_rate > 0.05
        for: 5m
        annotations:
          summary: "Error rate above 5%"
          
      # Circuit breaker open
      - alert: CircuitBreakerOpen
        expr: aureon_circuit_breaker_state == 0
        for: 1m
        annotations:
          summary: "Circuit breaker opened on {{ $labels.instance }}"
          
      # High latency
      - alert: HighLatency
        expr: aureon_latency_ms_p99 > 10
        for: 5m
        annotations:
          summary: "P99 latency above 10ms"
          
      # Disk space
      - alert: DiskSpaceWarning
        expr: node_filesystem_avail_bytes{mountpoint="/var/aureon"} < 5368709120
        for: 10m
        annotations:
          summary: "Less than 5GB disk space available"
          
      # Memory pressure
      - alert: HighMemoryUsage
        expr: aureon_memory_usage_bytes > 2147483648
        for: 5m
        annotations:
          summary: "Memory usage above 2GB"
```

### AlertManager Configuration

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m

route:
  receiver: 'default'
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  routes:
    - match:
        severity: critical
      receiver: 'ops-team'
      continue: true
      
    - match:
        severity: warning
      receiver: 'notifications'

receivers:
  - name: 'default'
    
  - name: 'ops-team'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK'
        channel: '#aureon-ops'
        
  - name: 'notifications'
    email_configs:
      - to: 'ops@example.com'
        from: 'alerts@example.com'
        smarthost: 'smtp.example.com:587'
```

## Dashboards

### Grafana Dashboard JSON

```json
{
  "dashboard": {
    "title": "Aureon Node Monitoring",
    "panels": [
      {
        "title": "Block Height",
        "targets": [
          {"expr": "aureon_block_height"}
        ],
        "type": "gauge"
      },
      {
        "title": "Latency Distribution",
        "targets": [
          {"expr": "aureon_latency_ms_avg"},
          {"expr": "aureon_latency_ms_p95"},
          {"expr": "aureon_latency_ms_p99"}
        ],
        "type": "graph"
      },
      {
        "title": "Error Rate",
        "targets": [
          {"expr": "aureon_error_rate * 100"}
        ],
        "type": "gauge",
        "thresholds": "0,5,10"
      },
      {
        "title": "Health Status",
        "targets": [
          {"expr": "aureon_health_status"}
        ],
        "type": "stat",
        "colorMode": "background"
      },
      {
        "title": "Validator Count",
        "targets": [
          {"expr": "aureon_validator_count"}
        ],
        "type": "stat"
      },
      {
        "title": "Throughput (ops/sec)",
        "targets": [
          {"expr": "aureon_throughput_ops_sec"}
        ],
        "type": "graph"
      }
    ]
  }
}
```

### Key Dashboard Metrics

1. **System Health**
   - Block height (gauge)
   - Health status (indicator)
   - Active validators (number)

2. **Performance**
   - Latency p95/p99 (graph)
   - Throughput ops/sec (graph)
   - Block time (gauge)

3. **Errors**
   - Error rate % (gauge)
   - Error count (counter)
   - Circuit breaker status (indicator)

4. **Resources**
   - Memory usage (gauge)
   - CPU usage (graph)
   - Disk space (gauge)

## Troubleshooting

### High Error Rate

```bash
# 1. Check logs for errors
grep ERROR /var/aureon/logs/node.log | tail -20

# 2. Check node connectivity
nc -zv bootstrap.example.com 6000

# 3. Check resource usage
ps aux | grep aureon-node
free -h
df -h

# 4. Check configuration
cat /etc/aureon/config.toml | grep -A5 network

# 5. Increase log level
systemctl stop aureon-node
RUST_LOG=debug aureon-node --config config.toml
```

### High Latency

```bash
# 1. Check CPU usage
top -p $(pgrep aureon-node)

# 2. Check disk I/O
iostat -x 1 10

# 3. Check network
ping bootstrap.example.com

# 4. Check database performance
ls -lh /var/aureon/db/*

# 5. Review database cache setting
grep cache_size /etc/aureon/config.toml
```

### Memory Issues

```bash
# 1. Check memory usage
ps -o pid,vsz,rss,comm= | grep aureon

# 2. Check for memory leaks
valgrind ./aureon-node --config config.toml

# 3. Reduce cache size
sed -i 's/cache_size_mb = .*/cache_size_mb = 128/' config.toml

# 4. Enable compression
echo "compression = true" >> config.toml

# 5. Restart node
systemctl restart aureon-node
```

### Network Issues

```bash
# 1. Check if node is listening
netstat -tlnp | grep aureon

# 2. Test connectivity
curl -v http://localhost:8080/chain/head

# 3. Check firewall
sudo ufw status

# 4. Check DNS
nslookup bootstrap.example.com

# 5. Check peer count
curl http://localhost:8080/peers
```

## Monitoring Checklist

- [ ] Health checks running every 5 minutes
- [ ] Prometheus scraping metrics every 15 seconds
- [ ] Grafana dashboards created and shared
- [ ] Alert rules configured for critical issues
- [ ] AlertManager sending notifications
- [ ] Log aggregation set up
- [ ] Backup and recovery tested
- [ ] Team trained on monitoring
- [ ] Runbooks created for common issues
- [ ] SLAs defined and tracked

## Performance Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Error Rate | >2% | >5% |
| Latency p95 | >5ms | >10ms |
| Latency p99 | >10ms | >20ms |
| Memory | >1.5GB | >2GB |
| CPU | >75% | >90% |
| Disk | <10GB | <5GB |

---

**Related Guides**:
- `DEPLOYMENT.md` - Node deployment
- `TROUBLESHOOTING.md` - Common issues
- `examples/production_monitoring.md` - Monitoring examples
