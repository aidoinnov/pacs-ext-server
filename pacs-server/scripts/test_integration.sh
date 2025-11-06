#!/bin/bash

# PACS Server Integration Test Script
# Global Roles with Permissions API 테스트

set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 설정
SERVER_URL="http://localhost:8080"
API_BASE="$SERVER_URL/api"
TEST_TOKEN=""

# 로그 함수
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 서버 상태 확인
check_server() {
    log_info "서버 상태 확인 중..."
    
    if curl -s --connect-timeout 5 "$SERVER_URL/health" > /dev/null 2>&1; then
        log_success "서버가 실행 중입니다"
        return 0
    else
        log_error "서버가 실행되지 않았거나 접근할 수 없습니다"
        log_info "서버를 시작하려면: cargo run"
        return 1
    fi
}

# 인증 토큰 획득 (간단한 테스트용)
get_auth_token() {
    log_info "인증 토큰 획득 중..."
    
    # 실제 환경에서는 Keycloak에서 토큰을 받아야 하지만,
    # 테스트용으로는 임시 토큰을 사용하거나 인증을 우회합니다
    TEST_TOKEN="test-token-12345"
    log_warning "테스트용 토큰을 사용합니다: $TEST_TOKEN"
}

# HTTP 요청 헬퍼 함수
make_request() {
    local method="$1"
    local url="$2"
    local data="$3"
    local expected_status="$4"
    
    log_info "$method $url"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $TEST_TOKEN" \
            -d "$data" \
            "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Authorization: Bearer $TEST_TOKEN" \
            "$url")
    fi
    
    # HTTP 상태 코드와 응답 본문 분리
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    log_info "응답 상태: $http_code"
    
    if [ "$http_code" = "$expected_status" ]; then
        log_success "예상 상태 코드와 일치: $expected_status"
        echo "$body"
        return 0
    else
        log_error "예상 상태 코드와 불일치: 예상 $expected_status, 실제 $http_code"
        echo "$body"
        return 1
    fi
}

# JSON 응답 검증
validate_json() {
    local json="$1"
    local field="$2"
    
    if echo "$json" | jq -e ".$field" > /dev/null 2>&1; then
        log_success "JSON 필드 '$field' 존재 확인"
        return 0
    else
        log_error "JSON 필드 '$field' 누락"
        return 1
    fi
}

# 테스트 1: 기본 페이지네이션 테스트
test_basic_pagination() {
    log_info "=== 테스트 1: 기본 페이지네이션 ==="
    
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions" "" "200")
    
    if [ $? -eq 0 ]; then
        # 필수 필드 검증
        validate_json "$response" "roles"
        validate_json "$response" "total_count"
        validate_json "$response" "page"
        validate_json "$response" "page_size"
        validate_json "$response" "total_pages"
        
        # 기본값 검증
        local page=$(echo "$response" | jq -r '.page')
        local page_size=$(echo "$response" | jq -r '.page_size')
        
        if [ "$page" = "1" ] && [ "$page_size" = "20" ]; then
            log_success "기본 페이지네이션 값 확인"
        else
            log_error "기본 페이지네이션 값 불일치: page=$page, page_size=$page_size"
        fi
        
        log_success "테스트 1 통과"
    else
        log_error "테스트 1 실패"
        return 1
    fi
}

# 테스트 2: 커스텀 페이지네이션 테스트
test_custom_pagination() {
    log_info "=== 테스트 2: 커스텀 페이지네이션 ==="
    
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions?page=2&page_size=10" "" "200")
    
    if [ $? -eq 0 ]; then
        local page=$(echo "$response" | jq -r '.page')
        local page_size=$(echo "$response" | jq -r '.page_size')
        
        if [ "$page" = "2" ] && [ "$page_size" = "10" ]; then
            log_success "커스텀 페이지네이션 값 확인"
        else
            log_error "커스텀 페이지네이션 값 불일치: page=$page, page_size=$page_size"
        fi
        
        log_success "테스트 2 통과"
    else
        log_error "테스트 2 실패"
        return 1
    fi
}

# 테스트 3: 잘못된 파라미터 테스트
test_invalid_params() {
    log_info "=== 테스트 3: 잘못된 파라미터 ==="
    
    # 음수 페이지
    local response1=$(make_request "GET" "$API_BASE/roles/global/with-permissions?page=-1" "" "200")
    if [ $? -eq 0 ]; then
        local page=$(echo "$response1" | jq -r '.page')
        if [ "$page" = "1" ]; then
            log_success "음수 페이지는 1로 처리됨"
        else
            log_error "음수 페이지 처리 실패: page=$page"
        fi
    fi
    
    # 너무 큰 페이지 크기
    local response2=$(make_request "GET" "$API_BASE/roles/global/with-permissions?page_size=200" "" "200")
    if [ $? -eq 0 ]; then
        local page_size=$(echo "$response2" | jq -r '.page_size')
        if [ "$page_size" -le 100 ]; then
            log_success "큰 페이지 크기는 100으로 제한됨"
        else
            log_error "페이지 크기 제한 실패: page_size=$page_size"
        fi
    fi
    
    log_success "테스트 3 통과"
}

