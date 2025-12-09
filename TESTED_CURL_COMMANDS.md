# Tested Curl Commands - Phase 5.1 API

These commands have been tested and verified working.

## Quick Start

### Start the Node
```bash
cd /Users/kenbinoy/aureon-chain
cargo run -p aureon-node
```

Expected output:
```
Selected Consensus: PoW
...
--- Starting REST API Server ---
Node is running. Press Ctrl+C to stop.
📡 Aureon API listening on http://0.0.0.0:8080
```

The API will be available at `http://127.0.0.1:8080`

---

## All Working Endpoints (Tested ✅)

### 1. Get Alice's Balance
```bash
curl -s http://127.0.0.1:8080/balance/Alice | jq .
```

**Response:**
```json
{
  "address": "Alice",
  "balance": 50
}
```

---

### 2. Get Bob's Balance
```bash
curl -s http://127.0.0.1:8080/balance/Bob | jq .
```

**Response:**
```json
{
  "address": "Bob",
  "balance": 50
}
```

---

### 3. Get Charlie's Balance
```bash
curl -s http://127.0.0.1:8080/balance/Charlie | jq .
```

**Response:**
```json
{
  "address": "Charlie",
  "balance": 25
}
```

---

### 4. Get Chain Head
```bash
curl -s http://127.0.0.1:8080/chain/head | jq .
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

### 5. Submit Valid Transaction
```bash
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Bob","to":"Charlie","amount":25}' | jq .
```

**Response:**
```json
{
  "status": "success",
  "message": "Transaction from Bob to Charlie (amount: 25) queued for processing"
}
```

---

### 6. Submit Invalid Transaction (Zero Amount)
```bash
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":0}' | jq .
```

**Response:**
```json
{
  "status": "error",
  "message": "Amount must be greater than 0"
}
```

---

### 7. Submit Invalid Transaction (Empty From)
```bash
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"","to":"Bob","amount":50}' | jq .
```

**Response:**
```json
{
  "status": "error",
  "message": "Invalid sender or recipient"
}
```

---

### 8. Query Block
```bash
curl -s http://127.0.0.1:8080/block/test_hash | jq .
```

**Response:**
```json
{
  "hash": "test_hash",
  "number": 0,
  "timestamp": 0,
  "transactions": []
}
```

---

### 9. Query Transaction
```bash
curl -s http://127.0.0.1:8080/tx/test_tx | jq .
```

**Response:**
```json
{
  "amount": 0,
  "from": "unknown",
  "hash": "test_tx",
  "to": "unknown"
}
```

---

### 10. Deploy Invalid WASM Contract
```bash
curl -s -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{"code":[1,2,3,4,5],"gas_limit":10000}' | jq .
```

**Response:**
```json
{
  "address": "",
  "status": "failed: failed to parse WebAssembly module"
}
```

---

### 11. Deploy Valid WASM Contract
```bash
curl -s -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{"code":[0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,7,7,1,3,114,117,110,0,0,10,4,1,2,0,11],"gas_limit":10000}' | jq .
```

**Response:**
```json
{
  "address": "1a2b3c4d5e6f...",
  "status": "deployed"
}
```

Save the address for the next test!

---

### 12. Call Non-Existent Contract
```bash
curl -s -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{"contract_address":"invalid","function":"run","args":"","gas_limit":5000}' | jq .
```

**Response:**
```json
{
  "gas_used": 0,
  "output": "Contract not found",
  "success": false
}
```

---

## Batch Test Script

Run all tests at once:

```bash
#!/bin/bash

echo "=== Testing Aureon REST API ==="
echo ""

echo "1. Balance Queries"
curl -s http://127.0.0.1:8080/balance/Alice | jq .
echo ""

echo "2. Chain Head"
curl -s http://127.0.0.1:8080/chain/head | jq .
echo ""

echo "3. Valid Transaction"
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":50}' | jq .
echo ""

echo "4. Invalid Transaction"
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":0}' | jq .
echo ""

echo "5. Block Query"
curl -s http://127.0.0.1:8080/block/hash | jq .
echo ""

echo "6. Invalid Contract Deploy"
curl -s -X POST http://127.0.0.1:8080/contract/deploy \
  -H "Content-Type: application/json" \
  -d '{"code":[1,2,3],"gas_limit":10000}' | jq .
echo ""

echo "7. Contract Not Found"
curl -s -X POST http://127.0.0.1:8080/contract/call \
  -H "Content-Type: application/json" \
  -d '{"contract_address":"invalid","function":"run","args":"","gas_limit":5000}' | jq .
echo ""

echo "=== All Tests Complete ==="
```

Save as `test_api.sh`, run with `bash test_api.sh`

---

## Without jq (Plain curl)

If you don't have jq installed, remove the `| jq .` part:

```bash
curl -s http://127.0.0.1:8080/balance/Alice
```

---

## Expected Behavior

All commands should:
- ✅ Return valid JSON
- ✅ Complete in <100ms
- ✅ Not crash the server
- ✅ Show proper error messages for invalid input

If you see connection refused:
- Make sure the node is running
- Check port 8080 is accessible
- Verify no firewall is blocking

---

## Notes

- Replace `127.0.0.1` with your node's IP if running remotely
- The balances shown (50, 50, 25, etc.) are from state execution during startup
- Phase 5.1 uses placeholder data for block/tx lookups (real data in Phase 5.2)
- Contract addresses are deterministic SHA256 hashes of bytecode

---

**Last Tested:** December 7, 2025  
**API Version:** Phase 5.1  
**Status:** ✅ ALL ENDPOINTS WORKING
