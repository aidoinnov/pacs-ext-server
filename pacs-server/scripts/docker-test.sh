#!/bin/bash

# Docker Test Script for PACS Extension Server
# PACS Extension Server용 Docker 테스트 스크립트

set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 로그 함수들
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

# 환경 변수 설정
ENVIRONMENT=${1:-test}
ENV_FILE="env.${ENVIRONMENT}"

log_info "Docker 컨테이너 테스트를 시작합니다..."
log_info "환경: ${ENVIRONMENT}"
log_info "환경 파일: ${ENV_FILE}"

# 환경별 설정 파일 확인
if [ ! -f "$ENV_FILE" ]; then
    log_error "환경 설정 파일을 찾을 수 없습니다: ${ENV_FILE}"
    log_info "사용 가능한 환경: development, production, test"
    exit 1
fi

# 테스트 함수들
test_health_check() {
    local service_name=$1
    local url=$2
    local max_attempts=${3:-30}
    local attempt=1
    
    log_info "${service_name} 헬스체크를 수행합니다... (${url})"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s "$url" > /dev/null 2>&1; then
            log_success "${service_name}이(가) 정상적으로 응답합니다."
            return 0
        fi
        
        log_info "시도 ${attempt}/${max_attempts}: ${service_name} 대기 중..."
        sleep 2
        ((attempt++))
    done
    
    log_error "${service_name}이(가) 응답하지 않습니다."
    return 1
}

test_api_endpoints() {
    local base_url=$1
    
    log_info "API 엔드포인트 테스트를 수행합니다..."
    
    # 헬스체크 엔드포인트
    if curl -f -s "${base_url}/health" > /dev/null 2>&1; then
        log_success "헬스체크 엔드포인트가 정상입니다."
    else
        log_error "헬스체크 엔드포인트에 문제가 있습니다."
        return 1
    fi
    
    # Swagger UI 엔드포인트
    if curl -f -s "${base_url}/swagger-ui/" > /dev/null 2>&1; then
        log_success "Swagger UI 엔드포인트가 정상입니다."
    else
        log_warning "Swagger UI 엔드포인트에 문제가 있습니다."
    fi
    
    # OpenAPI 스펙 엔드포인트
    if curl -f -s "${base_url}/api-docs/openapi.json" > /dev/null 2>&1; then
        log_success "OpenAPI 스펙 엔드포인트가 정상입니다."
    else
        log_warning "OpenAPI 스펙 엔드포인트에 문제가 있습니다."
    fi
    
    return 0
}

test_database_connection() {
    log_info "데이터베이스 연결을 테스트합니다..."
    
    # PostgreSQL 연결 테스트
    if docker-compose --env-file "$ENV_FILE" exec -T postgres pg_isready -U admin -d pacs_db > /dev/null 2>&1; then
        log_success "PostgreSQL 연결이 정상입니다."
    else
        log_error "PostgreSQL 연결에 문제가 있습니다."
        return 1
    fi
    
    return 0
}

test_redis_connection() {
    log_info "Redis 연결을 테스트합니다..."
    
    # Redis 연결 테스트
    if docker-compose --env-file "$ENV_FILE" exec -T redis redis-cli ping > /dev/null 2>&1; then
        log_success "Redis 연결이 정상입니다."
    else
        log_error "Redis 연결에 문제가 있습니다."
        return 1
    fi
    
    return 0
}

test_object_storage() {
    log_info "객체 저장소 연결을 테스트합니다..."
    
    # MinIO 연결 테스트
    if curl -f -s http://localhost:9000/minio/health/live > /dev/null 2>&1; then
        log_success "MinIO 연결이 정상입니다."
    else
        log_error "MinIO 연결에 문제가 있습니다."
        return 1
    fi
    
    return 0
}

# 메인 테스트 실행
main() {
    local test_passed=0
    local test_failed=0
    
    log_info "=== Docker 컨테이너 테스트 시작 ==="
    
    # 1. 데이터베이스 연결 테스트
    if test_database_connection; then
        ((test_passed++))
    else
        ((test_failed++))
    fi
    
    # 2. Redis 연결 테스트
    if test_redis_connection; then
        ((test_passed++))
    else
        ((test_failed++))
    fi
    
    # 3. 객체 저장소 연결 테스트
    if test_object_storage; then
        ((test_passed++))
    else
        ((test_failed++))
    fi
    
    # 4. PACS Server 헬스체크
    if test_health_check "PACS Server" "http://localhost:8080/health" 30; then
        ((test_passed++))
    else
        ((test_failed++))
    fi
    
    # 5. API 엔드포인트 테스트
    if test_api_endpoints "http://localhost:8080"; then
        ((test_passed++))
    else
        ((test_failed++))
    fi
    
    # 6. Nginx 프록시 테스트 (선택사항)
    if curl -f -s http://localhost:80/health > /dev/null 2>&1; then
        log_success "Nginx 프록시가 정상입니다."
        ((test_passed++))
    else
        log_warning "Nginx 프록시에 문제가 있습니다."
        ((test_failed++))
    fi
    
    # 테스트 결과 요약
    log_info "=== 테스트 결과 요약 ==="
    log_info "통과: ${test_passed}"
    log_info "실패: ${test_failed}"
    
    if [ $test_failed -eq 0 ]; then
        log_success "모든 테스트가 통과했습니다! 🎉"
        exit 0
    else
        log_error "일부 테스트가 실패했습니다. 로그를 확인해주세요."
        exit 1
    fi
}

# 도움말 표시
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "사용법: $0 [환경]"
    echo ""
    echo "환경:"
    echo "  development  개발 환경 (기본값)"
    echo "  production   프로덕션 환경"
    echo "  test         테스트 환경"
    echo ""
    echo "예시:"
    echo "  $0 test              # 테스트 환경에서 테스트"
    echo "  $0 development       # 개발 환경에서 테스트"
    exit 0
fi

# 메인 함수 실행
main
