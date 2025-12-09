#!/bin/bash
# Start the node and run basic API tests

echo "Starting Aureon Node..."
cd /Users/kenbinoy/aureon-chain

# Build first (if not already built)
echo "Building project..."
timeout 300 cargo build -p aureon-node --quiet 2>&1 &
BUILD_PID=$!

# Wait for build to complete or timeout
wait $BUILD_PID 2>/dev/null || true

# Start the node in background
echo "Starting node server..."
timeout 60 ./target/debug/aureon-node &
NODE_PID=$!

# Give server time to start
sleep 3

# Check if server is running
if kill -0 $NODE_PID 2>/dev/null; then
    echo "✅ Node started successfully (PID: $NODE_PID)"
    echo ""
    echo "Testing API endpoints..."
    echo ""
    
    # Test 1: Balance
    echo "1️⃣ Testing GET /balance/Alice"
    curl -s http://127.0.0.1:8080/balance/Alice | jq . 2>/dev/null || echo "❌ Failed"
    echo ""
    
    # Test 2: Chain Head
    echo "2️⃣ Testing GET /chain/head"
    curl -s http://127.0.0.1:8080/chain/head | jq . 2>/dev/null || echo "❌ Failed"
    echo ""
    
    # Test 3: Submit TX
    echo "3️⃣ Testing POST /submit-tx"
    curl -s -X POST http://127.0.0.1:8080/submit-tx \
      -H "Content-Type: application/json" \
      -d '{"from":"Alice","to":"Bob","amount":50}' | jq . 2>/dev/null || echo "❌ Failed"
    echo ""
    
    # Cleanup
    echo "Stopping node..."
    kill $NODE_PID 2>/dev/null || true
    echo "✅ Tests complete"
else
    echo "❌ Failed to start node"
fi
