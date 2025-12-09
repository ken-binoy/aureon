# Aureon Blockchain - Prioritized Implementation Roadmap

**Current Completion:** 65-70%  
**Target for Next Phase:** 85-90% (Production MVP)  
**Estimated Effort:** 10-14 days  
**Date:** December 7, 2025

---

## Priority Matrix

```
┌─────────────────────────────────────────────────────┐
│  CRITICAL & QUICK          │  CRITICAL & COMPLEX    │
│  (Do First)                │  (Do Second)           │
│─────────────────────────────────────────────────────│
│ • Nonce Enforcement        │ • P2P Block Sync       │
│ • Signature Verification   │                        │
│─────────────────────────────────────────────────────│
│  HIGH & QUICK              │  HIGH & COMPLEX        │
│  (Do Third)                │  (Do Later)            │
│─────────────────────────────────────────────────────│
│ • WebSocket Wiring         │ • Multi-node Testing   │
│ • build.rs Automation      │ • Key Management       │
│─────────────────────────────────────────────────────│
│  NICE-TO-HAVE              │  RESEARCH              │
│  (Polish)                  │  (Future)              │
│─────────────────────────────────────────────────────│
│ • CLI Client               │ • Advanced Slashing    │
│ • Metrics/Monitoring       │ • Privacy Enhancements │
└─────────────────────────────────────────────────────┘
```

---

## Phase 6.1: Security Foundation (3-4 days) 🔒

### Objective
Implement cryptographic security for transactions and blocks.

### Task 1: Signature Verification (2-3 days)

**What to implement:**
```rust
// In types.rs
pub struct Transaction {
    from: String,
    to: String,
    nonce: u64,
    amount: u64,
    signature: Vec<u8>,  // Currently empty!
    // ... other fields
}

// New implementations needed:
1. Keypair generation (Ed25519)
2. Transaction signing
3. Signature verification
4. Serialization for hashing
```

**Files to modify:**
- `aureon-node/src/types.rs` - Add KeyPair, signing methods
- `aureon-node/src/api.rs` - Sign transactions in submit_tx
- `aureon-node/src/mempool.rs` - Verify signatures on add
- `aureon-node/src/state_processor.rs` - Verify in block validation

**Dependencies to add:**
```toml
ed25519-dalek = "2.0"
sha2 = "0.10"
```

**Tests needed:**
- [ ] Keypair generation
- [ ] Sign & verify transaction
- [ ] Reject invalid signatures
- [ ] Reject unsigned transactions
- [ ] Mempool rejects unsigned txs

### Task 2: Nonce Enforcement (1 day)

**What to implement:**
```rust
// In mempool.rs
pub struct TransactionMempool {
    // ... existing fields
    account_nonces: HashMap<String, u64>,  // Track per-account nonce
}

// Add validation:
pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
    let current_nonce = self.account_nonces.get(&tx.from).copied().unwrap_or(0);
    if tx.nonce <= current_nonce {
        return Err("Nonce too low or duplicate".into());
    }
    Ok(())
}
```

**Files to modify:**
- `aureon-node/src/mempool.rs` - Track & validate nonces
- `aureon-node/src/state_processor.rs` - Update nonce on tx apply

**Tests needed:**
- [ ] Accept monotonically increasing nonce
- [ ] Reject nonce too low
- [ ] Reject duplicate nonce
- [ ] Reject out-of-order transactions
- [ ] Update nonce after inclusion

### Task 3: Block Size Limits (0.5 days)

**What to implement:**
```rust
// In consensus/mod.rs
const MAX_BLOCK_SIZE: usize = 1_000_000; // 1MB
const MAX_TRANSACTIONS_PER_BLOCK: usize = 1000;

pub fn validate_block_size(block: &Block) -> Result<()> {
    if block.transactions.len() > MAX_TRANSACTIONS_PER_BLOCK {
        return Err("Too many transactions".into());
    }
    let size = bincode::serialize(block)?.len();
    if size > MAX_BLOCK_SIZE {
        return Err("Block too large".into());
    }
    Ok(())
}
```

**Files to modify:**
- `aureon-node/src/consensus/mod.rs` - Add size validation
- `aureon-node/src/block_producer.rs` - Enforce in produce_block

**Tests needed:**
- [ ] Accept blocks under limit
- [ ] Reject blocks over tx limit
- [ ] Reject blocks over size limit

---

## Phase 6.2: P2P Block Synchronization (3-4 days) 🌐

### Objective
Enable multi-node networks to synchronize blocks and state.

### Task 1: Block Sync Protocol (2 days)

**What to implement:**
```rust
// In network/message.rs - Add new message types
pub enum P2PMessage {
    // ... existing types
    BlockRequest { start_height: u64, count: u64 },
    BlockResponse { blocks: Vec<Block> },
    StatusRequest,
    StatusResponse { height: u64, hash: String, state_root: Vec<u8> },
}

// In network/mod.rs
pub struct PeerManager {
    peers: HashMap<SocketAddr, PeerState>,
}

pub struct PeerState {
    best_height: u64,
    best_hash: String,
    last_sync: Instant,
}
```

