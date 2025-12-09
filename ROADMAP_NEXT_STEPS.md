# Aureon Development Roadmap - Next Steps

## Current Status
✅ **Phase 5.1 Complete** - REST API with 7 endpoints  
✅ **Phase 4.3 Complete** - Enhanced WASM runtime  
**Overall Completion:** ~65%

---

## Next Priority Actions

### 🔴 HIGH PRIORITY - Phase 4.2: Config System
**Why:** Blocks many subsequent phases from working properly  
**What:** Load consensus type from TOML configuration file  
**Effort:** 1-2 days  
**Impact:** Enables runtime consensus selection (PoA/PoW/PoS)

**Tasks:**
- [ ] Create `config.rs` module with TOML parsing
- [ ] Load from `aureon.toml` or environment variable
- [ ] Support `consensus_type = "pow" | "pos" | "poa"`
- [ ] Load validator configuration for PoS/PoA
- [ ] Wire into consensus engine factory

**Example Config:**
```toml
[blockchain]
name = "aureon"
chain_id = 1
consensus_type = "pow"

[consensus.pow]
difficulty = 4
min_difficulty = 2

[consensus.pos]
min_stake = 1000
validator_count = 10

[api]
host = "0.0.0.0"
port = 8080
```

---

### 🟡 MEDIUM PRIORITY - Phase 5.2: API Indexing & WebSocket
**Why:** Unblocks production-ready block/tx queries  
**What:** Index blocks/transactions + WebSocket subscriptions  
**Effort:** 2-3 days  
**Impact:** Complete API layer for client applications

**Tasks:**
- [ ] Add in-memory transaction hash index
- [ ] Add in-memory block hash index
- [ ] Implement actual `/block/:hash` lookup
- [ ] Implement actual `/tx/:hash` lookup
- [ ] Add WebSocket endpoint for block subscriptions
- [ ] Add WebSocket endpoint for transaction subscriptions
- [ ] Proper HTTP status codes (400/500)
- [ ] Rate limiting middleware

**Architecture:**
```rust
pub struct TxIndex {
    // tx_hash -> (block_number, index_in_block)
    index: HashMap<String, (u64, usize)>
}

pub struct BlockIndex {
    // block_hash -> block_data
    index: HashMap<String, Block>
}

// In main loop:
processor.apply_block(&block);
tx_index.index_block(&block);
block_index.add(&block);
broadcast_to_websockets(&block);  // New!
```

---

### 🟡 MEDIUM PRIORITY - Phase 5.3: Transaction Integration
**Why:** Makes submitted transactions actually execute  
**What:** Wire API transactions to consensus → block production  
**Effort:** 1-2 days  
**Impact:** End-to-end transaction flow working

**Tasks:**
- [ ] Create transaction mempool/queue
- [ ] Modify consensus engine to pull from mempool
- [ ] Generate block when:
  - [x] Time elapsed since last block
  - [x] Mempool not empty
  - [x] Minimum transactions received
- [ ] Return transaction hash on submission
- [ ] Implement transaction receipt lookup
- [ ] Track transaction status: pending → confirmed

**Flow:**
```
API POST /submit-tx
    ↓
Transaction::from_request() (validation)
    ↓
MEMPOOL.add(tx)
    ↓
Return {status: "pending", tx_hash: "0x..."}
    ↓
[Consensus loop picks up]
    ↓
Block::new([tx, tx, ...])
    ↓
Blockchain.add_block(block)
    ↓
WebSocket broadcast_block(block)
    ↓
Client polls GET /tx/{tx_hash} → status: "confirmed"
```

---

## Phase Execution Order

```
NOW: Phase 5.1 ✅ (Just completed)
  ↓
1. Phase 4.2 - Config system (1-2 days)
  ↓
2. Phase 5.2 - API indexing (2-3 days)
  ↓
3. Phase 5.3 - TX integration (1-2 days)
  ↓
4. Phase 6.1 - P2P enhancement (2-3 days)
  ↓
5. Phase 4.1 - Full contract features (3-5 days)
  ↓
Total: 9-15 days to MVP completion
```

