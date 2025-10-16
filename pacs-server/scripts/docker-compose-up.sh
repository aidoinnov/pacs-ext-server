#!/bin/bash

# Docker Compose Up Script for PACS Extension Server
# PACS Extension Server용 Docker Compose 실행 스크립트

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
ENVIRONMENT=${1:-development}
ENV_FILE="env.${ENVIRONMENT}"
COMPOSE_FILE="docker-compose.yaml"

log_info "Docker Compose를 시작합니다..."
log_info "환경: ${ENVIRONMENT}"
log_info "환경 파일: ${ENV_FILE}"

# 환경별 설정 파일 확인
if [ ! -f "$ENV_FILE" ]; then
    log_error "환경 설정 파일을 찾을 수 없습니다: ${ENV_FILE}"
    log_info "사용 가능한 환경: development, production, test"
    exit 1
fi

# Docker Compose 파일 확인
if [ ! -f "$COMPOSE_FILE" ]; then
    log_error "Docker Compose 파일을 찾을 수 없습니다: ${COMPOSE_FILE}"
    exit 1
fi

# 기존 컨테이너 정리 (선택사항)
if [ "$2" = "--clean" ]; then
    log_warning "기존 컨테이너와 볼륨을 정리합니다..."
    docker-compose --env-file "$ENV_FILE" down -v
    docker system prune -f
fi

# Docker Compose 실행
log_info "Docker Compose를 실행합니다..."
docker-compose --env-file "$ENV_FILE" up -d

if [ $? -eq 0 ]; then
    log_success "Docker Compose 실행이 완료되었습니다!"
    
    # 서비스 상태 확인
    log_info "서비스 상태를 확인합니다..."
    docker-compose --env-file "$ENV_FILE" ps
    
    # 헬스체크 대기
    log_info "서비스가 준비될 때까지 대기합니다..."
    sleep 10
    
    # 각 서비스의 헬스체크
    log_info "서비스 헬스체크를 수행합니다..."
    
    # PostgreSQL 헬스체크
    if docker-compose --env-file "$ENV_FILE" exec -T postgres pg_isready -U admin -d pacs_db > /dev/null 2>&1; then
        log_success "PostgreSQL이 준비되었습니다."
    else
        log_warning "PostgreSQL이 아직 준비되지 않았습니다."
    fi
    
    # Redis 헬스체크
    if docker-compose --env-file "$ENV_FILE" exec -T redis redis-cli ping > /dev/null 2>&1; then
        log_success "Redis가 준비되었습니다."
    else
        log_warning "Redis가 아직 준비되지 않았습니다."
    fi
    
    # MinIO 헬스체크
    if curl -f http://localhost:9000/minio/health/live > /dev/null 2>&1; then
        log_success "MinIO가 준비되었습니다."
    else
        log_warning "MinIO가 아직 준비되지 않았습니다."
    fi
    
    # PACS Server 헬스체크
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        log_success "PACS Server가 준비되었습니다."
    else
        log_warning "PACS Server가 아직 준비되지 않았습니다."
    fi
    
    # 서비스 접속 정보 출력
    log_info "서비스 접속 정보:"
    echo "  🌐 PACS Server:     http://localhost:8080"
    echo "  📖 Swagger UI:      http://localhost:8080/swagger-ui/"
    echo "  ❤️  Health Check:    http://localhost:8080/health"
    echo "  🗄️  PostgreSQL:      localhost:5432"
    echo "  🔴 Redis:           localhost:6379"
    echo "  📦 MinIO:           http://localhost:9000"
    echo "  🖥️  MinIO Console:   http://localhost:9001"
    echo "  🌐 Nginx:           http://localhost:80"
    
    log_info "로그를 확인하려면: docker-compose --env-file ${ENV_FILE} logs -f"
    log_info "서비스를 중지하려면: docker-compose --env-file ${ENV_FILE} down"
    
else
    log_error "Docker Compose 실행에 실패했습니다."
    exit 1
fi
