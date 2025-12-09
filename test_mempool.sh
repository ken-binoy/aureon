#!/bin/bash

# Start the node in background
./target/release/aureon-node &
NODE_PID=$!

# Wait for node to start
sleep 3

echo "=== Testing Mempool Integration ==="
echo ""

# Test 1: Check empty mempool
echo "1. Checking initial mempool state:"
curl -s http://127.0.0.1:8080/mempool | jq .
echo ""

# Test 2: Submit a transaction
echo "2. Submitting transaction to mempool:"
curl -s -X POST http://127.0.0.1:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice",
    "to": "bob",
    "amount": 100
  }' | jq .
echo ""

# Test 3: Check mempool after submission
echo "3. Checking mempool after transaction submission:"
curl -s http://127.0.0.1:8080/mempool | jq .
echo ""

# Test 4: Submit multiple transactions
echo "4. Submitting 3 more transactions:"
for i in 1 2 3; do
  curl -s -X POST http://127.0.0.1:8080/submit-tx \
    -H "Content-Type: application/json" \
    -d "{
      \"from\": \"alice\",
      \"to\": \"user$i\",
      \"amount\": $((100 + i * 10))
    }" > /dev/null
  echo "  Transaction $i submitted"
done
echo ""

# Test 5: Check final mempool state
echo "5. Final mempool state with all transactions:"
curl -s http://127.0.0.1:8080/mempool | jq .
echo ""

# Cleanup
kill $NODE_PID 2>/dev/null || true
wait $NODE_PID 2>/dev/null || true

echo "✅ Mempool integration test complete!"
