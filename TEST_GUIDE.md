# Testing Phase 5.1 REST API

## Quick Test Guide

### Prerequisites
The node should be running on port 8080. If not already started, use:
```bash
cargo run -p aureon-node
```

### Manual API Tests

#### 1. Test Balance Query
```bash
curl http://127.0.0.1:8080/balance/Alice
```

**Expected Response:**
```json
{
  "address": "Alice",
  "balance": 100
}
```

---

#### 2. Test Bob's Balance
```bash
curl http://127.0.0.1:8080/balance/Bob
```

**Expected Response:**
```json
{
  "address": "Bob",
  "balance": 0
}
```

---

#### 3. Test Chain Head
```bash
curl http://127.0.0.1:8080/chain/head
```

**Expected Response:**
```json
{
  "chain_name": "Aureon",
  "best_block_number": 0,
  "best_block_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

---

#### 4. Submit a Transaction (Success Case)
```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":50}'
```

**Expected Response:**
```json
{
  "status": "success",
  "message": "Transaction from Alice to Bob (amount: 50) queued for processing"
}
```

---

#### 5. Submit a Transaction (Error - Zero Amount)
```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":0}'
```

**Expected Response:**
```json
{
  "status": "error",
  "message": "Amount must be greater than 0"
}
```

---

#### 6. Submit a Transaction (Error - Empty From)
```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"","to":"Bob","amount":50}'
```

**Expected Response:**
```json
{
  "status": "error",
  "message": "Invalid sender or recipient"
}
```

---

#### 7. Query Block
```bash
curl http://127.0.0.1:8080/block/abc123def456
```

**Expected Response:**
```json
{
  "hash": "abc123def456",
  "number": 0,
  "timestamp": 0,
  "transactions": []
}
```

---

#### 8. Query Transaction
```bash
curl http://127.0.0.1:8080/tx/tx_hash_123
```

**Expected Response:**
```json
{
  "hash": "tx_hash_123",
  "from": "unknown",
  "to": "unknown",
  "amount": 0
}
```

---

## Advanced Tests

### Deploy Contract Test
```bash
# Create a minimal WASM contract (using hex bytes for a simple module)
curl -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "code": [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b],
    "gas_limit": 10000
  }'
```

**Expected Response:**
```json
{
  "address": "a1b2c3d4e5f6...",
  "status": "deployed"
}
```

---

### Call Contract Test
```bash
# Using the address from the deploy response above
curl -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{
    "contract_address": "a1b2c3d4e5f6...",
    "function": "run",
    "args": "",
    "gas_limit": 5000
  }'
```

**Expected Response:**
```json
{
  "success": true,
  "output": "Contract executed successfully",
  "gas_used": 100
}
```

---

### Call Non-existent Contract
```bash
curl -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{
    "contract_address": "invalid_address",
    "function": "run",
    "args": "",
    "gas_limit": 5000
  }'
```

**Expected Response:**
```json
{
  "success": false,
  "output": "Contract not found",
  "gas_used": 0
}
```

---

## Testing with Python

```python
import requests
import json

BASE_URL = "http://127.0.0.1:8080"

# Test balance query
resp = requests.get(f"{BASE_URL}/balance/Alice")
print(f"Alice balance: {resp.json()}")

# Test transaction submission
resp = requests.post(
    f"{BASE_URL}/submit-tx",
    json={"from": "Alice", "to": "Bob", "amount": 50}
)
print(f"Submit TX: {resp.json()}")

# Test chain head
resp = requests.get(f"{BASE_URL}/chain/head")
print(f"Chain head: {resp.json()}")
```

---

## Testing with JavaScript/Node.js

```javascript
const BASE_URL = 'http://127.0.0.1:8080';

// Test balance query
fetch(`${BASE_URL}/balance/Alice`)
  .then(r => r.json())
  .then(data => console.log('Alice balance:', data));

// Test transaction submission
fetch(`${BASE_URL}/submit-tx`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    from: 'Alice',
    to: 'Bob',
    amount: 50
  })
})
  .then(r => r.json())
  .then(data => console.log('Submit TX:', data));

// Test chain head
fetch(`${BASE_URL}/chain/head`)
  .then(r => r.json())
  .then(data => console.log('Chain head:', data));
```

---

## Test Checklist

- [ ] **Balance Query**
  - [x] Alice has balance 100
  - [x] Bob has balance 0
  - [x] Invalid address returns 0

- [ ] **Chain Info**
  - [x] Chain name is "Aureon"
  - [x] Best block number is 0
  - [x] Block hash is genesis hash

- [ ] **Transaction Submission**
  - [x] Valid transaction succeeds
  - [x] Empty sender fails
  - [x] Empty recipient fails
  - [x] Zero amount fails

- [ ] **Block/TX Lookup**
  - [x] Block lookup returns structure
  - [x] TX lookup returns structure
  - [x] Both queries are fast (<1ms)

- [ ] **Contract Operations**
  - [x] Deploy valid WASM succeeds
  - [x] Deploy invalid bytecode fails
  - [x] Call existing contract succeeds
  - [x] Call non-existent contract fails

---

## Performance Benchmarks

Expected response times (on localhost):
- Balance query: <1ms
- Transaction submission: <1ms
- Chain head: <1ms
- Block lookup: <1ms
- TX lookup: <1ms
- Contract deployment: <100ms (WASM validation)
- Contract call: <1000ms (WASM execution)

If any endpoint takes significantly longer, check:
1. System load
2. Network latency
3. WASM contract complexity (for contract operations)

---

## Troubleshooting

### Port Already in Use
```bash
# Kill existing process on port 8080
lsof -i :8080 | grep LISTEN | awk '{print $2}' | xargs kill -9

# Or use a different port (requires code modification)
```

### Connection Refused
- Ensure node is running: `cargo run -p aureon-node`
- Check port 8080 is accessible: `netstat -an | grep 8080`
- Verify API server started (check node output)

### No Response / Timeout
- Check node logs for errors
- Ensure sufficient disk space for RocksDB
- Verify system resources (RAM, CPU)

### Invalid JSON Response
- Ensure `Content-Type: application/json` header is set
- Check request body is valid JSON
- Verify endpoint path is correct

---

## Next Test Steps (Phase 5.2)

Once Phase 5.2 is complete, test:
- [ ] Transaction indexing (GET /tx returns actual data)
- [ ] Block indexing (GET /block returns actual data)
- [ ] WebSocket subscriptions (ws://127.0.0.1:8080/subscribe)
- [ ] Real-time notifications on block production

---

## Notes

- Initial account balances: Alice=100, Charlie=100, others=0
- Database: RocksDB in `aureon_db/` directory
- Contracts are stored in registry with SHA256 addressing
- All responses use JSON format
- Error handling returns HTTP 200 with error status field (MVP approach)

---

**Status:** Ready for testing ✅
