# Aureon Blockchain - DevOps & Deployment Guide

## Overview

Aureon is a production-ready blockchain implementation with comprehensive DevOps infrastructure for containerized deployment, multi-node testing, and cluster management.

## Quick Start

### Prerequisites
- Docker 20.10+ and Docker Compose 2.0+
- Rust 1.70+ (for local development)
- Make (for automation)

### Start 3-Node PoW Cluster (30 seconds)
```bash
# Build Docker image
docker build -t aureon-chain:latest .

# Start cluster
docker-compose up -d

# Verify health
curl http://localhost:8000/chain/head
curl http://localhost:8001/chain/head
curl http://localhost:8002/chain/head

# View logs
docker-compose logs -f
```

### Stop Cluster
```bash
docker-compose down
```

## Docker Architecture

### Multi-Stage Build
- **Builder Stage**: Compiles release binary in Rust container (~2GB)
- **Runtime Stage**: Minimal Debian image with only binary (~200MB)
- **Health Check**: Periodic verification of `/chain/head` endpoint

### Image Details
```
FROM rust:latest as builder
  - Copies Cargo.toml and source
  - Runs cargo build --release -p aureon-node
  - Result: 2GB intermediate image

FROM debian:bookworm-slim
  - Copies binary from builder
  - Copies config.toml
  - EXPOSE: 6000 (P2P), 8080 (REST), 8081 (WS)
  - HEALTHCHECK: curl http://localhost:8080/chain/head
  - Result: ~200MB production image
```

## Docker Compose Configurations

### 1. Production PoW Cluster (`docker-compose.yml`)

Three-node Proof of Work cluster for production testing and deployment.

```yaml
Services:
  aureon-node-1 (primary)
    - P2P Port: 6000
    - REST API: 8000
    - Database: node1_data volume
    - Role: Genesis/primary validator

  aureon-node-2
    - P2P Port: 6001
    - REST API: 8001
    - Database: node2_data volume
    - Depends on: aureon-node-1 healthy
    - Role: Secondary validator

  aureon-node-3
    - P2P Port: 6002
    - REST API: 8002
    - Database: node3_data volume
    - Depends on: aureon-node-1 healthy
    - Role: Tertiary validator

Network: aureon_network (bridge)
Consensus: PoW (difficulty=2 for testing)
```

**Start Production Cluster:**
```bash
make up
# or
docker-compose up -d
```

**API Endpoints:**
- Node 1: `http://localhost:8000`
- Node 2: `http://localhost:8001`
- Node 3: `http://localhost:8002`

### 2. Development PoS Cluster (`docker-compose.dev.yml`)

Proof of Stake validator setup for development and validator testing.

```yaml
Services:
  aureon-validator-1
    - P2P Port: 6010
    - REST API: 8010
    - Database: validator1_data volume
    - Role: PoS validator (stake >= 1000 tokens)

  aureon-validator-2
    - P2P Port: 6011
    - REST API: 8011
    - Database: validator2_data volume
    - Role: PoS validator (stake >= 1000 tokens)

  aureon-node
    - P2P Port: 6020
    - REST API: 8020
    - Database: node_data volume
    - Role: Regular node (non-validator)

Network: aureon_dev_network (bridge)
Consensus: PoS (minimum stake: 1000 tokens)
```

**Start Development Cluster:**
```bash
make up-dev
# or
docker-compose -f docker-compose.dev.yml up -d
```

**Validator Endpoints:**
- Validator 1: `http://localhost:8010`
- Validator 2: `http://localhost:8011`
- Regular Node: `http://localhost:8020`

## Make Commands

### Building
```bash
make build              # Build production Docker image
make build-dev         # Build development image
make dev-build         # Build locally with Cargo
```

### Cluster Operations
```bash
make up                # Start 3-node PoW cluster
make up-dev            # Start PoS validator cluster
make down              # Stop all containers
make clean             # Remove containers and volumes
make clean-images      # Remove Docker images
```