# 테스트 4: 응답 구조 검증
test_response_structure() {
    log_info "=== 테스트 4: 응답 구조 검증 ==="
    
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions" "" "200")
    
    if [ $? -eq 0 ]; then
        # roles 배열 검증
        local roles_count=$(echo "$response" | jq '.roles | length')
        log_info "역할 개수: $roles_count"
        
        if [ "$roles_count" -gt 0 ]; then
            # 첫 번째 역할의 구조 검증
            local first_role=$(echo "$response" | jq '.roles[0]')
            
            validate_json "$first_role" "id"
            validate_json "$first_role" "name"
            validate_json "$first_role" "scope"
            validate_json "$first_role" "permissions"
            
            # permissions 배열 검증
            local permissions_count=$(echo "$first_role" | jq '.permissions | length')
            log_info "첫 번째 역할의 권한 개수: $permissions_count"
            
            if [ "$permissions_count" -gt 0 ]; then
                local first_permission=$(echo "$first_role" | jq '.permissions[0]')
                validate_json "$first_permission" "id"
                validate_json "$first_permission" "resource_type"
                validate_json "$first_permission" "action"
                log_success "권한 구조 검증 완료"
            else
                log_warning "권한이 없는 역할입니다"
            fi
        else
            log_warning "역할이 없습니다"
        fi
        
        log_success "테스트 4 통과"
    else
        log_error "테스트 4 실패"
        return 1
    fi
}

# 테스트 5: 빈 결과 테스트
test_empty_result() {
    log_info "=== 테스트 5: 빈 결과 테스트 ==="
    
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions?page=999&page_size=10" "" "200")
    
    if [ $? -eq 0 ]; then
        local roles_count=$(echo "$response" | jq '.roles | length')
        local total_count=$(echo "$response" | jq '.total_count')
        
        if [ "$roles_count" = "0" ] && [ "$total_count" = "0" ]; then
            log_success "빈 결과 처리 확인"
        else
            log_error "빈 결과 처리 실패: roles=$roles_count, total=$total_count"
        fi
        
        log_success "테스트 5 통과"
    else
        log_error "테스트 5 실패"
        return 1
    fi
}

# 메인 테스트 실행
run_tests() {
    log_info "PACS Server 통합 테스트 시작"
    log_info "서버 URL: $SERVER_URL"
    echo
    
    # 서버 상태 확인
    if ! check_server; then
        exit 1
    fi
    
    # 인증 토큰 획득
    get_auth_token
    echo
    
    # 테스트 실행
    local failed_tests=0
    
    test_basic_pagination || ((failed_tests++))
    echo
    
    test_custom_pagination || ((failed_tests++))
    echo
    
    test_invalid_params || ((failed_tests++))
    echo
    
    test_response_structure || ((failed_tests++))
    echo
    
    test_empty_result || ((failed_tests++))
    echo
    
    # 결과 요약
    log_info "=== 테스트 결과 요약 ==="
    if [ $failed_tests -eq 0 ]; then
        log_success "모든 테스트 통과! 🎉"
        exit 0
    else
        log_error "$failed_tests 개 테스트 실패"
        exit 1
    fi
}

# 도움말
show_help() {
    echo "PACS Server 통합 테스트 스크립트"
    echo
    echo "사용법:"
    echo "  $0 [옵션]"
    echo
    echo "옵션:"
    echo "  -h, --help     이 도움말 표시"
    echo "  -s, --server   서버 URL (기본값: http://localhost:8080)"
    echo "  -t, --token    인증 토큰 (기본값: 테스트 토큰 사용)"
    echo
    echo "예시:"
    echo "  $0                                    # 기본 설정으로 테스트"
    echo "  $0 -s http://localhost:3000          # 다른 포트로 테스트"
    echo "  $0 -t your-jwt-token                 # 실제 토큰으로 테스트"
}

# 명령행 인수 처리
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -s|--server)
            SERVER_URL="$2"
            API_BASE="$SERVER_URL/api"
            shift 2
            ;;
        -t|--token)
            TEST_TOKEN="$2"
            shift 2
            ;;
        *)
            log_error "알 수 없는 옵션: $1"
            show_help
            exit 1
            ;;
    esac
done

# jq 설치 확인
if ! command -v jq &> /dev/null; then
    log_error "jq가 설치되지 않았습니다. 설치 후 다시 시도하세요:"
    log_info "  Ubuntu/Debian: sudo apt-get install jq"
    log_info "  CentOS/RHEL: sudo yum install jq"
    log_info "  macOS: brew install jq"
    exit 1
fi

# curl 설치 확인
if ! command -v curl &> /dev/null; then
    log_error "curl이 설치되지 않았습니다."
    exit 1
fi

# 테스트 실행
run_tests
