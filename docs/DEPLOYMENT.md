# Deployment Guide

Complete guide to deploying and operating Aureon blockchain nodes in production.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Single Node Deployment](#single-node-deployment)
3. [Multi-Node Network](#multi-node-network)
4. [Docker Deployment](#docker-deployment)
5. [Kubernetes Deployment](#kubernetes-deployment)
6. [Monitoring Setup](#monitoring-setup)
7. [Troubleshooting](#troubleshooting)

## Development Setup

### Prerequisites

- Rust 1.70+ ([install](https://rustup.rs/))
- Git
- 2GB RAM minimum
- 5GB disk space

### Local Build

```bash
# Clone repository
git clone https://github.com/ken-binoy/aureon-chain.git
cd aureon-chain

# Build release binary
cargo build --release

# Binary location
./target/release/aureon-node
```

### Verify Installation

```bash
# Run all tests (236 tests)
cargo test --all
# Result: test result: ok. 236 passed; 0 failed

# Check binary
./target/release/aureon-node --version
```

## Single Node Deployment

### Basic Configuration

Create `config.toml`:

```toml
[consensus]
engine = "pos"              # Proof of Stake
min_stake = 32              # Minimum validator stake
max_validators = 100        # Maximum validators
block_reward = 5.0          # Tokens per block

[network]
host = "0.0.0.0"           # Listen on all interfaces
p2p_port = 6000            # P2P port
api_port = 8080            # REST API port
bootstrap_peers = []        # Empty for bootstrap node

[database]
path = "./aureon_db"       # Database directory
cache_size_mb = 256        # RocksDB cache

[logging]
level = "info"             # debug, info, warn, error
```

### Run Node

```bash
# Start node with configuration
./target/release/aureon-node --config config.toml

# Check node is running
curl http://localhost:8080/chain/head

# Expected response:
# {"height": 0, "hash": "0x...", "timestamp": 1234567890}
```

### Initial Setup

```bash
# Create data directory
mkdir -p /var/aureon/{db,logs,config}

# Copy configuration
cp config.toml /var/aureon/config/

# Start with data directory
./target/release/aureon-node \
  --config /var/aureon/config/config.toml \
  --db-path /var/aureon/db \
  --log-file /var/aureon/logs/node.log
```

## Multi-Node Network

### Network Topology

```
Bootstrap Node (node0)
├─ Full Node 1 (node1)
├─ Full Node 2 (node2)
└─ Light Client (node3)
```

### Node 0: Bootstrap Node

```toml
[consensus]
engine = "pos"

[network]
host = "bootstrap.example.com"
p2p_port = 6000
bootstrap_peers = []  # This is the bootstrap node

[database]
path = "/var/aureon/node0/db"
```

Start:
```bash
./aureon-node --config node0.toml
```

### Node 1: Full Node (Validator)

```toml
[consensus]
engine = "pos"
min_stake = 32

[network]
host = "validator1.example.com"
p2p_port = 6001
bootstrap_peers = ["bootstrap.example.com:6000"]

[database]
path = "/var/aureon/node1/db"
```

Start:
```bash
./aureon-node --config node1.toml
```

### Node 2: Full Node (Non-Validator)

```toml
[consensus]
engine = "pos"

[network]
host = "node2.example.com"
p2p_port = 6002
bootstrap_peers = ["bootstrap.example.com:6000"]

[database]
path = "/var/aureon/node2/db"
```

Start:
```bash
./aureon-node --config node2.toml
```

### Verify Network

```bash
# Check node 0
curl http://bootstrap.example.com:8080/chain/head

# Check node 1
curl http://validator1.example.com:8080/chain/head

# Check node 2
curl http://node2.example.com:8080/chain/head

# All should show same block height (eventually consistent)
```

## Docker Deployment

### Build Image

```bash
# Build Docker image
docker build -t aureon:latest .

# Verify image
docker image ls aureon
```

### Run Single Container

```bash
# Create data volume
docker volume create aureon-data

# Run node
docker run -d \
  --name aureon-node \
  -p 6000:6000 \
  -p 8080:8080 \
  -v aureon-data:/aureon_db \
  -e RUST_LOG=info \
  aureon:latest

# Check logs
docker logs -f aureon-node

# Stop node
docker stop aureon-node
```

### Docker Compose (3-Node Cluster)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  node0:
    image: aureon:latest
    ports:
      - "6000:6000"
      - "8000:8080"
    volumes:
      - node0-data:/aureon_db
      - ./node0.toml:/config.toml
    environment:
      RUST_LOG: info
    command: /app/aureon-node --config /config.toml

  node1:
    image: aureon:latest
    ports:
      - "6001:6000"
      - "8001:8080"
    volumes:
      - node1-data:/aureon_db
      - ./node1.toml:/config.toml
    environment:
      RUST_LOG: info
    depends_on:
      - node0
    command: /app/aureon-node --config /config.toml

  node2:
    image: aureon:latest
    ports:
      - "6002:6000"
      - "8002:8080"
    volumes:
      - node2-data:/aureon_db
      - ./node2.toml:/config.toml
    environment:
      RUST_LOG: info
    depends_on:
      - node0
    command: /app/aureon-node --config /config.toml

volumes:
  node0-data:
  node1-data:
  node2-data:
```

### Run Docker Compose

```bash
# Start cluster
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f

# Stop cluster
docker-compose down

# Restart specific node
docker-compose restart node1
```

## Kubernetes Deployment

### Prerequisites

- kubectl configured
- Kubernetes 1.20+
- Helm 3+ (optional)

### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: aureon
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aureon-config
  namespace: aureon
data:
  config.toml: |
    [consensus]
    engine = "pos"
    
    [network]
    host = "0.0.0.0"
    p2p_port = 6000
    
    [database]
    path = "/data/aureon_db"
```

### StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: aureon-node
  namespace: aureon
spec:
  serviceName: aureon
  replicas: 3
  selector:
    matchLabels:
      app: aureon
  template:
    metadata:
      labels:
        app: aureon
    spec:
      containers:
      - name: aureon
        image: aureon:latest
        ports:
        - containerPort: 6000
          name: p2p
        - containerPort: 8080
          name: api
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /config
        env:
        - name: RUST_LOG
          value: info
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      volumes:
      - name: config
        configMap:
          name: aureon-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 50Gi
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: aureon
  namespace: aureon
spec:
  clusterIP: None
  selector:
    app: aureon
  ports:
  - port: 6000
    name: p2p
  - port: 8080
    name: api
```

### Deploy

```bash
# Create namespace
kubectl create namespace aureon

# Apply configs
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f statefulset.yaml
kubectl apply -f service.yaml

# Check status
kubectl get pods -n aureon
kubectl get svc -n aureon

# View logs
kubectl logs -n aureon aureon-node-0
kubectl logs -f -n aureon aureon-node-0

# Access API
kubectl port-forward -n aureon aureon-node-0 8080:8080
curl http://localhost:8080/chain/head
```

## Monitoring Setup

### Health Monitoring

```bash
# Check node health
curl http://localhost:8080/chain/head

# Expected response
{
  "height": 100,
  "hash": "0xabc123...",
  "timestamp": 1702000000,
  "validator": "0x...",
  "state_root": "0x..."
}
```

### Prometheus Integration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'aureon'
    static_configs:
      - targets: ['localhost:8080']
```

### Grafana Dashboards

Import dashboard template (in examples/grafana/):

```json
{
  "dashboard": {
    "title": "Aureon Node Metrics",
    "panels": [
      {
        "title": "Block Height",
        "targets": [
          "aureon_block_height"
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          "aureon_error_rate"
        ]
      }
    ]
  }
}
```

### Log Monitoring

```bash
# Follow logs
tail -f /var/aureon/logs/node.log

# Filter errors
grep ERROR /var/aureon/logs/node.log

# Count log levels
cat /var/aureon/logs/node.log | cut -d' ' -f1 | sort | uniq -c
```

## Performance Tuning

### Database Optimization

```toml
[database]
path = "./aureon_db"
cache_size_mb = 512          # Increase for high throughput
compression = true           # Enable compression
```

### Network Optimization

```toml
[network]
max_peers = 50              # Increase for larger network
sync_batch_size = 100       # Increase for faster sync
keep_alive_interval = 30    # Seconds
```

### Memory Optimization

```bash
# Monitor memory usage
ps aux | grep aureon-node

# Set memory limit
ulimit -v 2097152  # 2GB
```

## Security Hardening

### TLS for P2P

```toml
[network]
tls_enabled = true
tls_cert_path = "/etc/aureon/certs/node.crt"
tls_key_path = "/etc/aureon/certs/node.key"
```

### API Rate Limiting

```toml
[api]
rate_limit_per_second = 100
rate_limit_burst = 1000
```

### Firewall Rules

```bash
# Allow P2P
ufw allow 6000/tcp

# Allow API (internal only)
ufw allow from 10.0.0.0/8 to any port 8080

# Deny everything else
ufw default deny incoming
ufw default allow outgoing
```

## Backup & Recovery

### Backup Database

```bash
# Create backup
cp -r /var/aureon/db /var/aureon/db.backup.$(date +%Y%m%d)

# Compress
tar -czf aureon-db-backup.tar.gz /var/aureon/db.backup

# Upload to storage
aws s3 cp aureon-db-backup.tar.gz s3://backups/aureon/
```

### Restore Database

```bash
# Download backup
aws s3 cp s3://backups/aureon/aureon-db-backup.tar.gz .

# Extract
tar -xzf aureon-db-backup.tar.gz

# Stop node
systemctl stop aureon-node

# Replace database
rm -rf /var/aureon/db
cp -r /var/aureon/db.backup /var/aureon/db

# Start node
systemctl start aureon-node
```

## Troubleshooting

### Node Won't Start

```bash
# Check configuration
./aureon-node --config config.toml

# Verify binary
file ./target/release/aureon-node

# Check dependencies
ldd ./target/release/aureon-node

# Look at database
ls -la /var/aureon/db
```

### Network Issues

```bash
# Check port connectivity
nc -zv bootstrap.example.com 6000

# Verify DNS
nslookup bootstrap.example.com

# Check firewall
sudo ufw status
```

### Performance Issues

```bash
# Check CPU usage
top -p $(pgrep aureon-node)

# Check memory
free -h

# Check disk
df -h /var/aureon

# Profile node
cargo build --release --features profiling
```

### Sync Issues

```bash
# Check block height
curl http://localhost:8080/chain/head

# Check peers
curl http://localhost:8080/peers

# Increase log level
RUST_LOG=debug ./aureon-node --config config.toml
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Port already in use | Change P2P/API port in config |
| Out of memory | Reduce cache_size_mb or increase RAM |
| Slow sync | Increase max_peers, check network |
| Database corruption | Restore from backup |
| High CPU | Enable compression, reduce sync_batch_size |

## Maintenance

### Regular Tasks

- **Daily**: Check node health, monitor logs
- **Weekly**: Review metrics, check disk space
- **Monthly**: Backup database, update configuration
- **Quarterly**: Security audit, update Rust/dependencies

### Upgrade Procedure

```bash
# 1. Build new version
cd /path/to/aureon-chain
git pull origin main
cargo build --release

# 2. Test new version
cargo test --all

# 3. Stop old node
systemctl stop aureon-node

# 4. Backup database
cp -r /var/aureon/db /var/aureon/db.backup

# 5. Deploy new binary
sudo cp target/release/aureon-node /usr/local/bin/

# 6. Start new version
systemctl start aureon-node

# 7. Monitor
tail -f /var/aureon/logs/node.log
```

## Production Checklist

- [ ] Configuration reviewed by multiple people
- [ ] Network topology documented
- [ ] Monitoring and alerting set up
- [ ] Backup and recovery procedures tested
- [ ] Security hardening applied
- [ ] Performance baseline established
- [ ] Disaster recovery plan in place
- [ ] Team trained on operations
- [ ] Documentation updated
- [ ] Health checks implemented

---

**Need Help?**
- Check logs: `/var/aureon/logs/node.log`
- Check examples: `examples/README.md`
- Test locally: `cargo test --all`
- Review config: `config.toml`
