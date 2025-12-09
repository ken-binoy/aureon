# Aureon Blockchain Monitoring

This directory contains Grafana dashboard templates and monitoring configuration for the Aureon blockchain.

## Overview

The Aureon blockchain exposes Prometheus metrics on the `/metrics` endpoint (default: `http://localhost:8080/metrics`). The dashboards in this directory visualize these metrics in real-time.

## Dashboards

### 1. aureon-metrics-dashboard.json
**Main blockchain metrics dashboard**

Displays:
- Blocks produced (gauge)
- Block production rate (5-minute moving average)
- Transaction metrics (submitted, processed, failed)
- Mempool size with capacity thresholds
- Chain height
- Network peer count

**Use Case**: Monitor overall blockchain health, transaction throughput, and network connectivity.

### 2. aureon-consensus-dashboard.json
**Consensus engine performance dashboard**

Displays:
- Consensus round completion rate
- PoW difficulty tracking
- PoS active validators count
- Consensus round time (p95, p99 percentiles)
- Block production time distribution
- Database operation metrics

**Use Case**: Monitor consensus performance, engine efficiency, and database I/O.

### 3. aureon-api-network-dashboard.json
**API and network activity dashboard**

Displays:
- HTTP request rate by method and path
- HTTP error rate and status codes
- API request duration (p95 percentiles)
- Network message count (sent/received)
- Smart contract deployment and invocation rates
- Contract gas consumption tracking

**Use Case**: Monitor API usage, network activity, and smart contract execution.

## Metrics Reference

### Block Metrics
- `blocks_produced_total` - Total blocks produced by this node
- `blocks_received_total` - Total blocks received from network
- `block_production_time_seconds` - Histogram of block production time

### Transaction Metrics
- `transactions_submitted_total` - Total transactions submitted to mempool
- `transactions_processed_total` - Total transactions processed in blocks
- `transactions_failed_total` - Total transaction failures
- `mempool_size` - Current size of transaction mempool

### Consensus Metrics
- `consensus_rounds_total` - Total completed consensus rounds
- `consensus_round_time_seconds` - Histogram of consensus round time
- `pow_difficulty` - Current PoW difficulty setting
- `pos_validators_count` - Number of active PoS validators

### Network Metrics
- `peers_connected` - Number of connected peer nodes
- `messages_sent_total` - Total messages sent (by type)
- `messages_received_total` - Total messages received (by type)
- `peer_heights` - Height of connected peers

### State Metrics
- `chain_height` - Current blockchain height
- `state_root_updates_total` - Total state root updates
- `account_count` - Number of accounts in state

### API Metrics
- `http_requests_total` - Total HTTP requests (by method, path, status)
- `http_request_duration_seconds` - Histogram of HTTP request duration
- `http_errors_total` - Total HTTP errors (by path, status)

### Contract Metrics
- `contracts_deployed_total` - Total smart contracts deployed
- `contract_invocations_total` - Total contract function invocations
- `contract_execution_time_seconds` - Histogram of contract execution time
- `contract_gas_used_total` - Total gas consumed by contracts

### Database Metrics
- `db_operations_total` - Total database operations (by type)
- `db_operation_time_seconds` - Histogram of database operation time
- `db_key_count` - Number of keys in database

## Setup Instructions

### Prerequisites
- Grafana (version 8.0 or higher)
- Prometheus (configured to scrape Aureon node metrics)
- Aureon node running with metrics enabled (default port 8080)

### 1. Configure Prometheus

Add to your `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'aureon-node'
    static_configs:
      - targets: ['localhost:8080']
```

### 2. Import Dashboards into Grafana

1. Open Grafana UI (default: http://localhost:3000)
2. Go to **Dashboards** → **Import**
3. Either:
   - Upload JSON file from this directory, or
   - Paste the JSON content directly
4. Select Prometheus as the data source
5. Click **Import**

### 3. Verify Metrics Collection

Check Prometheus targets at: `http://localhost:9090/targets`

Verify metrics are being scraped: `http://localhost:9090/api/v1/query?query=blocks_produced_total`

## Dashboard Customization

You can customize the dashboards by:
1. Cloning a dashboard in Grafana UI
2. Editing panels to add custom queries
3. Exporting the modified dashboard as JSON

### Common Customizations

**Change refresh rate:**
Edit the top of each JSON file:
```json
"refresh": "10s"  // Change to "30s", "1m", etc.
```

**Adjust time range:**
Edit the `time` object:
```json
"time": {
  "from": "now-1h",  // Change to "now-24h", "now-7d", etc.
  "to": "now"
}
```

**Add alerting rules:**
In Grafana, click the alarm icon on any panel to set alert thresholds.

## Performance Tips

- For large clusters (50+ nodes), consider increasing scrape interval to 30s
- Use sampling for high-frequency metrics in long-term storage
- Set up alerts on critical metrics (chain height, peer count, error rate)
- Retention: Keep 15 days of data at 1m resolution, 1 year at 1h resolution

## Troubleshooting

**Metrics not appearing:**
1. Verify Aureon node is running: `curl http://localhost:8080/metrics`
2. Check Prometheus scrape logs
3. Verify datasource connection in Grafana

**Dashboard shows no data:**
1. Check time range (top right corner)
2. Verify Prometheus has collected data (check Prometheus UI)
3. Try different time range

**High cardinality issues:**
If you have thousands of labels, add relabeling rules in Prometheus config to reduce cardinality.

## Integration with Alerting

Example alert rules for Prometheus (`alerts.yml`):

```yaml
groups:
  - name: aureon_alerts
    rules:
      - alert: BlockProductionStopped
        expr: increase(blocks_produced_total[5m]) == 0
        for: 10m
        annotations:
          summary: "Block production has stopped"
      
      - alert: MempoolFull
        expr: mempool_size > 900
        for: 2m
        annotations:
          summary: "Mempool capacity above 90%"
      
      - alert: HighErrorRate
        expr: rate(http_errors_total[5m]) > 0.1
        for: 5m
        annotations:
          summary: "API error rate exceeds 10%"
```

## Further Reading

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Dashboarding](https://grafana.com/docs/grafana/latest/dashboards/)
- [Aureon Node Metrics API](../README.md#metrics)
