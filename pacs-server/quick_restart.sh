#!/bin/bash

# PACS Server 빠른 리스타트 스크립트
echo "🔄 빠른 리스타트..."

# 기존 프로세스 종료
pkill -f "pacs-server" 2>/dev/null || true
pkill -f "cargo run --bin pacs_server" 2>/dev/null || true

# 포트 강제 해제
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# 잠시 대기
sleep 1

# 서버 시작
echo "🚀 서버 시작..."
cargo run
