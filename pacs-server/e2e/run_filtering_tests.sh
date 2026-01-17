#!/bin/bash
# 필터링 관련 E2E 테스트 실행 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 필터링 관련 E2E 테스트 실행"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=()

run_test() {
    local test_file=$1
    local test_name=$2
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo ""
    echo "=========================================="
    echo "📋 테스트: $test_name"
    echo "=========================================="
    
    if python3 "$test_file"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        FAILED_TESTS+=("$test_name")
        return 1
    fi
}

set +e

run_test "test_annotation_level_filtering.py" "레벨 필터링 (Study/Series/Instance)"
run_test "test_annotation_permission_filtering.py" "권한 기반 필터링"

echo ""
echo "=========================================="
echo "📊 필터링 테스트 결과"
echo "=========================================="
echo "총 테스트: $TOTAL_TESTS"
echo -e "${GREEN}통과: $PASSED_TESTS${NC}"
echo -e "${RED}실패: ${#FAILED_TESTS[@]}${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}실패한 테스트:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "${RED}  - $test${NC}"
    done
    echo ""
    exit 1
else
    echo ""
    echo -e "${GREEN}🎉 모든 필터링 테스트 통과!${NC}"
    echo ""
    exit 0
fi