---

## Estimated Timeline to Production

| Phase | Task | Effort | Status |
|-------|------|--------|--------|
| 4.1 | WASM Runtime MVP | 2d | ✅ Done |
| 4.3 | Enhanced WASM | 3h | ✅ Done |
| 5.1 | REST API | 2h | ✅ Done |
| **4.2** | **Config System** | **1-2d** | 🔴 **NEXT** |
| **5.2** | **API Indexing** | **2-3d** | ⏳ QUEUE |
| **5.3** | **TX Integration** | **1-2d** | ⏳ QUEUE |
| 6.1 | P2P Enhancement | 2-3d | ⏳ QUEUE |
| 2.3 | Full consensus validation | 1d | ⏳ QUEUE |

---

## Testing Strategy

### Phase 4.2 Testing
```bash
# 1. Create config file
cat > aureon.toml << EOF
[blockchain]
consensus_type = "pow"
EOF

# 2. Run with config
cargo run -- --config aureon.toml

# 3. Verify output
# Expected: "Selected Consensus: Pow"
```

### Phase 5.2 Testing
```bash
# 1. Deploy contract
curl -X POST http://127.0.0.1:8080/contract/deploy \
  -d '{"code":[...], "gas_limit":10000}' \
  -H "Content-Type: application/json"
# Returns: {"address":"abc123","status":"deployed"}

# 2. Look up by hash
curl http://127.0.0.1:8080/block/abc123
# Should now return actual block data (not placeholder)

# 3. WebSocket subscription
wscat -c ws://127.0.0.1:8080/subscribe/blocks
# Should receive: {"type":"block_produced","block":{...}}
```

### Phase 5.3 Testing
```bash
# 1. Submit transaction
TXID=$(curl -X POST http://127.0.0.1:8080/submit-tx \
  -d '{"from":"Alice","to":"Bob","amount":50}' \
  | jq -r '.tx_hash')

# 2. Check status (should be pending)
curl http://127.0.0.1:8080/tx/$TXID
# {"status":"pending","from":"Alice","to":"Bob","amount":50}

# 3. Wait for block production
sleep 5

# 4. Check status again (should be confirmed)
curl http://127.0.0.1:8080/tx/$TXID
# {"status":"confirmed","block_number":1,"block_hash":"..."}
```

---

## Code Patterns & Examples

### Phase 4.2: Config Loading Pattern
```rust
// config.rs
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub blockchain: BlockchainConfig,
}

#[derive(Deserialize, Serialize)]
pub struct BlockchainConfig {
    pub consensus_type: String,
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

// main.rs
let config = Config::load("aureon.toml")?;
let engine = match config.blockchain.consensus_type.as_str() {
    "pow" => Box::new(ProofOfWork::new()) as Box<dyn ConsensusEngine>,
    "pos" => Box::new(ProofOfStake::new()),
    _ => panic!("Unknown consensus type"),
};
```

### Phase 5.2: WebSocket Pattern
```rust
// api.rs - add WebSocket handler
use tokio_tungstenite::connect_async;

async fn subscribe_blocks(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|ws| handle_block_subscription(ws))
}

async fn handle_block_subscription(ws: WebSocket) {
    // On each block produced:
    // ws.send(Message::Text(json!(block))).await
}

// In consensus loop:
block_subscription_tx.send(block).ok();
```

### Phase 5.3: Transaction Integration Pattern
```rust
// types.rs - add Transaction type
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl TransactionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.from.is_empty() { return Err("empty from".into()); }
        if self.to.is_empty() { return Err("empty to".into()); }
        if self.amount == 0 { return Err("zero amount".into()); }
        Ok(())
    }
    
    pub fn to_transaction(&self) -> Transaction {
        Transaction {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount,
            nonce: 0, // Will be assigned by mempool
        }
    }
}

// In api handler:
async fn submit_tx(
    Json(req): Json<TransactionRequest>,
) -> Json<TransactionResponse> {
    req.validate()?;
    let tx = req.to_transaction();
    let tx_hash = tx.hash();
    MEMPOOL.lock().unwrap().add(tx);
    Json(TransactionResponse {
        status: "pending".to_string(),
        tx_hash,
    })
}
```

