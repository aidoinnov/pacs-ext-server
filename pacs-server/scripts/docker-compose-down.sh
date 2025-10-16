#!/bin/bash

# Docker Compose Down Script for PACS Extension Server
# PACS Extension Server용 Docker Compose 중지 스크립트

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

log_info "Docker Compose를 중지합니다..."
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

# 실행 중인 서비스 확인
log_info "실행 중인 서비스를 확인합니다..."
docker-compose --env-file "$ENV_FILE" ps

# 서비스 중지 옵션 확인
CLEAN_VOLUMES=false
CLEAN_IMAGES=false

case "$2" in
    "--volumes"|"-v")
        CLEAN_VOLUMES=true
        log_warning "볼륨도 함께 제거합니다."
        ;;
    "--all"|"-a")
        CLEAN_VOLUMES=true
        CLEAN_IMAGES=true
        log_warning "볼륨과 이미지도 함께 제거합니다."
        ;;
    "--help"|"-h")
        echo "사용법: $0 [환경] [옵션]"
        echo ""
        echo "환경:"
        echo "  development  개발 환경 (기본값)"
        echo "  production   프로덕션 환경"
        echo "  test         테스트 환경"
        echo ""
        echo "옵션:"
        echo "  --volumes, -v    볼륨도 함께 제거"
        echo "  --all, -a        볼륨과 이미지도 함께 제거"
        echo "  --help, -h       도움말 표시"
        exit 0
        ;;
esac

# Docker Compose 중지
log_info "Docker Compose를 중지합니다..."

if [ "$CLEAN_VOLUMES" = true ]; then
    docker-compose --env-file "$ENV_FILE" down -v
    log_success "서비스와 볼륨이 중지/제거되었습니다."
else
    docker-compose --env-file "$ENV_FILE" down
    log_success "서비스가 중지되었습니다."
fi

# 이미지 제거 (선택사항)
if [ "$CLEAN_IMAGES" = true ]; then
    log_warning "Docker 이미지를 제거합니다..."
    docker-compose --env-file "$ENV_FILE" down --rmi all
    log_success "이미지가 제거되었습니다."
fi

# 정리 작업
log_info "Docker 시스템을 정리합니다..."
docker system prune -f

# 중지 확인
log_info "중지된 서비스 확인:"
docker-compose --env-file "$ENV_FILE" ps

log_success "Docker Compose 중지가 완료되었습니다!"

# 다음 단계 안내
if [ "$CLEAN_VOLUMES" = false ]; then
    log_info "데이터는 보존되었습니다. 다시 시작하려면:"
    log_info "  ./scripts/docker-compose-up.sh ${ENVIRONMENT}"
else
    log_info "모든 데이터가 제거되었습니다. 처음부터 시작하려면:"
    log_info "  ./scripts/docker-compose-up.sh ${ENVIRONMENT}"
fi
