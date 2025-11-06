#!/bin/bash

# Project User Matrix API 통합 테스트 스크립트
# 이 스크립트는 Matrix API의 모든 기능을 테스트합니다.

set -e  # 에러 발생 시 스크립트 종료

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 서버 URL
BASE_URL="http://localhost:8080"
MATRIX_ENDPOINT="$BASE_URL/api/project-user-matrix"

# 테스트 결과 카운터
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 로그 함수
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# 테스트 헬퍼 함수
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_status="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_info "Running: $test_name"
    
    if eval "$test_command" > /tmp/test_output.json 2>&1; then
        local actual_status=$(jq -r '.status // "unknown"' /tmp/test_output.json 2>/dev/null || echo "unknown")
        
        if [ "$actual_status" = "$expected_status" ] || [ "$expected_status" = "any" ]; then
            log_success "$test_name"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            return 0
        else
            log_error "$test_name - Expected status: $expected_status, Got: $actual_status"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            return 1
        fi
    else
        log_error "$test_name - Command failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# JSON 응답 검증 함수
validate_json_response() {
    local response_file="$1"
    local required_fields="$2"
    
    if ! jq empty "$response_file" 2>/dev/null; then
        log_error "Invalid JSON response"
        return 1
    fi
    
    for field in $required_fields; do
        if ! jq -e ".$field" "$response_file" > /dev/null 2>&1; then
            log_error "Missing required field: $field"
            return 1
        fi
    done
    
    return 0
}

# 서버 상태 확인
check_server() {
    log_info "Checking server status..."
    
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        log_success "Server is running"
        return 0
    else
        log_error "Server is not running or not accessible"
        return 1
    fi
}

# 기본 매트릭스 조회 테스트
test_basic_matrix() {
    log_info "Testing basic matrix retrieval..."
    
    local response_file="/tmp/basic_matrix_response.json"
    
    if curl -s "$MATRIX_ENDPOINT" -o "$response_file"; then
        if validate_json_response "$response_file" "matrix users pagination"; then
            local matrix_count=$(jq '.matrix | length' "$response_file")
            local user_count=$(jq '.users | length' "$response_file")
            
            log_success "Basic matrix test - Matrix: $matrix_count projects, Users: $user_count"
            return 0
        else
            log_error "Basic matrix test - Invalid response structure"
            return 1
        fi
    else
        log_error "Basic matrix test - Request failed"
        return 1
    fi
}

# 페이지네이션 테스트
test_pagination() {
    log_info "Testing pagination..."
    
    # 프로젝트 페이지네이션 테스트
    local response_file="/tmp/pagination_response.json"
    
    if curl -s "$MATRIX_ENDPOINT?project_page=1&project_page_size=3" -o "$response_file"; then
        if validate_json_response "$response_file" "matrix pagination"; then
            local matrix_count=$(jq '.matrix | length' "$response_file")
            local project_page_size=$(jq '.pagination.project_page_size' "$response_file")
            
            if [ "$matrix_count" -le "$project_page_size" ]; then
                log_success "Project pagination test - Returned $matrix_count projects (max: $project_page_size)"
            else
                log_error "Project pagination test - Returned more projects than requested"
                return 1
            fi
        else
            log_error "Project pagination test - Invalid response structure"
            return 1
        fi
    else
        log_error "Project pagination test - Request failed"
        return 1
    fi
    
    # 사용자 페이지네이션 테스트
    if curl -s "$MATRIX_ENDPOINT?user_page=1&user_page_size=5" -o "$response_file"; then
        if validate_json_response "$response_file" "users pagination"; then
            local user_count=$(jq '.users | length' "$response_file")
            local user_page_size=$(jq '.pagination.user_page_size' "$response_file")
            
            if [ "$user_count" -le "$user_page_size" ]; then
                log_success "User pagination test - Returned $user_count users (max: $user_page_size)"
            else
                log_error "User pagination test - Returned more users than requested"
                return 1
            fi
        else
            log_error "User pagination test - Invalid response structure"
            return 1
        fi
    else
        log_error "User pagination test - Request failed"
        return 1
    fi
}

# 상태 필터링 테스트
test_status_filtering() {
    log_info "Testing status filtering..."
    
    local response_file="/tmp/status_filter_response.json"
    
    # IN_PROGRESS 상태 필터링 테스트
    if curl -s "$MATRIX_ENDPOINT?project_status=IN_PROGRESS" -o "$response_file"; then
        if validate_json_response "$response_file" "matrix"; then
            local matrix_count=$(jq '.matrix | length' "$response_file")
            local all_in_progress=$(jq '.matrix | map(.status == "IN_PROGRESS") | all' "$response_file")
            
            if [ "$all_in_progress" = "true" ]; then
                log_success "Status filtering test - All $matrix_count projects are IN_PROGRESS"
            else
                log_error "Status filtering test - Some projects are not IN_PROGRESS"
                return 1
            fi
        else
            log_error "Status filtering test - Invalid response structure"
            return 1
        fi
    else
        log_error "Status filtering test - Request failed"
        return 1
    fi
}

