#!/bin/bash

# PACS Extension Server - 전체 시스템 재시작 스크립트
# 백엔드(Rust) + 프론트엔드(React) 동시 재시작

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

# 프로젝트 루트 디렉토리
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "================================================================================"
echo "🔄 PACS Extension Server - 전체 시스템 재시작"
echo "================================================================================"
echo ""

# 1. 종료
log_info "기존 서버 종료 중..."
"$PROJECT_ROOT/stop-all.sh"

echo ""
log_info "잠시 대기 중..."
sleep 3

# 2. 시작
log_info "서버 시작 중..."
"$PROJECT_ROOT/start-all.sh"

echo ""
log_success "재시작 완료!"