### Monitoring & Debugging
```bash
make status            # Show container status (docker ps)
make logs              # Stream all container logs
make logs-node-1       # Stream Node 1 logs
make logs-node-2       # Stream Node 2 logs
make logs-node-3       # Stream Node 3 logs
make shell-node-1      # Open /bin/bash in Node 1
make health-check      # Verify all nodes responding
```

### Testing
```bash
make test              # Run integration tests against PoW cluster
make test-dev          # Run tests against PoS cluster
make dev-test          # Run local Cargo tests
```

## REST API Endpoints

### Chain Info
```bash
# Get blockchain head
curl http://localhost:8000/chain/head
# Response: { "head": { "height": 42, "hash": "0x...", ... } }

# Get block by height
curl http://localhost:8000/block/10
# Response: { "block": { ... }, "status": "ok" }

# Get transaction by hash
curl http://localhost:8000/tx/0xabcd1234
# Response: { "tx": { ... }, "status": "ok" }
```

### Account Management
```bash
# Get account balance
curl http://localhost:8000/balance/0xaddress
# Response: { "balance": 1000, "status": "ok" }
```

### Transaction Submission
```bash
# Submit transaction
curl -X POST http://localhost:8000/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"tx": {"from": "...", "to": "...", ...}}'
```

### Contract Operations
```bash
# Deploy contract
curl -X POST http://localhost:8000/deploy-contract \
  -H "Content-Type: application/json" \
  -d '{"wasm": "0x...", "args": {}}'

# Call contract
curl -X POST http://localhost:8000/call-contract \
  -H "Content-Type: application/json" \
  -d '{"address": "0x...", "function": "...", "args": {}}'
```

## Network Communication

### P2P Protocol
- **Transport**: TCP over Docker bridge network
- **Message Types**: 10 types (Ping, Pong, Block, GetBlock, GetBlockResponse, SyncRequest, SyncResponse, PeerInfo, Transactions)
- **Serialization**: JSON (text-line protocol)
- **Peer Discovery**: Bootstrap via startup, automatic height tracking

### Node Communication Example
```bash
# Container 1 sends message to Container 2
docker-compose exec aureon-node-1 curl http://aureon-node-2:8080/chain/head

# Works because docker-compose creates DNS entries for service names
```

## Health Checks

### Container Health
```bash
# Check Docker health status
docker-compose ps

# Output shows HEALTHY/UNHEALTHY for each node
CONTAINER ID   STATUS
aureon-node-1  Up 2 minutes (healthy)
aureon-node-2  Up 2 minutes (healthy)
aureon-node-3  Up 2 minutes (healthy)
```

### API Health
```bash
make health-check

# Output:
# {"head": {...}}✅ Node 1 healthy
# {"head": {...}}✅ Node 2 healthy
# {"head": {...}}✅ Node 3 healthy
```

## Development Workflow

### Local Development
```bash
# Build locally
make dev-build

# Run tests
make dev-test

# Run single node locally
make dev-run
```

### Containerized Testing
```bash
# Build container
make build

# Start cluster
make up

# Submit test transaction
curl -X POST http://localhost:8000/submit-tx \
  -H "Content-Type: application/json" \
  -d '{
    "from": "0xsender",
    "to": "0xreceiver",
    "amount": 100,
    "nonce": 1
  }'

# Verify propagation to other nodes
curl http://localhost:8001/tx/0xhash
curl http://localhost:8002/tx/0xhash

# Stop cluster
make down
```

## Production Deployment

### Prerequisites
- Docker registry (Docker Hub, ECR, GCR)
- Kubernetes cluster or Docker Swarm
- TLS/HTTPS configuration
- Persistent volume provisioner

### Build and Push
```bash
# Build with production tag
docker build -t myregistry/aureon:latest .
docker build -t myregistry/aureon:v1.0.0 .

# Push to registry
docker push myregistry/aureon:latest
docker push myregistry/aureon:v1.0.0
```

### Kubernetes Deployment
```yaml
apiVersion: v1
kind: Service
metadata:
  name: aureon-node
spec:
  selector:
    app: aureon
  ports:
  - name: p2p
    port: 6000
    targetPort: 6000
  - name: api
    port: 8080
    targetPort: 8080
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: aureon-node
spec:
  serviceName: aureon-node
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
      - name: aureon-node
        image: myregistry/aureon:latest
        ports:
        - containerPort: 6000
          name: p2p
        - containerPort: 8080
          name: api
        livenessProbe:
          httpGet:
            path: /chain/head
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        volumeMounts:
        - name: data
          mountPath: /var/aureon
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 10Gi
```

