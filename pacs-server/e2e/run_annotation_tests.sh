#!/bin/bash

# 어노테이션 E2E 테스트 실행 스크립트

set -e  # 에러 발생 시 중단

echo "🚀 어노테이션 E2E 테스트 실행 시작..."
echo ""

# 서버 상태 확인
echo "🔍 서버 상태 확인 중..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "❌ 서버가 실행 중이지 않습니다. 먼저 서버를 시작해주세요."
    echo "   실행 방법: cd pacs-server && cargo run --bin pacs_server"
    exit 1
fi
echo "✅ 서버 실행 중"
echo ""

# 테스트 디렉토리로 이동
cd "$(dirname "$0")"

# 테스트 카운터
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 테스트 실행 함수
run_test() {
    local test_file=$1
    local test_name=$2
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 테스트: $test_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if python3 "$test_file"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "✅ $test_name 통과"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "❌ $test_name 실패"
    fi
    
    echo ""
}

# 각 테스트 실행
run_test "test_annotation_api_debug.py" "기본 API 디버그 테스트"
run_test "test_annotation_permission_filtering.py" "권한 기반 필터링 테스트"
run_test "test_annotation_level_filtering.py" "레벨 필터링 테스트"
run_test "test_annotation_version_conflict.py" "버전 충돌 테스트"
run_test "test_annotation_head_request.py" "HEAD 요청 테스트"
run_test "test_annotation_snapshot_e2e.py" "스냅샷 업로드 테스트"

# 결과 요약
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 테스트 결과 요약"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "총 테스트: $TOTAL_TESTS"
echo "통과: $PASSED_TESTS"
echo "실패: $FAILED_TESTS"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo "🎉 모든 테스트 통과!"
    exit 0
else
    echo "❌ $FAILED_TESTS 개의 테스트 실패"
    exit 1
fi

