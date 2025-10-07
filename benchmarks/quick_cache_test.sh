#!/bin/bash

# Quick Cache Test - Simplified version for immediate testing
# Tests a single endpoint with cache on/off

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SERVER_URL="http://localhost:8080"
ENDPOINT="/health"
DURATION=10
THREADS=4
CONNECTIONS=50

echo -e "${BLUE}==================================${NC}"
echo -e "${BLUE}Quick Cache Performance Test${NC}"
echo -e "${BLUE}==================================${NC}"
echo ""

# Check prerequisites
if ! command -v wrk &> /dev/null; then
    echo -e "${RED}wrk not installed. Install with: brew install wrk${NC}"
    exit 1
fi

if ! curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}Server not running at ${SERVER_URL}${NC}"
    echo "Start with: cd pacs-server && cargo run"
    exit 1
fi

echo -e "${GREEN}✓ Server is running${NC}"
echo ""

# Check current cache setting
echo -e "${YELLOW}Checking current cache setting...${NC}"
CACHE_HEADER=$(curl -s -I "${SERVER_URL}${ENDPOINT}" | grep -i "cache-control" || echo "")

if echo "$CACHE_HEADER" | grep -q "max-age"; then
    CURRENT_STATE="ENABLED"
    echo -e "Current: ${GREEN}Cache ENABLED${NC}"
    echo "$CACHE_HEADER"
else
    CURRENT_STATE="DISABLED"
    echo -e "Current: ${YELLOW}Cache DISABLED${NC}"
    echo "$CACHE_HEADER"
fi

echo ""
echo -e "${BLUE}Running benchmark (${DURATION}s)...${NC}"
echo "Endpoint: ${ENDPOINT}"
echo "Threads: ${THREADS}, Connections: ${CONNECTIONS}"
echo ""

# Run benchmark
wrk -t${THREADS} -c${CONNECTIONS} -d${DURATION}s \
    --latency \
    "${SERVER_URL}${ENDPOINT}" 2>&1 | tee /tmp/quick_cache_test.txt

echo ""
echo -e "${BLUE}Results for Cache ${CURRENT_STATE}:${NC}"
echo ""

# Extract key metrics
REQ_SEC=$(grep "Requests/sec:" /tmp/quick_cache_test.txt | awk '{print $2}')
AVG_LAT=$(grep "Latency" /tmp/quick_cache_test.txt | head -1 | awk '{print $2}')
TRANSFER=$(grep "Transfer/sec:" /tmp/quick_cache_test.txt | awk '{print $2}')

echo -e "${GREEN}Requests/sec:${NC} ${REQ_SEC}"
echo -e "${GREEN}Avg Latency:${NC}  ${AVG_LAT}"
echo -e "${GREEN}Transfer/sec:${NC} ${TRANSFER}"
echo ""

# Save result
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="/Users/aido/Code/pacs-ext-server/benchmarks/results/quick_test_${CURRENT_STATE}_${TIMESTAMP}.txt"
mkdir -p /Users/aido/Code/pacs-ext-server/benchmarks/results
cp /tmp/quick_cache_test.txt "$RESULT_FILE"

echo -e "${BLUE}Result saved to:${NC} ${RESULT_FILE}"
echo ""

# Instructions for comparison
if [ "$CURRENT_STATE" == "ENABLED" ]; then
    echo -e "${YELLOW}To test with cache DISABLED:${NC}"
    echo "1. Edit pacs-server/.env: CACHE_ENABLED=false"
    echo "2. Restart server: cargo run"
    echo "3. Run this script again"
else
    echo -e "${YELLOW}To test with cache ENABLED:${NC}"
    echo "1. Edit pacs-server/.env: CACHE_ENABLED=true"
    echo "2. Restart server: cargo run"
    echo "3. Run this script again"
fi

rm -f /tmp/quick_cache_test.txt
