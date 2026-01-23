#!/bin/bash
# 캐시 관련 E2E 테스트 실행 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 캐시 관련 E2E 테스트 실행"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# 테스트 결과 추적
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=()

# 테스트 실행 함수
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

# 각 테스트 실행 (실패해도 계속 진행)
set +e

echo ""
echo "=========================================="
echo "📦 Annotation 캐시 테스트"
echo "=========================================="
run_test "test_annotation_head_request.py" "HEAD 요청 및 캐시 검증"

echo ""
echo "=========================================="
echo "⚡ HTTP Caching 테스트"
echo "=========================================="
run_test "test_capability_cache_e2e.py" "Capability API Cache"

echo ""
echo "=========================================="
echo "🚀 QIDO Redis Caching 테스트"
echo "=========================================="
run_test "test_qido_cache_e2e.py" "QIDO Cache (Series & Studies)"

# 최종 결과 출력
echo ""
echo "=========================================="
echo "📊 테스트 결과 요약"
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
    echo -e "${GREEN}🎉 모든 캐시 테스트 통과!${NC}"
    echo ""
    exit 0
fi