**Implementation steps:**
1. [ ] Extend network message types
2. [ ] Implement request/response handlers
3. [ ] Add peer tracking (best height/hash)
4. [ ] Implement block request logic
5. [ ] Implement block application logic

**Files to modify:**
- `aureon-node/src/network/message.rs` - New message types
- `aureon-node/src/network/mod.rs` - Protocol logic
- `aureon-node/src/main.rs` - Handle sync on peer connect

**Tests needed:**
- [ ] Request blocks by range
- [ ] Receive & validate blocks
- [ ] Apply blocks in order
- [ ] Verify state root after each
- [ ] Handle missing blocks
- [ ] Handle invalid blocks

### Task 2: Initial Sync (1.5 days)

**What to implement:**
```rust
// New sync module
pub struct BlockSync {
    target_height: u64,
    current_height: u64,
    pending_blocks: HashMap<u64, Block>,
}

impl BlockSync {
    pub async fn sync_to_height(&mut self, peer: &Peer, height: u64) {
        // Request blocks in batches
        // Apply each block with validation
        // Update current_height
    }
}
```

**Implementation steps:**
1. [ ] Create sync module
2. [ ] Request blocks from peer with higher height
3. [ ] Queue and apply blocks sequentially
4. [ ] Verify state root after each block
5. [ ] Handle reorg if needed (for later)
6. [ ] Resume from checkpoint on restart

**Files to create:**
- `aureon-node/src/sync.rs` - Block synchronization logic

**Tests needed:**
- [ ] Sync from genesis to head
- [ ] Sync from partial state
- [ ] Handle peer disconnection during sync
- [ ] Verify all blocks applied correctly

### Task 3: Peer Status Exchange (0.5 days)

**What to implement:**
```rust
// Periodic status exchange
pub async fn broadcast_status(node: &Node) {
    let status = StatusResponse {
        height: node.best_block.number,
        hash: node.best_block.hash.clone(),
        state_root: node.state_root.clone(),
    };
    node.network.broadcast(P2PMessage::StatusResponse(status));
}

// On receiving status
pub fn on_status_received(peer: &mut Peer, status: StatusResponse) {
    peer.best_height = status.height;
    peer.best_hash = status.hash;
    
    // If peer is ahead, initiate sync
    if status.height > node.best_height {
        sync.request_blocks(peer, node.best_height + 1, status.height);
    }
}
```

**Files to modify:**
- `aureon-node/src/network/mod.rs` - Status handlers
- `aureon-node/src/main.rs` - Status broadcast loop

---

## Phase 5.4: Real-Time Updates (1-2 days) 📡

### Objective
Enable clients to subscribe to blockchain events.

### Task 1: WebSocket Subscriptions (1-2 days)

**What to implement:**
```rust
// In api.rs - Add subscription handler
pub struct WebSocketState {
    subscribers: Arc<Mutex<HashMap<String, Vec<tokio::sync::mpsc::Sender<String>>>>>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    state: AxumState<Arc<WebSocketState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// When new block produced
pub fn broadcast_new_block(block: &Block, state: &WebSocketState) {
    let message = serde_json::json!({
        "type": "new_block",
        "block": {
            "number": block.number,
            "hash": block.hash,
            "timestamp": block.timestamp,
            "transactions": block.transactions.len(),
        }
    });
    
    // Send to all new_blocks subscribers
    state.broadcast("new_blocks", message);
}
```

**Implementation steps:**
1. [ ] Create WebSocketState struct
2. [ ] Implement WebSocket upgrade handler
3. [ ] Add subscription/unsubscription logic
4. [ ] Wire block producer to broadcast
5. [ ] Add tx status tracking
6. [ ] Test with WebSocket client

**Files to modify:**
- `aureon-node/src/api.rs` - WebSocket handler
- `aureon-node/src/block_producer.rs` - Broadcast on new block
- `aureon-node/src/main.rs` - Initialize WebSocket state

**Channels:**
- `new_blocks` - New block headers
- `tx_status:{hash}` - Transaction status changes
- `logs` - Contract event logs (later)

**Tests needed:**
- [ ] Connect to WebSocket
- [ ] Receive new block notifications
- [ ] Subscribe/unsubscribe
- [ ] Multiple subscribers
- [ ] Handle disconnections

---

## Phase 7.1: Developer Experience (1-2 days) 🔧

### Task 1: build.rs Contract Compilation (1 day)

**What to implement:**
```rust
// aureon-node/build.rs
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Find all .wat files
    let contracts_dir = Path::new("src/contracts");
    
    for entry in fs::read_dir(contracts_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "wat") {
            // Compile .wat to .wasm
            let wasm_path = path.with_extension("wasm");
            
            Command::new("wat2wasm")
                .arg(&path)
                .arg("-o")
                .arg(&wasm_path)
                .output()
                .expect("Failed to compile WAT");
                
            println!("cargo:warning=Compiled {:?}", path);
        }
    }
}
```

**Files to create:**
- `aureon-node/build.rs` - Contract compilation

**Dependencies to add:**
```toml
[build-dependencies]
# wabt can be compiled from source or use system binary
```

