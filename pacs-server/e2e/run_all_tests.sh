#!/bin/bash
# 전체 E2E 테스트 실행 스크립트

set -e  # 에러 발생 시 중단

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 전체 E2E 테스트 실행"
echo "=========================================="
echo ""

# 색상 정의
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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
set +e  # 에러 발생해도 계속 진행

echo ""
echo "=========================================="
echo "📦 Annotation 테스트"
echo "=========================================="
run_test "test_annotation_head_request.py" "HEAD 요청 및 캐시 검증"
run_test "test_annotation_level_filtering.py" "레벨 필터링 (Study/Series/Instance)"
run_test "test_annotation_version_conflict.py" "버전 충돌 (Optimistic Locking)"
run_test "test_annotation_permission_filtering.py" "권한 기반 필터링"
run_test "test_annotation_snapshot_e2e.py" "스냅샷 업로드"

echo ""
echo "=========================================="
echo "🏥 DICOM Gateway 테스트"
echo "=========================================="
run_test "test_dicom_gateway_study_series_e2e.py" "DICOM Gateway Study/Series"
run_test "test_dicom_gateway_report_status_filter_e2e.py" "DICOM Gateway Report Status Filter"
run_test "test_qido_enhanced_e2e.py" "QIDO Enhanced"

echo ""
echo "=========================================="
echo "📊 Series 테스트"
echo "=========================================="
run_test "test_series_note_e2e.py" "Series Note"
run_test "test_series_report_e2e.py" "Series Report"
run_test "test_series_resource_level_e2e.py" "Series Resource Level"
run_test "test_series_uid_api_e2e.py" "Series UID API"
run_test "test_series_user_report_api_e2e.py" "Series User Report API"

echo ""
echo "=========================================="
echo "🖥️ Viewer 테스트"
echo "=========================================="
run_test "test_viewer_api_e2e.py" "Viewer API"
run_test "test_view_selection_e2e.py" "View Selection"
run_test "test_study_list_view_e2e.py" "Study List View"

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
    echo -e "${GREEN}🎉 모든 테스트 통과!${NC}"
    echo ""
    exit 0
fi