### Docker Swarm Deployment
```bash
# Initialize swarm (on manager node)
docker swarm init

# Create config
docker config create aureon-config config.toml

# Deploy stack
docker stack deploy -c docker-compose.yml aureon
```

## Troubleshooting

### Nodes Not Connecting
```bash
# Check logs
docker-compose logs aureon-node-1
docker-compose logs aureon-node-2

# Verify network
docker network ls
docker network inspect aureon_network

# Check P2P port accessibility
docker-compose exec aureon-node-1 nc -zv aureon-node-2 6001
```

### High Memory Usage
```bash
# Check memory stats
docker stats aureon-node-1

# Reduce cache/buffer sizes in config.toml
# Increase Docker memory limit
# Use Docker memory constraints: mem_limit: 2g
```

### Database Corruption
```bash
# Stop containers
docker-compose down

# Remove corrupted volume
docker volume rm aureon-chain_node1_data

# Start fresh
docker-compose up -d
```

### Node Out of Sync
```bash
# Force resync by clearing database and restarting
docker-compose down -v
docker-compose up -d

# Or check block heights
curl http://localhost:8000/chain/head | jq .head.height
curl http://localhost:8001/chain/head | jq .head.height
curl http://localhost:8002/chain/head | jq .head.height
```

## Performance Tuning

### Network Optimization
```yaml
# docker-compose.yml
services:
  aureon-node-1:
    # Enable IPv4 forwarding
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.tcp_tw_reuse=1
```

### Database Optimization
```toml
# config.toml
[database]
cache_size_mb = 256  # Increase from default
compression = true   # Enable RocksDB compression
```

### API Performance
```bash
# Use HTTP/2 and keep-alive
curl --http2 http://localhost:8000/chain/head
curl -H "Connection: keep-alive" http://localhost:8000/chain/head
```

## Monitoring & Logging

### Container Logs
```bash
# Real-time logs for all containers
docker-compose logs -f

# Logs for specific service
docker-compose logs -f aureon-node-1

# Last N lines
docker-compose logs --tail=100 aureon-node-1
```

### Log Rotation (Production)
```bash
# Configure in docker-compose.yml
services:
  aureon-node-1:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "3"
```

### Metrics Export (Future)
Prometheus and Grafana integration planned for:
- Block production rate
- Transaction throughput
- Consensus round times
- P2P peer counts
- Database metrics

## Continuous Integration

### GitHub Actions Example
```yaml
name: Build and Test

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: docker/setup-buildx-action@v1
    - uses: docker/build-push-action@v2
      with:
        push: false
        tags: aureon:latest
    - run: |
        docker run --rm aureon:latest cargo test --all
```

## Backup & Recovery

### Database Backup
```bash
# Backup node1 database
docker run --rm -v aureon-chain_node1_data:/data \
  -v $(pwd)/backups:/backups \
  debian:bookworm-slim \
  tar czf /backups/node1-$(date +%Y%m%d).tar.gz -C /data .

# List backups
ls -lh backups/
```

### Restore from Backup
```bash
# Remove existing volume
docker-compose down -v

# Restore from backup
docker volume create aureon-chain_node1_data
docker run --rm -v aureon-chain_node1_data:/data \
  -v $(pwd)/backups:/backups \
  debian:bookworm-slim \
  tar xzf /backups/node1-20240101.tar.gz -C /data

# Start cluster
docker-compose up -d
```

## References

- **Docker Documentation**: https://docs.docker.com/
- **Docker Compose**: https://docs.docker.com/compose/
- **Kubernetes**: https://kubernetes.io/docs/
- **Aureon Blockchain**: See README.md

## Support

For issues or questions:
1. Check logs: `docker-compose logs`
2. Verify health: `make health-check`
3. Review config: `cat config.toml`
4. Check Docker version: `docker --version && docker-compose --version`