**Tests needed:**
- [ ] .wat files auto-compile
- [ ] Compilation errors reported
- [ ] .wasm files generated
- [ ] Binary includes compiled contracts

### Task 2: docker-compose for Multi-Node Testing (1 day)

**What to create:**
```yaml
# docker-compose.yml
version: '3.8'

services:
  node1:
    build: .
    environment:
      NODE_ID: 1
      BOOTSTRAP_PEERS: "node2:6001,node3:6002"
    ports:
      - "8001:8080"
    volumes:
      - node1_data:/data
  
  node2:
    build: .
    environment:
      NODE_ID: 2
      BOOTSTRAP_PEERS: "node1:6000,node3:6002"
    ports:
      - "8002:8080"
    volumes:
      - node2_data:/data
  
  node3:
    build: .
    environment:
      NODE_ID: 3
      BOOTSTRAP_PEERS: "node1:6000,node2:6001"
    ports:
      - "8003:8080"
    volumes:
      - node3_data:/data

volumes:
  node1_data:
  node2_data:
  node3_data:
```

**Files to create:**
- `docker-compose.yml` - 3-node testnet
- `Dockerfile` - Build configuration
- `.dockerignore` - Build optimization

**Tests needed:**
- [ ] 3 nodes start correctly
- [ ] Nodes discover each other
- [ ] Blocks propagate between nodes
- [ ] State stays in sync
- [ ] Restart/recovery works

---

## Phase 8: Polish & Hardening (Later)

### Optional Items
- CLI client tool
- Prometheus metrics
- Better logging with tracing crate
- Performance optimization
- More comprehensive tests
- Documentation improvements

---

## Implementation Checklist

### Immediate Actions (This Week)

- [ ] **Create security roadmap document** ← Start here
- [ ] **Implement Ed25519 signature verification** (2-3 days)
  - [ ] Keypair generation
  - [ ] Transaction signing
  - [ ] Signature validation
  - [ ] Integration tests
  
- [ ] **Add nonce enforcement** (1 day)
  - [ ] Per-account nonce tracking
  - [ ] Validation in mempool
  - [ ] Update on tx inclusion
  - [ ] Unit tests

- [ ] **Add block size limits** (0.5 days)
  - [ ] Constant definitions
  - [ ] Validation in consensus
  - [ ] Tests

### Following Week

- [ ] **Implement P2P block sync** (3-4 days)
  - [ ] BlockRequest/Response messages
  - [ ] Peer tracking
  - [ ] Sync state machine
  - [ ] Integration tests

- [ ] **Wire WebSocket subscriptions** (1-2 days)
  - [ ] WebSocket handler
  - [ ] Subscription management
  - [ ] Broadcast on events
  - [ ] Client tests

### Future (After 2 Weeks)

- [ ] build.rs contract compilation
- [ ] docker-compose multi-node setup
- [ ] CLI client
- [ ] Metrics & monitoring
- [ ] Advanced features (slashing, privacy, etc.)

---

## Estimated Timeline to 85% Complete

```
Week 1:
├─ Mon-Tue: Signature Verification (2-3 days)
├─ Wed: Nonce Enforcement (1 day)
├─ Wed-Thu: Block Size Limits (0.5 days)
└─ Thu-Fri: Testing & debugging

Week 2:
├─ Mon-Wed: P2P Block Sync (3-4 days)
├─ Thu: WebSocket Wiring (1 day)
└─ Fri: Testing & documentation

Estimated Total: 10-14 days
Target Completion: 85% by Dec 21, 2025
```

---

## Success Criteria

### Definition of Done for Each Phase

**Phase 6.1 (Security):**
- ✅ All transactions have valid signatures
- ✅ All blocks reject unsigned/invalid txs
- ✅ Nonces enforced per account
- ✅ Block size limited
- ✅ 100% of security tests pass

**Phase 6.2 (Networking):**
- ✅ 3 nodes can sync blocks
- ✅ State remains consistent across nodes
- ✅ No blocks lost during sync
- ✅ Recovery from peer disconnect

**Phase 5.4 (WebSockets):**
- ✅ Clients receive new block notifications
- ✅ TX status updates work
- ✅ Multiple subscribers supported
- ✅ Connection management

**Phase 7.1 (DevOps):**
- ✅ Contracts auto-compile via build.rs
- ✅ docker-compose launches 3-node testnet
- ✅ All nodes synced after startup
- ✅ docs updated

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Signature implementation bugs | Extensive unit tests, comparison with reference |
| P2P sync race conditions | Careful state machine design, race testing |
| WebSocket memory leaks | Proper subscriber cleanup, connection limits |
| Breaking changes | Backward compatibility tests, version tracking |

---

## Final Recommendation

**Start with Phase 6.1 (Security) immediately.** This is the critical blocker preventing multi-node operation. Once signatures and nonce validation are in place:

1. Phase 6.2 (P2P Sync) becomes safe to implement
2. Phase 5.4 (WebSockets) can be done in parallel
3. Phase 7.1 (DevOps) provides testing infrastructure

**Target:** 85% completion in 2 weeks, ready for production MVP in 3-4 weeks.

The foundation is excellent. These remaining pieces are well-understood and achievable.
