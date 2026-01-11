#!/bin/bash

# Viewer API 테스트 실행 스크립트
# 
# 사용법:
#   ./scripts/test_viewer_api.sh [integration|performance|all]

set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 로그 함수
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 테스트 타입 (기본값: integration)
TEST_TYPE="${1:-integration}"

log_info "Viewer API 테스트 시작"
log_info "테스트 타입: $TEST_TYPE"
echo ""

# 환경 변수 확인
if [ -z "$DATABASE_URL" ]; then
    log_warning "DATABASE_URL이 설정되지 않았습니다. 기본값 사용"
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/pacs_test"
fi

log_info "데이터베이스: $DATABASE_URL"
echo ""

# 통합 테스트 실행
run_integration_tests() {
    log_info "=== 통합 테스트 실행 ==="
    echo ""
    
    cargo test --test viewer_controller_integration_test -- --nocapture
    
    if [ $? -eq 0 ]; then
        log_success "통합 테스트 통과"
    else
        log_error "통합 테스트 실패"
        return 1
    fi
}

# 성능 테스트 실행
run_performance_tests() {
    log_info "=== 성능 테스트 실행 ==="
    echo ""
    
    log_warning "성능 테스트는 실제 QIDO 서버가 필요합니다"
    log_warning "테스트 실행 전 dcm4chee 서버가 실행 중인지 확인하세요"
    echo ""
    
    # 성능 테스트는 --ignored 플래그로 실행
    cargo test --test viewer_controller_performance_test -- --ignored --nocapture
    
    if [ $? -eq 0 ]; then
        log_success "성능 테스트 통과"
    else
        log_error "성능 테스트 실패"
        return 1
    fi
}

# 모든 테스트 실행
run_all_tests() {
    log_info "=== 모든 테스트 실행 ==="
    echo ""
    
    run_integration_tests
    INTEGRATION_RESULT=$?
    
    echo ""
    echo "================================"
    echo ""
    
    run_performance_tests
    PERFORMANCE_RESULT=$?
    
    echo ""
    echo "================================"
    echo ""
    
    if [ $INTEGRATION_RESULT -eq 0 ] && [ $PERFORMANCE_RESULT -eq 0 ]; then
        log_success "모든 테스트 통과!"
        return 0
    else
        log_error "일부 테스트 실패"
        return 1
    fi
}

# 테스트 타입에 따라 실행
case "$TEST_TYPE" in
    integration)
        run_integration_tests
        ;;
    performance)
        run_performance_tests
        ;;
    all)
        run_all_tests
        ;;
    *)
        log_error "알 수 없는 테스트 타입: $TEST_TYPE"
        echo "사용법: $0 [integration|performance|all]"
        exit 1
        ;;
esac

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    log_success "테스트 완료!"
else
    log_error "테스트 실패!"
fi

exit $EXIT_CODE

