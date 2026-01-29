#!/bin/bash

# HTTP Caching E2E Test Runner
# 모든 HTTP 캐싱 관련 E2E 테스트를 실행합니다.

set -e

# 색상 코드
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 테스트 결과 추적
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_TEST_NAMES=()

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                            ║${NC}"
echo -e "${BLUE}║         🧪 HTTP Caching E2E Test Suite Runner 🧪          ║${NC}"
echo -e "${BLUE}║                                                            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 현재 디렉토리 확인
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo -e "${YELLOW}📂 Working directory: ${SCRIPT_DIR}${NC}"
echo ""

# Python 의존성 확인
echo -e "${YELLOW}🐍 Checking Python dependencies...${NC}"

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ python3 not found${NC}"
    exit 1
fi

# requests 모듈 확인
if ! python3 -c "import requests" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  requests module not found. Installing...${NC}"
    pip3 install requests
fi

echo -e "${GREEN}✅ Python dependencies OK${NC}"
echo ""

# 서버 상태 확인
echo -e "${YELLOW}🔍 Checking server status...${NC}"
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${RED}❌ Server is not running at http://localhost:8080${NC}"
    echo -e "${YELLOW}💡 Please start the server first:${NC}"
    echo -e "${YELLOW}   cd ../pacs-server && ./target/debug/pacs_server${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Server is running${NC}"
echo ""

# 테스트 실행 함수
run_test() {
    local test_file=$1
    local test_name=$2
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}🧪 Running: ${test_name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    if python3 "$test_file"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo ""
        echo -e "${GREEN}✅ ${test_name} - PASSED${NC}"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("$test_name")
        echo ""
        echo -e "${RED}❌ ${test_name} - FAILED${NC}"
    fi
}

# 테스트 실행
echo -e "${YELLOW}🚀 Starting test execution...${NC}"
echo ""

# 1. User Role Assignment Cache Tests
if [ -f "test_user_role_assignment_cache_e2e.py" ]; then
    run_test "test_user_role_assignment_cache_e2e.py" "User Role Assignment Cache"
else
    echo -e "${YELLOW}⚠️  test_user_role_assignment_cache_e2e.py not found, skipping...${NC}"
fi

# 2. Role-Capability Matrix Cache Tests
if [ -f "test_role_capability_matrix_cache_e2e.py" ]; then
    run_test "test_role_capability_matrix_cache_e2e.py" "Role-Capability Matrix Cache"
else
    echo -e "${YELLOW}⚠️  test_role_capability_matrix_cache_e2e.py not found, skipping...${NC}"
fi

# 3. Capability Cache Tests
if [ -f "test_capability_cache_e2e.py" ]; then
    run_test "test_capability_cache_e2e.py" "Capability Cache"
else
    echo -e "${YELLOW}⚠️  test_capability_cache_e2e.py not found, skipping...${NC}"
fi

# 최종 결과 출력
echo ""
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                            ║${NC}"
echo -e "${BLUE}║                   📊 Test Results Summary                  ║${NC}"
echo -e "${BLUE}║                                                            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:   ${TOTAL_TESTS}"
echo -e "  ${GREEN}Passed:        ${PASSED_TESTS}${NC}"
echo -e "  ${RED}Failed:        ${FAILED_TESTS}${NC}"
echo ""

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}❌ Failed Tests:${NC}"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "${RED}   • ${test_name}${NC}"
    done
    echo ""
    echo -e "${RED}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                    ❌ TESTS FAILED ❌                      ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════╝${NC}"
    exit 1
else
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                            ║${NC}"
    echo -e "${GREEN}║            🎉 ALL TESTS PASSED! 🎉                        ║${NC}"
    echo -e "${GREEN}║                                                            ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    exit 0
fi