---

## Success Criteria for Each Phase

### Phase 4.2 Success
- [x] `cargo build` completes without errors
- [x] Node accepts `--config` argument
- [x] Loads consensus type from TOML
- [x] Produces blocks with selected engine
- [x] PoW and PoS both work via config

### Phase 5.2 Success
- [x] Block lookup returns real block data
- [x] Transaction lookup returns real tx data
- [x] WebSocket connects successfully
- [x] Block notifications sent in real-time
- [x] HTTP status codes correct (200, 400, 404, 500)

### Phase 5.3 Success
- [x] Transactions submitted to API are included in blocks
- [x] Transaction status progresses pending → confirmed
- [x] Transaction receipts queryable
- [x] Balances update correctly after confirmation
- [x] End-to-end flow: API → mempool → block → finality

---

## Branch Strategy (Optional)

If using git branches for tracking:

```bash
# Create feature branches
git checkout -b feature/phase-4.2-config
git checkout -b feature/phase-5.2-indexing
git checkout -b feature/phase-5.3-integration

# After each phase:
git merge feature/phaseX-Y --ff
git tag phase-X.Y-complete
```

---

## Environment Setup for Next Phase

### Dependencies to Add (Phase 4.2)
```toml
toml = "0.8"  # For TOML config parsing
```

### Dependencies to Add (Phase 5.2)
```toml
tokio-tungstenite = "0.20"  # WebSocket support
uuid = { version = "1.0", features = ["v4", "serde"] }  # Unique IDs
```

### No new dependencies needed for Phase 5.3

---

## Recommended Reading

### For Phase 4.2
- Tokio async patterns
- TOML specification
- Configuration best practices

### For Phase 5.2  
- WebSocket protocol (RFC 6455)
- Indexing strategies
- Event subscription patterns

### For Phase 5.3
- Mempool design
- Transaction lifecycle
- State finality concepts

---

## Questions to Consider

**Before Phase 4.2:**
- Should we support multiple config formats (TOML, YAML, JSON)?
- Should consensus settings be hot-reloadable?
- Should we validate config at startup?

**Before Phase 5.2:**
- Should block index be in-memory or persistent?
- Should WebSocket auth be required?
- How many concurrent subscribers per block?

**Before Phase 5.3:**
- What's the mempool max size?
- Should transactions have fees/gas prices?
- How long to keep pending transactions?

---

## Quick Reference

### Files to Modify Next
```
Phase 4.2:
  ├─ aureon-node/src/config.rs (NEW - 100 lines)
  ├─ aureon-node/src/main.rs (modify - 10 lines)
  └─ aureon-node/Cargo.toml (add toml dep)

Phase 5.2:
  ├─ aureon-node/src/api.rs (extend - 100 lines)
  └─ aureon-node/Cargo.toml (add tokio-tungstenite)

Phase 5.3:
  ├─ aureon-node/src/mempool.rs (NEW - 200 lines)
  ├─ aureon-node/src/api.rs (extend - 50 lines)
  ├─ aureon-node/src/types.rs (extend - 30 lines)
  └─ aureon-node/src/consensus/mod.rs (modify - 30 lines)
```

---

## Summary

**We are at a critical juncture:**
- ✅ Core blockchain infrastructure complete
- ✅ REST API framework ready
- ✅ WASM execution functional
- ⏳ Missing: Config system + Transaction flow

**The next 7-10 days will unlock full blockchain functionality.**

Recommended approach:
1. **Phase 4.2** (1-2 days) - Unblocks Node configuration
2. **Phase 5.2** (2-3 days) - Unblocks Production API
3. **Phase 5.3** (1-2 days) - Unblocks End-to-End Testing

After these three phases, Aureon will be a functional, testable blockchain.

---

**Ready to start Phase 4.2?** 🚀
