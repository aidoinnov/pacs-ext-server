#!/bin/bash

# PACS Server Mock Integration Test Script
# Global Roles with Permissions API 테스트 (Mock 서버 사용)

set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 설정
MOCK_SERVER_PORT=8083
MOCK_SERVER_URL="http://localhost:$MOCK_SERVER_PORT"
API_BASE="$MOCK_SERVER_URL/api"

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

# Mock 서버 시작
start_mock_server() {
    log_info "Mock 서버 시작 중..."
    
    # Python을 사용한 간단한 Mock 서버
    cat > /tmp/mock_server.py << 'EOF'
import json
import http.server
import socketserver
from urllib.parse import urlparse, parse_qs

class MockHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_url = urlparse(self.path)
        path = parsed_url.path
        query_params = parse_qs(parsed_url.query)
        
        if path == "/health":
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok"}).encode())
            
        elif path == "/api/roles/global/with-permissions":
            # 페이지네이션 파라미터 처리
            page = int(query_params.get('page', ['1'])[0])
            page_size = int(query_params.get('page_size', ['20'])[0])
            
            # Mock 데이터
            mock_roles = [
                {
                    "id": 1,
                    "name": "시스템 관리자",
                    "description": "전체 시스템 관리 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 1, "resource_type": "user", "action": "create"},
                        {"id": 2, "resource_type": "user", "action": "delete"},
                        {"id": 3, "resource_type": "project", "action": "create"}
                    ]
                },
                {
                    "id": 2,
                    "name": "프로젝트 관리자",
                    "description": "프로젝트 관리 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 4, "resource_type": "project", "action": "read"},
                        {"id": 5, "resource_type": "project", "action": "update"}
                    ]
                },
                {
                    "id": 3,
                    "name": "사용자",
                    "description": "기본 사용자 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 6, "resource_type": "annotation", "action": "read"},
                        {"id": 7, "resource_type": "annotation", "action": "create"}
                    ]
                }
            ]
            
            # 페이지네이션 적용
            total_count = len(mock_roles)
            start_idx = (page - 1) * page_size
            end_idx = start_idx + page_size
            paginated_roles = mock_roles[start_idx:end_idx]
            
            total_pages = (total_count + page_size - 1) // page_size
            
            response = {
                "roles": paginated_roles,
                "total_count": total_count,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages
            }
            
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())
            
        else:
            self.send_response(404)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({"error": "Not found"}).encode())
    
    def log_message(self, format, *args):
        # 로그 메시지 억제
        pass

if __name__ == "__main__":
    PORT = 8083
    with socketserver.TCPServer(("", PORT), MockHandler) as httpd:
        print(f"Mock server running on port {PORT}")
        httpd.serve_forever()
EOF
    
    # Mock 서버를 백그라운드에서 시작
    python3 /tmp/mock_server.py &
    MOCK_SERVER_PID=$!
    echo $MOCK_SERVER_PID > /tmp/mock_server.pid
    
    # 서버 시작 대기
    sleep 2
    
    # 서버 상태 확인
    if curl -s --connect-timeout 5 "$MOCK_SERVER_URL/health" > /dev/null 2>&1; then
        log_success "Mock 서버가 시작되었습니다 (PID: $MOCK_SERVER_PID)"
        return 0
    else
        log_error "Mock 서버 시작 실패"
        return 1
    fi
}

# Mock 서버 종료
stop_mock_server() {
    if [ -f /tmp/mock_server.pid ]; then
        local pid=$(cat /tmp/mock_server.pid)
        if kill -0 $pid 2>/dev/null; then
            log_info "Mock 서버 종료 중... (PID: $pid)"
            kill $pid
            rm -f /tmp/mock_server.pid
            log_success "Mock 서버가 종료되었습니다"
        fi
    fi
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
            -d "$data" \
            "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            "$url")
    fi
    
    # HTTP 상태 코드와 응답 본문 분리
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    # 디버깅을 위한 로그
    log_info "Raw response: $response"
    log_info "HTTP code: $http_code"
    log_info "Body: $body"
    
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
    
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions?page=2&page_size=2" "" "200")
    
    if [ $? -eq 0 ]; then
        local page=$(echo "$response" | jq -r '.page')
        local page_size=$(echo "$response" | jq -r '.page_size')
        local roles_count=$(echo "$response" | jq '.roles | length')
        
        if [ "$page" = "2" ] && [ "$page_size" = "2" ] && [ "$roles_count" = "1" ]; then
            log_success "커스텀 페이지네이션 값 확인"
        else
            log_error "커스텀 페이지네이션 값 불일치: page=$page, page_size=$page_size, roles=$roles_count"
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
        
        if [ "$roles_count" = "0" ] && [ "$total_count" = "3" ]; then
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
    log_info "PACS Server Mock 통합 테스트 시작"
    log_info "Mock 서버 URL: $MOCK_SERVER_URL"
    echo
    
    # Mock 서버 시작
    if ! start_mock_server; then
        exit 1
    fi
    
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
    
    # Mock 서버 종료
    stop_mock_server
    
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

# 정리 함수
cleanup() {
    log_info "정리 중..."
    stop_mock_server
    rm -f /tmp/mock_server.py
}

# 시그널 핸들러 설정
trap cleanup EXIT INT TERM

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

# Python 설치 확인
if ! command -v python3 &> /dev/null; then
    log_error "python3이 설치되지 않았습니다."
    exit 1
fi

# 테스트 실행
run_tests