# 복합 필터링 테스트
test_complex_filtering() {
    log_info "Testing complex filtering..."
    
    local response_file="/tmp/complex_filter_response.json"
    
    # 복합 파라미터 테스트
    if curl -s "$MATRIX_ENDPOINT?project_page=1&project_page_size=2&user_page=1&user_page_size=3" -o "$response_file"; then
        if validate_json_response "$response_file" "matrix users pagination"; then
            local matrix_count=$(jq '.matrix | length' "$response_file")
            local user_count=$(jq '.users | length' "$response_file")
            local project_page_size=$(jq '.pagination.project_page_size' "$response_file")
            local user_page_size=$(jq '.pagination.user_page_size' "$response_file")
            
            if [ "$matrix_count" -le "$project_page_size" ] && [ "$user_count" -le "$user_page_size" ]; then
                log_success "Complex filtering test - Matrix: $matrix_count/$project_page_size, Users: $user_count/$user_page_size"
            else
                log_error "Complex filtering test - Pagination limits exceeded"
                return 1
            fi
        else
            log_error "Complex filtering test - Invalid response structure"
            return 1
        fi
    else
        log_error "Complex filtering test - Request failed"
        return 1
    fi
}

# 성능 테스트
test_performance() {
    log_info "Testing performance..."
    
    local start_time=$(date +%s%N)
    local response_file="/tmp/performance_response.json"
    
    if curl -s "$MATRIX_ENDPOINT" -o "$response_file"; then
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))  # 밀리초로 변환
        
        if [ "$duration" -lt 1000 ]; then  # 1초 미만
            log_success "Performance test - Response time: ${duration}ms"
        else
            log_warning "Performance test - Response time: ${duration}ms (slow)"
        fi
    else
        log_error "Performance test - Request failed"
        return 1
    fi
}

# 에러 처리 테스트
test_error_handling() {
    log_info "Testing error handling..."
    
    # 잘못된 파라미터 테스트
    local response_file="/tmp/error_response.json"
    
    if curl -s "$MATRIX_ENDPOINT?project_page=0&project_page_size=0" -o "$response_file"; then
        # 잘못된 파라미터는 기본값으로 처리되어야 함
        if validate_json_response "$response_file" "matrix users pagination"; then
            log_success "Error handling test - Invalid parameters handled gracefully"
        else
            log_error "Error handling test - Invalid response structure"
            return 1
        fi
    else
        log_error "Error handling test - Request failed"
        return 1
    fi
}

# 데이터 무결성 테스트
test_data_integrity() {
    log_info "Testing data integrity..."
    
    local response_file="/tmp/integrity_response.json"
    
    if curl -s "$MATRIX_ENDPOINT" -o "$response_file"; then
        if validate_json_response "$response_file" "matrix users pagination"; then
            # 매트릭스의 각 프로젝트가 모든 사용자와의 관계를 포함하는지 확인
            local matrix_count=$(jq '.matrix | length' "$response_file")
            local user_count=$(jq '.users | length' "$response_file")
            
            # 각 프로젝트의 user_roles 수가 사용자 수와 일치하는지 확인
            local all_projects_valid=$(jq ".matrix | map(.user_roles | length == $user_count) | all" "$response_file")
            
            if [ "$all_projects_valid" = "true" ]; then
                log_success "Data integrity test - All $matrix_count projects have relationships with all $user_count users"
            else
                log_error "Data integrity test - Matrix structure is inconsistent"
                return 1
            fi
        else
            log_error "Data integrity test - Invalid response structure"
            return 1
        fi
    else
        log_error "Data integrity test - Request failed"
        return 1
    fi
}

# 메인 테스트 실행
main() {
    echo "=========================================="
    echo "Project User Matrix API Integration Tests"
    echo "=========================================="
    echo
    
    # 서버 상태 확인
    if ! check_server; then
        echo "Please start the server first: cargo run"
        exit 1
    fi
    
    echo "Starting comprehensive integration tests..."
    echo
    
    # 테스트 실행
    test_basic_matrix
    test_pagination
    test_status_filtering
    test_complex_filtering
    test_performance
    test_error_handling
    test_data_integrity
    
    echo
    echo "=========================================="
    echo "Test Results Summary"
    echo "=========================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo -e "${GREEN}All tests passed! 🎉${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed! ❌${NC}"
        exit 1
    fi
}

# 스크립트 실행
main "$@"
