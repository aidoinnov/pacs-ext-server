#!/bin/bash

# Docker Build Script for PACS Extension Server
# PACS Extension Server용 Docker 빌드 스크립트

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
IMAGE_NAME="pacs-server"
IMAGE_TAG="${IMAGE_NAME}:${ENVIRONMENT}"

log_info "Docker 이미지 빌드를 시작합니다..."
log_info "환경: ${ENVIRONMENT}"
log_info "이미지 태그: ${IMAGE_TAG}"

# 환경별 설정 파일 확인
ENV_FILE="env.${ENVIRONMENT}"
if [ ! -f "$ENV_FILE" ]; then
    log_error "환경 설정 파일을 찾을 수 없습니다: ${ENV_FILE}"
    log_info "사용 가능한 환경: development, production, test"
    exit 1
fi

log_info "환경 설정 파일 사용: ${ENV_FILE}"

# Docker 이미지 빌드
log_info "Docker 이미지를 빌드합니다..."
docker build -t "${IMAGE_TAG}" .

if [ $? -eq 0 ]; then
    log_success "Docker 이미지 빌드가 완료되었습니다: ${IMAGE_TAG}"
    
    # 이미지 정보 출력
    log_info "빌드된 이미지 정보:"
    docker images "${IMAGE_NAME}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
else
    log_error "Docker 이미지 빌드에 실패했습니다."
    exit 1
fi

# 선택적으로 이미지를 latest로 태그
if [ "$ENVIRONMENT" = "production" ]; then
    log_info "프로덕션 이미지를 latest로 태그합니다..."
    docker tag "${IMAGE_TAG}" "${IMAGE_NAME}:latest"
    log_success "latest 태그가 추가되었습니다."
fi

log_success "빌드 프로세스가 완료되었습니다!"
log_info "다음 명령어로 컨테이너를 실행할 수 있습니다:"
log_info "  docker run -p 8080:8080 --env-file ${ENV_FILE} ${IMAGE_TAG}"
log_info "또는 docker-compose를 사용:"
log_info "  docker-compose --env-file ${ENV_FILE} up"
