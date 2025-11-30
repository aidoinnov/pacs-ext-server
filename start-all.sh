#!/bin/bash

# PACS Extension Server - 전체 시스템 시작 스크립트
# DB 터널 + 백엔드(Rust) + 프론트엔드(React) 동시 실행

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

# 프로젝트 루트 디렉토리
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$PROJECT_ROOT/pacs-server"
FRONTEND_DIR="$PROJECT_ROOT/auth-dashboard"
SCRIPTS_DIR="$PROJECT_ROOT/scripts"

# PID 파일
DB_TUNNEL_PID_FILE="$PROJECT_ROOT/.db-tunnel.pid"
BACKEND_PID_FILE="$PROJECT_ROOT/.backend.pid"
FRONTEND_PID_FILE="$PROJECT_ROOT/.frontend.pid"

# 로그 파일
DB_TUNNEL_LOG="$PROJECT_ROOT/db-tunnel.log"
BACKEND_LOG="$PROJECT_ROOT/backend.log"
FRONTEND_LOG="$PROJECT_ROOT/frontend.log"

echo "================================================================================"
echo "🚀 PACS Extension Server - 전체 시스템 시작"
echo "================================================================================"

# 0. DB 터널 시작
log_info "DB 터널 시작 중..."

# 기존 터널 확인
if [ -f "$DB_TUNNEL_PID_FILE" ]; then
    DB_TUNNEL_PID=$(cat "$DB_TUNNEL_PID_FILE")
    if ps -p "$DB_TUNNEL_PID" > /dev/null 2>&1; then
        log_warning "기존 DB 터널 종료 중... (PID: $DB_TUNNEL_PID)"
        kill "$DB_TUNNEL_PID" 2>/dev/null || true
        sleep 2
    fi
    rm -f "$DB_TUNNEL_PID_FILE"
fi

# DB 터널 시작
cd "$SCRIPTS_DIR"
nohup ./db-tunnel.sh > "$DB_TUNNEL_LOG" 2>&1 &
DB_TUNNEL_PID=$!
echo "$DB_TUNNEL_PID" > "$DB_TUNNEL_PID_FILE"

# DB 터널 연결 대기
log_info "DB 터널 연결 대기 중..."
for i in {1..10}; do
    if lsof -ti:5456 > /dev/null 2>&1; then
        log_success "DB 터널 연결 완료! (PID: $DB_TUNNEL_PID, Port: 5456)"
        break
    fi
    if [ $i -eq 10 ]; then
        log_error "DB 터널 연결 타임아웃!"
        log_info "로그 확인: tail -f $DB_TUNNEL_LOG"
        exit 1
    fi
    sleep 1
done

cd "$PROJECT_ROOT"

# 1. 기존 프로세스 확인 및 종료
log_info "기존 프로세스 확인 중..."

if [ -f "$BACKEND_PID_FILE" ]; then
    BACKEND_PID=$(cat "$BACKEND_PID_FILE")
    if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
        log_warning "기존 백엔드 프로세스 종료 중... (PID: $BACKEND_PID)"
        kill "$BACKEND_PID" 2>/dev/null || true
        sleep 2
    fi
    rm -f "$BACKEND_PID_FILE"
fi

if [ -f "$FRONTEND_PID_FILE" ]; then
    FRONTEND_PID=$(cat "$FRONTEND_PID_FILE")
    if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
        log_warning "기존 프론트엔드 프로세스 종료 중... (PID: $FRONTEND_PID)"
        kill "$FRONTEND_PID" 2>/dev/null || true
        sleep 2
    fi
    rm -f "$FRONTEND_PID_FILE"
fi

# 포트 확인 및 정리
log_info "포트 8080, 3000 확인 중..."
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
sleep 1

# 2. 백엔드 시작
log_info "백엔드 서버 시작 중..."
cd "$BACKEND_DIR"

if [ ! -f ".env" ]; then
    log_error "백엔드 .env 파일이 없습니다!"
    exit 1
fi

# 백엔드 빌드 및 실행
log_info "Rust 백엔드 빌드 중..."
cargo build --bin pacs_server 2>&1 | tee "$BACKEND_LOG" &
BUILD_PID=$!
wait $BUILD_PID

if [ $? -ne 0 ]; then
    log_error "백엔드 빌드 실패!"
    exit 1
fi

log_success "백엔드 빌드 완료"

# 백엔드 실행
log_info "백엔드 서버 실행 중..."
nohup cargo run --bin pacs_server > "$BACKEND_LOG" 2>&1 &
BACKEND_PID=$!
echo "$BACKEND_PID" > "$BACKEND_PID_FILE"

# 백엔드 시작 대기
log_info "백엔드 서버 시작 대기 중..."
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_success "백엔드 서버 시작 완료! (PID: $BACKEND_PID)"
        break
    fi
    if [ $i -eq 30 ]; then
        log_error "백엔드 서버 시작 타임아웃!"
        log_info "로그 확인: tail -f $BACKEND_LOG"
        exit 1
    fi
    sleep 1
done

# 3. 프론트엔드 시작
log_info "프론트엔드 서버 시작 중..."
cd "$FRONTEND_DIR"

if [ ! -d "node_modules" ]; then
    log_info "npm 패키지 설치 중..."
    npm install
fi

# 프론트엔드 실행
log_info "React 개발 서버 실행 중..."
nohup npm start > "$FRONTEND_LOG" 2>&1 &
FRONTEND_PID=$!
echo "$FRONTEND_PID" > "$FRONTEND_PID_FILE"

# 프론트엔드 시작 대기
log_info "프론트엔드 서버 시작 대기 중..."
for i in {1..60}; do
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        log_success "프론트엔드 서버 시작 완료! (PID: $FRONTEND_PID)"
        break
    fi
    if [ $i -eq 60 ]; then
        log_error "프론트엔드 서버 시작 타임아웃!"
        log_info "로그 확인: tail -f $FRONTEND_LOG"
        exit 1
    fi
    sleep 1
done

# 4. 완료 메시지
echo ""
echo "================================================================================"
echo "✨ 전체 시스템 시작 완료!"
echo "================================================================================"
echo ""
echo "🔌 DB 터널:"
echo "   - PID: $DB_TUNNEL_PID"
echo "   - Local Port: 5456 (extension), 5457 (postgres)"
echo "   - Remote: pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com"
echo "   - 로그: tail -f $DB_TUNNEL_LOG"
echo ""
echo "📦 백엔드 서버:"
echo "   - PID: $BACKEND_PID"
echo "   - URL: http://localhost:8080"
echo "   - Swagger UI: http://localhost:8080/swagger-ui/"
echo "   - Health Check: http://localhost:8080/health"
echo "   - 로그: tail -f $BACKEND_LOG"
echo ""
echo "🎨 프론트엔드 서버:"
echo "   - PID: $FRONTEND_PID"
echo "   - URL: http://localhost:3000"
echo "   - 로그: tail -f $FRONTEND_LOG"
echo ""
echo "🛑 서버 중지: ./stop-all.sh"
echo "🔄 서버 재시작: ./restart-all.sh"
echo "📊 서버 상태: ./status-all.sh"
echo ""
echo "================================================================================"

# 브라우저 자동 열기 (선택사항)
if command -v open > /dev/null 2>&1; then
    log_info "브라우저 열기 중..."
    sleep 2
    open http://localhost:3000
fi

