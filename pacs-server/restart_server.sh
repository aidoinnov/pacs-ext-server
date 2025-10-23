#!/bin/bash

# PACS Server 강제 리스타트 스크립트
# 사용법: ./restart_server.sh

echo "🔄 PACS Server 강제 리스타트 시작..."

# 현재 디렉토리를 pacs-server로 변경
cd "$(dirname "$0")"

echo "📁 현재 디렉토리: $(pwd)"

# 1. 기존 서버 프로세스 강제 종료
echo "🛑 기존 서버 프로세스 종료 중..."
pkill -f "pacs-server" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true
pkill -f "target/release/pacs-server" 2>/dev/null || true

# 잠시 대기
sleep 2

# 2. 포트 사용 중인 프로세스 확인 및 종료
echo "🔍 포트 8080 사용 중인 프로세스 확인..."
if lsof -ti:8080 > /dev/null 2>&1; then
    echo "⚠️  포트 8080이 사용 중입니다. 강제 종료합니다..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

# 3. 빌드 캐시 정리 (선택사항)
echo "🧹 빌드 캐시 정리 중..."
cargo clean > /dev/null 2>&1 || true

# 4. 환경 변수 확인
echo "🔧 환경 변수 확인..."
if [ -f ".env" ]; then
    echo "✅ .env 파일 발견"
    echo "📋 S3 설정 확인:"
    grep "OBJECT_STORAGE" .env | head -5
else
    echo "⚠️  .env 파일이 없습니다!"
fi

# 5. 서버 시작
echo "🚀 서버 시작 중..."
echo "=========================================="

# 백그라운드에서 서버 실행
nohup cargo run > server.log 2>&1 &
SERVER_PID=$!

# 서버 시작 대기
echo "⏳ 서버 시작 대기 중..."
sleep 5

# 서버 상태 확인
if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo "✅ 서버가 성공적으로 시작되었습니다! (PID: $SERVER_PID)"
    echo "🌐 서버 URL: http://localhost:8080"
    echo "❤️  Health Check: http://localhost:8080/health"
    echo "📖 Swagger UI: http://localhost:8080/swagger-ui/"
    echo "📝 로그 파일: server.log"
    echo ""
    echo "서버를 중지하려면: kill $SERVER_PID"
    echo "로그를 보려면: tail -f server.log"
else
    echo "❌ 서버 시작에 실패했습니다!"
    echo "📝 로그 확인:"
    tail -20 server.log
    exit 1
fi

echo "=========================================="
echo "🎉 PACS Server 리스타트 완료!"
