#!/bin/bash

# PACS Server Linux Build & Test Script
# 리눅스 서버에서 실행하세요

set -e  # 에러 발생시 스크립트 중단

echo "🚀 PACS Server Linux Build & Test 시작"
echo "=================================="

# 1. 현재 디렉토리 확인
echo "📁 현재 디렉토리: $(pwd)"
echo "📋 파일 목록:"
ls -la

# 2. Docker 설치 확인
echo ""
echo "🐳 Docker 설치 확인:"
if command -v docker &> /dev/null; then
    echo "✅ Docker 버전: $(docker --version)"
else
    echo "❌ Docker가 설치되지 않았습니다!"
    exit 1
fi

# 3. GLIBC 버전 확인
echo ""
echo "📚 GLIBC 버전 확인:"
if [ -f /lib/x86_64-linux-gnu/libc.so.6 ]; then
    /lib/x86_64-linux-gnu/libc.so.6 | head -1
elif [ -f /lib64/libc.so.6 ]; then
    /lib64/libc.so.6 | head -1
else
    echo "⚠️  GLIBC 버전을 확인할 수 없습니다"
fi

# 4. Docker 이미지 빌드
echo ""
echo "🔨 Docker 이미지 빌드 시작..."
docker build -t pacs-server:linux-test .

if [ $? -eq 0 ]; then
    echo "✅ Docker 이미지 빌드 성공!"
else
    echo "❌ Docker 이미지 빌드 실패!"
    exit 1
fi

# 5. 이미지 정보 확인
echo ""
echo "📊 빌드된 이미지 정보:"
docker images pacs-server:linux-test

# 6. 컨테이너 내부 GLIBC 확인
echo ""
echo "🔍 컨테이너 내부 GLIBC 버전:"
docker run --rm pacs-server:linux-test /lib/x86_64-linux-gnu/libc.so.6 | head -1 || \
docker run --rm pacs-server:linux-test /lib64/libc.so.6 | head -1 || \
echo "⚠️  컨테이너 내부 GLIBC 버전을 확인할 수 없습니다"

# 7. 바이너리 의존성 확인
echo ""
echo "🔗 바이너리 의존성 확인:"
docker run --rm pacs-server:linux-test ldd /app/pacs-server

# 8. 바이너리 실행 테스트
echo ""
echo "▶️  바이너리 실행 테스트:"
echo "실행 중... (5초 후 중단)"
timeout 5s docker run --rm pacs-server:linux-test || echo "⏰ 5초 후 정상적으로 중단됨"

# 9. GLIBC 심볼 확인
echo ""
echo "🔍 바이너리가 요구하는 GLIBC 심볼들:"
docker run --rm pacs-server:linux-test strings /app/pacs-server | grep GLIBC | sort | uniq

echo ""
echo "🎉 테스트 완료!"
echo "=================================="
echo "💡 만약 GLIBC 에러가 발생하지 않았다면, 맥의 Docker 에뮬레이션 문제였습니다."
echo "💡 만약 여전히 에러가 발생한다면, 다른 원인을 찾아야 합니다."
