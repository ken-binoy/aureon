#!/bin/bash
# API Testing Script for Aureon

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_URL="http://127.0.0.1:8080"

echo -e "${YELLOW}=== Aureon REST API Test Suite ===${NC}\n"

# Test 1: Balance Query
echo -e "${YELLOW}Test 1: GET /balance/Alice${NC}"
curl -s $API_URL/balance/Alice | jq . || echo "❌ Failed"
echo ""

# Test 2: Balance Query - Different Account
echo -e "${YELLOW}Test 2: GET /balance/Bob${NC}"
curl -s $API_URL/balance/Bob | jq . || echo "❌ Failed"
echo ""

# Test 3: Chain Head
echo -e "${YELLOW}Test 3: GET /chain/head${NC}"
curl -s $API_URL/chain/head | jq . || echo "❌ Failed"
echo ""

# Test 4: Transaction Submission
echo -e "${YELLOW}Test 4: POST /submit-tx${NC}"
curl -s -X POST $API_URL/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"Alice","to":"Bob","amount":50}' | jq . || echo "❌ Failed"
echo ""

# Test 5: Invalid Transaction (empty from)
echo -e "${YELLOW}Test 5: POST /submit-tx (invalid - empty from)${NC}"
curl -s -X POST $API_URL/submit-tx \
  -H "Content-Type: application/json" \
  -d '{"from":"","to":"Bob","amount":50}' | jq . || echo "❌ Failed"
echo ""

# Test 6: Block Query
echo -e "${YELLOW}Test 6: GET /block/test_hash${NC}"
curl -s $API_URL/block/test_hash | jq . || echo "❌ Failed"
echo ""

# Test 7: Transaction Query
echo -e "${YELLOW}Test 7: GET /tx/test_tx_hash${NC}"
curl -s $API_URL/tx/test_tx_hash | jq . || echo "❌ Failed"
echo ""

echo -e "${GREEN}=== Test Suite Complete ===${NC}"
