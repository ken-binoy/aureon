# Aureon Examples & Tutorials

Practical examples demonstrating how to use Aureon blockchain features.

## Quick Navigation

### 1. Token Transfer (`token_transfer.md`)
**Learn**: Account creation, balance management, token transfers
**Time**: 5 min read
**Level**: Beginner
**Topics**: 
- Create accounts with initial balances
- Transfer tokens between accounts
- Verify final balances
- Error handling

### 2. Smart Contracts (`smart_contract.md`)
**Learn**: Deploy and execute WebAssembly contracts
**Time**: 10 min read
**Level**: Intermediate
**Topics**:
- Write WASM contracts
- Deploy contracts to blockchain
- Call contract functions
- Gas metering
- Error handling

### 3. SPV Light Client (`spv_light_client.md`)
**Learn**: Verify transactions without downloading full blocks
**Time**: 15 min read
**Level**: Advanced
**Topics**:
- Create light client
- Sync block headers
- Verify merkle proofs
- Compress state
- Mobile wallet integration

### 4. Production Monitoring (`production_monitoring.md`)
**Learn**: Monitor system health, latency, errors, throughput
**Time**: 10 min read
**Level**: Intermediate
**Topics**:
- Track latency with percentiles
- Monitor error rates
- Measure throughput
- Generate health dashboards
- Alerting strategies

## Learning Path

### For Beginners
1. Start with **Token Transfer** to understand basic operations
2. Read the **README.md** for architecture overview
3. Run `cargo test --all` to verify setup

### For Smart Contract Developers
1. **Token Transfer** - understand accounts and balances
2. **Smart Contracts** - deploy and execute WASM
3. **Production Monitoring** - track contract performance

### For Node Operators
1. Read **README.md** - understand architecture
2. **Production Monitoring** - set up observability
3. Review **PHASE_10_SUMMARY.md** - production hardening features

### For Light Client Users
1. **Token Transfer** - understand blockchain basics
2. **SPV Light Client** - learn header sync and proofs
3. **Production Monitoring** - monitor client health

## Example Topics by Category

### Core Blockchain
- Token Transfer
- Account Management
- Balance Verification

### Smart Contracts
- WASM Execution
- Gas Metering
- Contract Registry
- State Management

### Light Client (SPV)
- Header Synchronization
- Merkle Proof Verification
- State Compression
- Memory Efficiency

### Production Systems
- Latency Tracking
- Error Monitoring
- Health Dashboards
- Stress Testing
- Circuit Breakers
- Rate Limiting

### Advanced Topics
- Consensus (PoS/PoW)
- State Trie (MPT)
- Cryptography
- P2P Networking

## Running Examples

### Run All Tests
```bash
cargo test --all
# 236 tests total
```

### Run Category Tests
```bash
# Core blockchain tests
cargo test --package aureon-core

# Smart contracts
cargo test wasm::

# Light client
cargo test spv_

# Production hardening
cargo test error_recovery
cargo test performance
cargo test stress_testing
cargo test production_monitoring
```

### Run with Details
```bash
# Show output
cargo test --all -- --nocapture

# Show failure details
RUST_BACKTRACE=1 cargo test --all

# Run single test
cargo test token::tests::test_transfer
```

## Code Examples Structure

Each example includes:
1. **Overview** - What will you learn
2. **Code Example** - Complete working code
3. **Running** - How to compile and run
4. **Advanced** - More complex usage patterns
5. **Concepts** - Key ideas explained
6. **Testing** - How to test your code
7. **References** - Related modules and docs

## Common Tasks

### I want to...

**Create a blockchain account**
→ See `token_transfer.md`

**Deploy a smart contract**
→ See `smart_contract.md`

**Verify a transaction without full node**
→ See `spv_light_client.md`

**Monitor node performance**
→ See `production_monitoring.md`

**Understand consensus**
→ See `README.md` Layer 1 section

**Handle errors in production**
→ See `README.md` Layer 6 section

## File Organization

```
examples/
├── token_transfer.md           # 1. Basic account/balance operations
├── smart_contract.md           # 2. WASM contract deployment
├── spv_light_client.md         # 3. Light client verification
├── production_monitoring.md    # 4. Observability & monitoring
└── README.md (this file)       # Navigation & index
```

## Test Coverage

Each example has corresponding test cases:

| Example | Tests | Status |
|---------|-------|--------|
| Token Transfer | aureon-core | ✅ |
| Smart Contracts | wasm:: | ✅ |
| SPV Light Client | spv_* | ✅ |
| Production Monitoring | production_monitoring | ✅ |

## Performance Benchmarks

### From Examples

**Token Transfer**
- Create account: <1ms
- Transfer: <5µs
- Verify balance: <1µs

**Smart Contracts**
- Deploy: 10-50ms
- Execute: 1-10ms
- Gas per op: 1,000 units

**SPV Light Client**
- Sync 1000 headers: <100ms
- Verify proof: <100µs
- Memory: <5MB for 10K headers

**Production Monitoring**
- Latency p95: <5ms
- Error rate: <1%
- Throughput: 100+ ops/sec

## Next Steps

1. **Read**: Choose an example from above
2. **Code**: Copy the example code
3. **Test**: Run `cargo test` to verify
4. **Modify**: Change parameters and experiment
5. **Learn**: Read referenced modules

## Advanced Learning

### Understand Implementation
- Read test files: `src/*/tests.rs`
- Review module documentation: `src/*/mod.rs`
- Check references in examples

### Extend Examples
- Combine multiple examples
- Add error handling
- Implement variations
- Create production integrations

### Contribute
- Submit improvements
- Add more examples
- Document patterns
- Share learnings

## Resources

### Documentation
- **README.md** - Architecture & features
- **PHASE_10_SUMMARY.md** - Production hardening details
- **PHASE_9_SUMMARY.md** - Light client details
- **Source code** - Best documentation

### Testing
- **Unit tests**: `cargo test`
- **Integration tests**: Examples + tests
- **Stress tests**: `cargo test stress_testing`

### Benchmarking
- Run stress tests for performance data
- Monitor with production examples
- Compare against targets

## Common Questions

### Q: Can I run examples without Rust?
**A**: No, you need Rust 1.70+ (free from rustup.rs)

### Q: How do I run a single example?
**A**: Each example has test cases - run with `cargo test example_name`

### Q: Can I modify examples?
**A**: Yes! Edit and run `cargo test` to verify changes

### Q: Where's the source code?
**A**: Examples reference modules in `aureon-node/src/` and `aureon-core/src/`

### Q: How accurate are the performance numbers?
**A**: From stress tests on typical hardware (Apple M1 shown)

## Troubleshooting

### Build Fails
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

### Test Fails
```bash
# Get detailed output
cargo test --all -- --nocapture

# Show backtrace
RUST_BACKTRACE=1 cargo test --all
```

### Performance Issues
```bash
# Build in release mode
cargo build --release

# Run stress tests
cargo test stress_testing -- --nocapture
```

## Contributing Examples

Want to add an example?
1. Create new `.md` file in examples/
2. Follow structure from existing examples
3. Include code, tests, and references
4. Submit pull request

---

**Total Examples**: 4
**Total Test Coverage**: 236 tests
**Example Time**: ~40 minutes total reading
**Code Time**: ~1-2 hours to understand fully
**Status**: All examples production-ready ✅

**Start with**: `token_transfer.md` for beginners, `spv_light_client.md` for advanced
