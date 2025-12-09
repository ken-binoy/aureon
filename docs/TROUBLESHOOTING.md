# Troubleshooting Guide

Common issues and solutions for Aureon blockchain operations.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Build Issues](#build-issues)
3. [Runtime Issues](#runtime-issues)
4. [Network Issues](#network-issues)
5. [Performance Issues](#performance-issues)
6. [Data Issues](#data-issues)
7. [Consensus Issues](#consensus-issues)

## Installation Issues

### Problem: Rust Not Installed

**Error**: `command not found: cargo`

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source $HOME/.cargo/env

# Verify
cargo --version
```

### Problem: Wrong Rust Version

**Error**: `error: toolchain 'stable' is not installed`

**Solution**:
```bash
# Update Rust
rustup update

# Check version
rustc --version

# Should be 1.70+
```

### Problem: Dependencies Not Found

**Error**: `error: failed to resolve: use of undeclared type`

**Solution**:
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

## Build Issues

### Problem: Compilation Failed

**Error**: `error[E0433]: cannot find function in scope`

**Solution**:
```bash
# 1. Check Rust version
rustc --version

# 2. Update dependencies
cargo update

# 3. Clean build
cargo clean
cargo build --release

# 4. Check specific errors
cargo build --release 2>&1 | head -50
```

### Problem: Out of Memory During Build

**Error**: `error: could not compile`

**Solution**:
```bash
# Reduce parallel jobs
cargo build --release -j 2

# Or use single-threaded build
cargo build --release -j 1

# Free up memory
killall chrome    # or other heavy processes
```

### Problem: Linking Error

**Error**: `error: linking with 'cc' failed`

**Solution**:
```bash
# Install build tools (macOS)
xcode-select --install

# Install build tools (Ubuntu)
sudo apt-get install build-essential

# Install build tools (CentOS)
sudo yum groupinstall "Development Tools"
```

## Runtime Issues

### Problem: Node Won't Start

**Error**: `panicked at 'failed to initialize...`

**Solution**:
```bash
# 1. Check configuration file
cat config.toml | grep -v '^#' | grep -v '^$'

# 2. Validate configuration
./aureon-node --config config.toml --validate-only

# 3. Check file permissions
ls -la config.toml
chmod 644 config.toml

# 4. Check database permissions
ls -la ./aureon_db/
chmod 755 ./aureon_db/

# 5. Try verbose logging
RUST_LOG=debug ./aureon-node --config config.toml
```

### Problem: "Port Already In Use"

**Error**: `error: Address already in use`

**Solution**:
```bash
# Find process using port
lsof -i :6000

# Kill the process
kill -9 <PID>

# Or use different port in config.toml
sed -i 's/p2p_port = 6000/p2p_port = 6001/' config.toml

# Verify port is free
nc -zv localhost 6001
```

### Problem: "Permission Denied"

**Error**: `error: Permission denied`

**Solution**:
```bash
# Check file ownership
ls -la /var/aureon/

# Fix permissions
sudo chown -R aureon:aureon /var/aureon/

# Run as aureon user
sudo -u aureon ./aureon-node --config config.toml

# Or run with sudo
sudo ./aureon-node --config config.toml
```

### Problem: Segmentation Fault

**Error**: `Segmentation fault (core dumped)`

**Solution**:
```bash
# 1. Check memory
free -h

# 2. Run with debugger
rust-gdb ./aureon-node

# 3. Get backtrace
RUST_BACKTRACE=1 ./aureon-node --config config.toml

# 4. Check database integrity
cargo run --release -p aureon-node -- --check-db

# 5. Rebuild clean
cargo clean
cargo build --release
```

## Network Issues

### Problem: Cannot Connect to Peer

**Error**: `failed to connect to bootstrap peer`

**Solution**:
```bash
# 1. Check network connectivity
ping bootstrap.example.com

# 2. Check DNS resolution
nslookup bootstrap.example.com
dig bootstrap.example.com

# 3. Test port connectivity
nc -zv bootstrap.example.com 6000

# 4. Check firewall
sudo ufw status
sudo ufw allow 6000/tcp

# 5. Check configuration
grep bootstrap_peers config.toml

# 6. Try direct IP
sed -i 's/bootstrap.example.com/10.0.0.1/' config.toml
```

### Problem: Node Gets Disconnected

**Error**: `peer disconnected` (repeated in logs)

**Solution**:
```bash
# 1. Check network stability
ping -c 100 bootstrap.example.com

# 2. Check firewall rules
sudo ufw status
sudo iptables -L

# 3. Increase keep-alive timeout
sed -i 's/keep_alive_interval = .*/keep_alive_interval = 60/' config.toml

# 4. Check logs for pattern
grep "disconnected" /var/aureon/logs/node.log

# 5. Check peer count
curl http://localhost:8080/peers
```

### Problem: All Peers Rejected

**Error**: `rejected connection from peer`

**Solution**:
```bash
# 1. Check node identity
curl http://localhost:8080/node-id

# 2. Verify network compatibility
grep engine config.toml  # Should match other nodes

# 3. Check protocol version
grep version config.toml

# 4. Try manual peer connection
# Add peer directly in config
echo "bootstrap_peers = ['manual.peer:6000']" >> config.toml

# 5. Check logs
RUST_LOG=aureon_node::network=debug ./aureon-node
```

## Performance Issues

### Problem: High CPU Usage

**Error**: Node using >80% CPU

**Solution**:
```bash
# 1. Identify hot function
cargo build --release --features profiling
perf record -p $(pgrep aureon-node)
perf report

# 2. Check if validating
grep engine config.toml  # pos = higher CPU

# 3. Reduce work
# Disable some features, reduce peers
sed -i 's/max_peers = .*/max_peers = 10/' config.toml

# 4. Check for busy loops
grep -r "while true" src/

# 5. Use release build
cargo build --release
```

### Problem: Slow Block Processing

**Error**: Block time > 10 seconds

**Solution**:
```bash
# 1. Check latency
curl http://localhost:8080/metrics | grep latency

# 2. Check database performance
du -sh ./aureon_db/

# 3. Increase cache
sed -i 's/cache_size_mb = .*/cache_size_mb = 512/' config.toml
systemctl restart aureon-node

# 4. Check network latency
ping bootstrap.example.com

# 5. Monitor
RUST_LOG=debug ./aureon-node | grep "block"
```

### Problem: High Memory Usage

**Error**: Node using >2GB RAM

**Solution**:
```bash
# 1. Check memory
ps -o pid,vsz,rss,comm= | grep aureon

# 2. Enable compression
sed -i 's/\[database\]/&\ncompression = true/' config.toml

# 3. Reduce cache
sed -i 's/cache_size_mb = .*/cache_size_mb = 128/' config.toml

# 4. Check for memory leaks
valgrind ./aureon-node --config config.toml

# 5. Restart
systemctl restart aureon-node

# 6. Monitor memory
watch -n 1 'ps aux | grep aureon'
```

### Problem: High Disk I/O

**Error**: Disk 100% utilization

**Solution**:
```bash
# 1. Check disk usage
df -h
du -sh ./aureon_db/

# 2. Enable compression
sed -i 's/compression = false/compression = true/' config.toml

# 3. Reduce sync frequency
sed -i 's/sync_interval = .*/sync_interval = 60/' config.toml

# 4. Move to faster disk
mv ./aureon_db/ /mnt/fast-disk/

# 5. Monitor I/O
iostat -x 1 10
```

## Data Issues

### Problem: Database Corrupted

**Error**: `database corruption detected` or `checksum failed`

**Solution**:
```bash
# 1. Stop node
systemctl stop aureon-node

# 2. Backup corrupted database
mv aureon_db aureon_db.corrupted

# 3. Restore from backup (if available)
cp -r aureon_db.backup aureon_db

# 4. Or resync from network
rm -rf aureon_db
systemctl start aureon-node

# 5. Monitor recovery
tail -f /var/aureon/logs/node.log
```

### Problem: Blockchain Fork

**Error**: `fork detected at height 100`

**Solution**:
```bash
# 1. Check block height
curl http://localhost:8080/chain/head

# 2. Compare with peers
curl http://peer1:8080/chain/head
curl http://peer2:8080/chain/head

# 3. If behind, resync
systemctl stop aureon-node
rm -rf aureon_db
systemctl start aureon-node

# 4. Monitor sync
watch -n 5 'curl -s http://localhost:8080/chain/head | jq .height'

# 5. Wait for consensus
# Node will automatically sync to main chain
```

### Problem: State Root Mismatch

**Error**: `state root mismatch at block 50`

**Solution**:
```bash
# 1. Stop node
systemctl stop aureon-node

# 2. Restore from backup
rm -rf aureon_db
tar -xzf aureon-db-backup-<date>.tar.gz

# 3. Start with verbose logging
RUST_LOG=debug systemctl start aureon-node

# 4. Monitor state transitions
tail -f /var/aureon/logs/node.log | grep state

# 5. If still failing, resync
rm -rf aureon_db
systemctl start aureon-node
```

## Consensus Issues

### Problem: Circuit Breaker Open

**Error**: `circuit breaker is open`

**Solution**:
```bash
# 1. Wait for timeout
# Circuit breaker auto-closes after timeout period (default: 60s)

# 2. Check error logs
grep "circuit" /var/aureon/logs/node.log

# 3. Address root cause
# Usually network or resource issue

# 4. Monitor recovery
watch -n 1 'curl -s http://localhost:8080/health | jq .status'

# 5. If still failing, restart
systemctl restart aureon-node
```

### Problem: Rate Limited

**Error**: `rate limit exceeded`

**Solution**:
```bash
# 1. Check rate limit config
grep rate_limit config.toml

# 2. Increase limits
sed -i 's/rate_limit_per_second = .*/rate_limit_per_second = 1000/' config.toml

# 3. Restart node
systemctl restart aureon-node

# 4. Monitor requests
tail -f /var/aureon/logs/node.log | grep rate
```

### Problem: Not Validating

**Error**: Node won't become validator

**Solution**:
```bash
# 1. Check balance
curl http://localhost:8080/balance/0xaddress

# 2. Check minimum stake
grep min_stake config.toml

# 3. Ensure balance >= min_stake
# Transfer tokens to validator address

# 4. Check consensus engine
grep engine config.toml  # Should be 'pos'

# 5. Wait for next epoch
# Validators elected at block boundaries
```

## Getting Help

### Check Logs

```bash
# Last 50 lines
tail -50 /var/aureon/logs/node.log

# Follow logs
tail -f /var/aureon/logs/node.log

# Search for errors
grep ERROR /var/aureon/logs/node.log

# Time range
sed -n '/2024-01-15T10:00/,/2024-01-15T11:00/p' /var/aureon/logs/node.log
```

### Run Diagnostics

```bash
#!/bin/bash
# Run diagnostics

echo "=== System Info ==="
uname -a
uptime

echo "=== Rust/Cargo ==="
rustc --version
cargo --version

echo "=== Node Status ==="
curl http://localhost:8080/chain/head

echo "=== Peer Count ==="
curl http://localhost:8080/peers | jq '.length'

echo "=== Memory Usage ==="
ps aux | grep aureon-node | grep -v grep

echo "=== Disk Usage ==="
du -sh ./aureon_db/

echo "=== Recent Errors ==="
tail -20 /var/aureon/logs/node.log | grep ERROR
```

### Report Issue

When reporting issues, include:
1. Output of diagnostic script
2. Configuration file (sanitized)
3. Recent log entries
4. Steps to reproduce
5. Expected vs actual behavior

---

**Quick Links**:
- `DEPLOYMENT.md` - Deployment guide
- `MONITORING.md` - Monitoring setup
- `examples/` - Working examples
- GitHub Issues: https://github.com/ken-binoy/aureon-chain/issues
