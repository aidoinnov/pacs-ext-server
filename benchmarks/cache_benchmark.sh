#!/bin/bash

# HTTP Cache Performance Benchmark Script
# Compares performance with cache enabled vs disabled

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_URL="http://localhost:8080"
BENCHMARK_DURATION=30
THREADS=4
CONNECTIONS=100
WARMUP_DURATION=5

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}HTTP Cache Performance Benchmark${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo -e "${RED}Error: 'wrk' is not installed${NC}"
    echo "Install with: brew install wrk (macOS) or apt-get install wrk (Linux)"
    exit 1
fi

# Check if server is running
if ! curl -s "${SERVER_URL}/health" > /dev/null; then
    echo -e "${RED}Error: Server is not running at ${SERVER_URL}${NC}"
    echo "Start the server first: cd pacs-server && cargo run"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites met${NC}"
echo ""

# Function to run benchmark
run_benchmark() {
    local cache_enabled=$1
    local endpoint=$2
    local label=$3

    echo -e "${BLUE}Running benchmark: ${label}${NC}"
    echo "Cache: ${cache_enabled}, Endpoint: ${endpoint}"
    echo "Threads: ${THREADS}, Connections: ${CONNECTIONS}, Duration: ${BENCHMARK_DURATION}s"
    echo ""

    # Run wrk
    wrk -t${THREADS} -c${CONNECTIONS} -d${BENCHMARK_DURATION}s \
        --latency \
        "${SERVER_URL}${endpoint}" \
        2>&1 | tee /tmp/wrk_output_${cache_enabled}.txt

    echo ""
}

# Function to extract metrics
extract_metrics() {
    local file=$1

    # Extract requests/sec
    local req_per_sec=$(grep "Requests/sec:" "$file" | awk '{print $2}')

    # Extract latency
    local avg_latency=$(grep "Latency" "$file" | head -1 | awk '{print $2}')

    # Extract transfer/sec
    local transfer=$(grep "Transfer/sec:" "$file" | awk '{print $2}')

    echo "${req_per_sec}|${avg_latency}|${transfer}"
}

# Restart server function
restart_server_with_cache() {
    local enabled=$1

    echo -e "${YELLOW}Updating .env file...${NC}"
    cd /Users/aido/Code/pacs-ext-server/pacs-server

    # Update CACHE_ENABLED in .env
    if [ -f .env ]; then
        sed -i.bak "s/CACHE_ENABLED=.*/CACHE_ENABLED=${enabled}/" .env
    else
        echo "CACHE_ENABLED=${enabled}" >> .env
        echo "CACHE_TTL_SECONDS=300" >> .env
    fi

    echo -e "${YELLOW}Please restart the server with the new settings${NC}"
    echo "1. Stop the current server (Ctrl+C)"
    echo "2. Run: cargo run"
    echo "3. Press Enter when ready to continue..."
    read
}

# Create results directory
mkdir -p /Users/aido/Code/pacs-ext-server/benchmarks/results
RESULT_FILE="/Users/aido/Code/pacs-ext-server/benchmarks/results/cache_benchmark_$(date +%Y%m%d_%H%M%S).md"

# Start benchmark report
cat > "$RESULT_FILE" << EOF
# HTTP Cache Performance Benchmark Results

**Date**: $(date)
**Server**: ${SERVER_URL}
**Benchmark Tool**: wrk
**Configuration**:
- Threads: ${THREADS}
- Connections: ${CONNECTIONS}
- Duration: ${BENCHMARK_DURATION}s per test

---

EOF

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test 1: Cache ENABLED${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

restart_server_with_cache "true"

# Warmup
echo -e "${YELLOW}Warming up (${WARMUP_DURATION}s)...${NC}"
curl -s "${SERVER_URL}/health" > /dev/null
sleep ${WARMUP_DURATION}

# Test endpoints with cache enabled
run_benchmark "enabled" "/health" "Health endpoint (cache enabled)"
run_benchmark "enabled" "/api/users" "Users API (cache enabled)" || echo "Users endpoint may not exist"

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test 2: Cache DISABLED${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

restart_server_with_cache "false"

# Warmup
echo -e "${YELLOW}Warming up (${WARMUP_DURATION}s)...${NC}"
curl -s "${SERVER_URL}/health" > /dev/null
sleep ${WARMUP_DURATION}

# Test endpoints with cache disabled
run_benchmark "disabled" "/health" "Health endpoint (cache disabled)"
run_benchmark "disabled" "/api/users" "Users API (cache disabled)" || echo "Users endpoint may not exist"

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Analyzing Results${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Extract and compare metrics
if [ -f /tmp/wrk_output_enabled.txt ] && [ -f /tmp/wrk_output_disabled.txt ]; then
    metrics_enabled=$(extract_metrics /tmp/wrk_output_enabled.txt)
    metrics_disabled=$(extract_metrics /tmp/wrk_output_disabled.txt)

    req_enabled=$(echo $metrics_enabled | cut -d'|' -f1)
    lat_enabled=$(echo $metrics_enabled | cut -d'|' -f2)
    transfer_enabled=$(echo $metrics_enabled | cut -d'|' -f3)

    req_disabled=$(echo $metrics_disabled | cut -d'|' -f1)
    lat_disabled=$(echo $metrics_disabled | cut -d'|' -f2)
    transfer_disabled=$(echo $metrics_disabled | cut -d'|' -f3)

    # Calculate improvements
    req_improvement=$(echo "scale=2; ($req_enabled - $req_disabled) / $req_disabled * 100" | bc 2>/dev/null || echo "N/A")

    cat >> "$RESULT_FILE" << EOF
## Test Results

### Cache ENABLED

\`\`\`
$(cat /tmp/wrk_output_enabled.txt)
\`\`\`

### Cache DISABLED

\`\`\`
$(cat /tmp/wrk_output_disabled.txt)
\`\`\`

---

## Performance Comparison

| Metric | Cache Enabled | Cache Disabled | Improvement |
|--------|---------------|----------------|-------------|
| Requests/sec | ${req_enabled} | ${req_disabled} | ${req_improvement}% |
| Avg Latency | ${lat_enabled} | ${lat_disabled} | - |
| Transfer/sec | ${transfer_enabled} | ${transfer_disabled} | - |

---

## Analysis

EOF

    echo -e "${GREEN}Results saved to: ${RESULT_FILE}${NC}"
    echo ""
    echo -e "${BLUE}Summary:${NC}"
    echo "Cache Enabled:  ${req_enabled} req/sec, ${lat_enabled} latency"
    echo "Cache Disabled: ${req_disabled} req/sec, ${lat_disabled} latency"
    if [ "$req_improvement" != "N/A" ]; then
        echo -e "Improvement: ${GREEN}${req_improvement}%${NC}"
    fi
fi

# Cleanup
rm -f /tmp/wrk_output_*.txt

echo ""
echo -e "${GREEN}Benchmark complete!${NC}"
echo -e "Full report: ${RESULT_FILE}"
