#!/bin/bash
# Test script for the MCP HTTP Server
# This script demonstrates how to interact with the HTTP server

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

HTTP_HOST="${HTTP_HOST:-localhost}"
HTTP_PORT="${HTTP_PORT:-3000}"
BASE_URL="http://${HTTP_HOST}:${HTTP_PORT}"

echo -e "${BLUE}=== MCP HTTP Server Test Script ===${NC}\n"

# Test 1: Health Check
echo -e "${BLUE}Test 1: Health Check${NC}"
echo "GET ${BASE_URL}/health"
curl -s "${BASE_URL}/health"
echo -e "\n${GREEN}✓ Health check passed${NC}\n"

# Test 2: Initialize
echo -e "${BLUE}Test 2: Initialize${NC}"
INIT_REQUEST='{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "sampling": {},
      "roots": { "listChanged": true }
    },
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  }
}'

echo "Request:"
echo "${INIT_REQUEST}" | jq .
echo ""

curl -s -X POST "${BASE_URL}/mcp" \
  -H "Content-Type: application/json" \
  -d "${INIT_REQUEST}" | jq .
echo -e "${GREEN}✓ Initialize passed${NC}\n"

# Test 3: List Tools
echo -e "${BLUE}Test 3: List Available Tools${NC}"
LIST_TOOLS_REQUEST='{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}'

echo "Request:"
echo "${LIST_TOOLS_REQUEST}" | jq .
echo ""

curl -s -X POST "${BASE_URL}/mcp" \
  -H "Content-Type: application/json" \
  -d "${LIST_TOOLS_REQUEST}" | jq .
echo -e "${GREEN}✓ List tools passed${NC}\n"

# Test 4: Call a Tool (Get Wazuh Agents)
echo -e "${BLUE}Test 4: Call Tool - Get Wazuh Agents${NC}"
CALL_TOOL_REQUEST='{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_wazuh_agents",
    "arguments": {
      "limit": 5
    }
  }
}'

echo "Request:"
echo "${CALL_TOOL_REQUEST}" | jq .
echo ""

curl -s -X POST "${BASE_URL}/mcp" \
  -H "Content-Type: application/json" \
  -d "${CALL_TOOL_REQUEST}" | jq .
echo -e "${GREEN}✓ Call tool passed${NC}\n"

# Test 5: Call Tool - Get Alert Summary
echo -e "${BLUE}Test 5: Call Tool - Get Wazuh Alert Summary${NC}"
ALERT_REQUEST='{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "get_wazuh_alert_summary",
    "arguments": {
      "limit": 10
    }
  }
}'

echo "Request:"
echo "${ALERT_REQUEST}" | jq .
echo ""

curl -s -X POST "${BASE_URL}/mcp" \
  -H "Content-Type: application/json" \
  -d "${ALERT_REQUEST}" | jq .
echo -e "${GREEN}✓ Get alerts passed${NC}\n"

echo -e "${GREEN}=== All tests completed! ===${NC}"
