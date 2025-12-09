# Aureon REST API - Quick Reference Guide

## Server Setup
The REST API is now integrated into the main node and starts automatically on port 8080.

### Start the Node
```bash
cargo run --bin aureon-node
```

### Expected Output
```
📡 Aureon API listening on http://0.0.0.0:8080 (access via http://127.0.0.1:8080 locally)
```

## API Endpoints

### 1. GET Balance
Retrieve account balance from blockchain state.

```bash
curl http://127.0.0.1:8080/balance/Alice
```

**Response:**
```json
{
  "address": "Alice",
  "balance": 100
}
```

---

### 2. POST Submit Transaction
Submit a transfer transaction to the network.

```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{
    "from": "Alice",
    "to": "Bob",
    "amount": 50
  }'
```

**Response:**
```json
{
  "status": "success",
  "message": "Transaction from Alice to Bob (amount: 50) queued for processing"
}
```

---

### 3. GET Block Info
Retrieve information about a block by hash.

```bash
curl http://127.0.0.1:8080/block/abc123def456
```

**Response:**
```json
{
  "hash": "abc123def456",
  "number": 0,
  "timestamp": 0,
  "transactions": []
}
```

---

### 4. GET Transaction Info
Retrieve information about a transaction by hash.

```bash
curl http://127.0.0.1:8080/tx/tx_hash_here
```

**Response:**
```json
{
  "hash": "tx_hash_here",
  "from": "unknown",
  "to": "unknown",
  "amount": 0
}
```

---

### 5. GET Chain Head
Get current blockchain state (best block).

```bash
curl http://127.0.0.1:8080/chain/head
```

**Response:**
```json
{
  "chain_name": "Aureon",
  "best_block_number": 0,
  "best_block_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

---

### 6. POST Deploy Contract
Deploy a new smart contract to the blockchain.

```bash
curl -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "code": [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
    "gas_limit": 10000
  }'
```

**Response (success):**
```json
{
  "address": "1a2b3c4d5e6f7g8h9i0j",
  "status": "deployed"
}
```

**Response (invalid WASM):**
```json
{
  "address": "",
  "status": "failed: invalid WASM module"
}
```

---

### 7. POST Call Contract
Execute a deployed smart contract.

```bash
curl -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{
    "contract_address": "1a2b3c4d5e6f7g8h9i0j",
    "function": "run",
    "args": "",
    "gas_limit": 5000
  }'
```

**Response (success):**
```json
{
  "success": true,
  "output": "Contract executed successfully",
  "gas_used": 1250
}
```

**Response (not found):**
```json
{
  "success": false,
  "output": "Contract not found",
  "gas_used": 0
}
```

---

## Testing Workflow

### 1. Check Initial Balances
```bash
curl http://127.0.0.1:8080/balance/Alice
curl http://127.0.0.1:8080/balance/Bob
```

### 2. Submit Transfers
```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":30}'
```

### 3. Deploy a Contract
```bash
# First, create a simple WASM contract (or use an existing one)
# Then encode to byte array and deploy

curl -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "code": [/* WASM bytecode */],
    "gas_limit": 10000
  }'

# Save the returned address
```

### 4. Call the Contract
```bash
curl -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{
    "contract_address": "returned_address_from_step_3",
    "function": "run",
    "args": "",
    "gas_limit": 5000
  }'
```

### 5. Check Final Balances
```bash
curl http://127.0.0.1:8080/balance/Alice
curl http://127.0.0.1:8080/balance/Bob
```

---

## Integration with Other Tools

### Using curl
```bash
curl -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":50}'
```

### Using Python requests
```python
import requests

response = requests.post(
    'http://127.0.0.1:8080/submit-tx',
    json={
        'from': 'Alice',
        'to': 'Bob',
        'amount': 50
    }
)
print(response.json())
```

### Using JavaScript fetch
```javascript
fetch('http://127.0.0.1:8080/balance/Alice')
  .then(r => r.json())
  .then(data => console.log(data))
```

---

## Notes

- Initial accounts: Alice (100), Bob (0), Charlie (100), Dave (0)
- All responses are JSON format
- Contract code must be valid WASM bytecode
- Gas limit affects execution duration and cost
- Changes to balances/state are persisted in RocksDB

---

## Next Steps (Phase 5.2)

Coming soon:
- ✅ Transaction/Block indexing for lookup endpoints
- ✅ WebSocket subscriptions for real-time events
- ✅ Proper HTTP status codes (400/500 errors)
- ✅ Transaction receipt tracking
- ✅ Advanced query filtering

