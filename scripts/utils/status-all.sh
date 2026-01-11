#!/bin/bash

# PACS Extension Server - 전체 시스템 상태 확인 스크립트
# 백엔드(Rust) + 프론트엔드(React) 상태 확인

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 로그 함수
log_info() {
    echo -e "${BLUE}$1${NC}"
}

log_success() {
    echo -e "${GREEN}$1${NC}"
}

log_error() {
    echo -e "${RED}$1${NC}"
}

log_warning() {
    echo -e "${YELLOW}$1${NC}"
}

# 프로젝트 루트 디렉토리
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# PID 파일
BACKEND_PID_FILE="$PROJECT_ROOT/.backend.pid"
FRONTEND_PID_FILE="$PROJECT_ROOT/.frontend.pid"

echo "================================================================================"
echo "📊 PACS Extension Server - 시스템 상태"
echo "================================================================================"
echo ""

# 1. 백엔드 상태
echo "🔧 백엔드 서버 (Rust - Actix-web)"
echo "--------------------------------------------------------------------------------"

BACKEND_RUNNING=false
if [ -f "$BACKEND_PID_FILE" ]; then
    BACKEND_PID=$(cat "$BACKEND_PID_FILE")
    if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
        log_success "✅ 실행 중 (PID: $BACKEND_PID)"
        BACKEND_RUNNING=true
        
        # 메모리 사용량
        MEM=$(ps -o rss= -p "$BACKEND_PID" | awk '{printf "%.1f MB", $1/1024}')
        echo "   메모리: $MEM"
        
        # CPU 사용량
        CPU=$(ps -o %cpu= -p "$BACKEND_PID" | awk '{printf "%.1f%%", $1}')
        echo "   CPU: $CPU"
        
        # 실행 시간
        ELAPSED=$(ps -o etime= -p "$BACKEND_PID" | awk '{print $1}')
        echo "   실행 시간: $ELAPSED"
    else
        log_error "❌ 중지됨 (PID 파일 존재하지만 프로세스 없음)"
    fi
else
    log_error "❌ 중지됨 (PID 파일 없음)"
fi

# 포트 확인
if lsof -ti:8080 > /dev/null 2>&1; then
    PORT_PID=$(lsof -ti:8080)
    log_success "   포트 8080: 사용 중 (PID: $PORT_PID)"
else
    log_warning "   포트 8080: 사용 안 함"
fi

# Health Check
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_success "   Health Check: OK"
    echo "   URL: http://localhost:8080"
    echo "   Swagger UI: http://localhost:8080/swagger-ui/"
else
    log_error "   Health Check: FAIL"
fi

echo ""

# 2. 프론트엔드 상태
echo "🎨 프론트엔드 서버 (React)"
echo "--------------------------------------------------------------------------------"

FRONTEND_RUNNING=false
if [ -f "$FRONTEND_PID_FILE" ]; then
    FRONTEND_PID=$(cat "$FRONTEND_PID_FILE")
    if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
        log_success "✅ 실행 중 (PID: $FRONTEND_PID)"
        FRONTEND_RUNNING=true
        
        # 메모리 사용량
        MEM=$(ps -o rss= -p "$FRONTEND_PID" | awk '{printf "%.1f MB", $1/1024}')
        echo "   메모리: $MEM"
        
        # CPU 사용량
        CPU=$(ps -o %cpu= -p "$FRONTEND_PID" | awk '{printf "%.1f%%", $1}')
        echo "   CPU: $CPU"
        
        # 실행 시간
        ELAPSED=$(ps -o etime= -p "$FRONTEND_PID" | awk '{print $1}')
        echo "   실행 시간: $ELAPSED"
    else
        log_error "❌ 중지됨 (PID 파일 존재하지만 프로세스 없음)"
    fi
else
    log_error "❌ 중지됨 (PID 파일 없음)"
fi

# 포트 확인
if lsof -ti:3000 > /dev/null 2>&1; then
    PORT_PID=$(lsof -ti:3000)
    log_success "   포트 3000: 사용 중 (PID: $PORT_PID)"
else
    log_warning "   포트 3000: 사용 안 함"
fi

# HTTP 확인
if curl -s http://localhost:3000 > /dev/null 2>&1; then
    log_success "   HTTP Check: OK"
    echo "   URL: http://localhost:3000"
else
    log_error "   HTTP Check: FAIL"
fi

echo ""

# 3. 전체 상태 요약
echo "================================================================================"
if [ "$BACKEND_RUNNING" = true ] && [ "$FRONTEND_RUNNING" = true ]; then
    log_success "✅ 전체 시스템 정상 작동 중"
elif [ "$BACKEND_RUNNING" = true ] || [ "$FRONTEND_RUNNING" = true ]; then
    log_warning "⚠️  일부 서버만 실행 중"
else
    log_error "❌ 모든 서버 중지됨"
fi
echo "================================================================================"
echo ""

# 4. 사용 가능한 명령어
echo "📝 사용 가능한 명령어:"
echo "   🚀 시작:    ./start-all.sh"
echo "   🛑 중지:    ./stop-all.sh"
echo "   🔄 재시작:  ./restart-all.sh"
echo "   📊 상태:    ./status-all.sh"
echo ""

# 5. 로그 파일 정보
if [ -f "$PROJECT_ROOT/backend.log" ]; then
    BACKEND_LOG_SIZE=$(du -h "$PROJECT_ROOT/backend.log" | cut -f1)
    echo "📄 백엔드 로그: backend.log ($BACKEND_LOG_SIZE)"
    echo "   tail -f backend.log"
fi

if [ -f "$PROJECT_ROOT/frontend.log" ]; then
    FRONTEND_LOG_SIZE=$(du -h "$PROJECT_ROOT/frontend.log" | cut -f1)
    echo "📄 프론트엔드 로그: frontend.log ($FRONTEND_LOG_SIZE)"
    echo "   tail -f frontend.log"
fi

echo ""

