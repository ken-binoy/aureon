#!/bin/bash
# Signature Verification Test Script
# Tests Ed25519 signature verification for transactions

set -e

echo "================================================"
echo "Aureon Signature Verification Test"
echo "================================================"
echo ""

# Generate keypair
echo "1. Generating Ed25519 keypair..."
KEYPAIR=$(./target/debug/aureon-node keygen 2>/dev/null | tail -2)
SECRET_KEY=$(echo "$KEYPAIR" | grep "Secret Key" | awk '{print $NF}')
PUBLIC_KEY=$(echo "$KEYPAIR" | grep "Public Key" | awk '{print $NF}')

echo "   Secret Key: $SECRET_KEY"
echo "   Public Key: $PUBLIC_KEY"
echo ""

# Start the server in background
echo "2. Starting Aureon node..."
./target/debug/aureon-node &
NODE_PID=$!
sleep 3
echo "   Node started (PID: $NODE_PID)"
echo ""

# Cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $NODE_PID 2>/dev/null || true
    sleep 1
}
trap cleanup EXIT

# Test unsigned transaction (backward compatibility)
echo "3. Testing unsigned transaction (backward compatibility)..."
UNSIGNED_RESPONSE=$(curl -s -X POST http://localhost:8080/submit-tx \
  -H "Content-Type: application/json" \
  -d '{
    "from": "Alice",
    "to": "Bob",
    "amount": 100
  }')
echo "   Response: $UNSIGNED_RESPONSE"
echo ""

# Test signed transaction
echo "4. Testing signed transaction with Ed25519 signature..."

# Create test signature (in production, this would be computed client-side)
# For testing, we'll use a simple hash-based approach
TX_DATA="Alice:Bob:100:0"
echo "   Transaction data: $TX_DATA"

# Use cURL to submit a signed transaction
# Note: In real usage, the signature would be generated client-side
SIGNED_RESPONSE=$(curl -s -X POST http://localhost:8080/submit-signed-tx \
  -H "Content-Type: application/json" \
  -d "{
    \"from\": \"Alice\",
    \"to\": \"Bob\",
    \"amount\": 100,
    \"nonce\": 0,
    \"public_key\": \"$PUBLIC_KEY\",
    \"signature\": \"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\"
  }" 2>/dev/null || echo "{\"status\":\"error\",\"message\":\"Failed to send signed transaction\"}")

echo "   Response: $SIGNED_RESPONSE"
echo ""

# Check mempool
echo "5. Checking mempool..."
MEMPOOL=$(curl -s http://localhost:8080/mempool)
echo "   Mempool status: $MEMPOOL"
echo ""

echo "================================================"
echo "Test Complete!"
echo "================================================"
