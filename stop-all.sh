#!/bin/bash

# PACS Extension Server - 전체 시스템 종료 스크립트
# 백엔드(Rust) + 프론트엔드(React) 동시 종료

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

# PID 파일
BACKEND_PID_FILE="$PROJECT_ROOT/.backend.pid"
FRONTEND_PID_FILE="$PROJECT_ROOT/.frontend.pid"

echo "================================================================================"
echo "🛑 PACS Extension Server - 전체 시스템 종료"
echo "================================================================================"

STOPPED=0

# 1. 백엔드 종료
if [ -f "$BACKEND_PID_FILE" ]; then
    BACKEND_PID=$(cat "$BACKEND_PID_FILE")
    if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
        log_info "백엔드 서버 종료 중... (PID: $BACKEND_PID)"
        kill "$BACKEND_PID" 2>/dev/null || true
        
        # 종료 대기
        for i in {1..10}; do
            if ! ps -p "$BACKEND_PID" > /dev/null 2>&1; then
                log_success "백엔드 서버 종료 완료"
                STOPPED=$((STOPPED + 1))
                break
            fi
            sleep 1
        done
        
        # 강제 종료
        if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
            log_warning "백엔드 서버 강제 종료 중..."
            kill -9 "$BACKEND_PID" 2>/dev/null || true
            log_success "백엔드 서버 강제 종료 완료"
            STOPPED=$((STOPPED + 1))
        fi
    else
        log_warning "백엔드 서버가 실행 중이 아닙니다 (PID: $BACKEND_PID)"
    fi
    rm -f "$BACKEND_PID_FILE"
else
    log_warning "백엔드 PID 파일이 없습니다"
fi

# 2. 프론트엔드 종료
if [ -f "$FRONTEND_PID_FILE" ]; then
    FRONTEND_PID=$(cat "$FRONTEND_PID_FILE")
    if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
        log_info "프론트엔드 서버 종료 중... (PID: $FRONTEND_PID)"
        kill "$FRONTEND_PID" 2>/dev/null || true
        
        # 종료 대기
        for i in {1..10}; do
            if ! ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
                log_success "프론트엔드 서버 종료 완료"
                STOPPED=$((STOPPED + 1))
                break
            fi
            sleep 1
        done
        
        # 강제 종료
        if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
            log_warning "프론트엔드 서버 강제 종료 중..."
            kill -9 "$FRONTEND_PID" 2>/dev/null || true
            log_success "프론트엔드 서버 강제 종료 완료"
            STOPPED=$((STOPPED + 1))
        fi
    else
        log_warning "프론트엔드 서버가 실행 중이 아닙니다 (PID: $FRONTEND_PID)"
    fi
    rm -f "$FRONTEND_PID_FILE"
else
    log_warning "프론트엔드 PID 파일이 없습니다"
fi

# 3. 포트 정리 (추가 안전장치)
log_info "포트 8080, 3000 정리 중..."
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
lsof -ti:3000 | xargs kill -9 2>/dev/null || true

# 4. 완료 메시지
echo ""
echo "================================================================================"
if [ $STOPPED -gt 0 ]; then
    log_success "전체 시스템 종료 완료! (종료된 서버: $STOPPED개)"
else
    log_warning "실행 중인 서버가 없었습니다"
fi
echo "================================================================================"
echo ""
echo "🚀 서버 시작: ./start-all.sh"
echo "🔄 서버 재시작: ./restart-all.sh"
echo ""

