#!/bin/bash

# PACS Server Linux 배포용 패키징 스크립트
# 맥에서 실행하여 리눅스 서버로 전송할 파일을 준비합니다

set -e

echo "📦 PACS Server Linux 배포용 패키징 시작"
echo "======================================"

# 1. 현재 디렉토리 확인
if [ ! -f "Cargo.toml" ]; then
    echo "❌ pacs-server 디렉토리에서 실행해주세요!"
    exit 1
fi

# 2. 패키지 이름 생성 (타임스탬프 포함)
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
PACKAGE_NAME="pacs-server-linux-${TIMESTAMP}.tar.gz"

echo "📁 패키지 이름: ${PACKAGE_NAME}"

# 3. 불필요한 파일 제외하고 패키징
echo "🗜️  파일 압축 중..."
tar -czf "../${PACKAGE_NAME}" \
    --exclude='target/' \
    --exclude='.git/' \
    --exclude='*.log' \
    --exclude='.env*' \
    --exclude='ssl/' \
    --exclude='test_images/' \
    --exclude='server.log' \
    --exclude='.sqlx/' \
    --exclude='*.tar.gz' \
    .

# 4. 패키지 정보 출력
echo ""
echo "✅ 패키징 완료!"
echo "📊 패키지 정보:"
ls -lh "../${PACKAGE_NAME}"

echo ""
echo "🚀 리눅스 서버로 전송하는 방법:"
echo "scp ../${PACKAGE_NAME} user@server:/path/to/destination/"
echo ""
echo "📋 리눅스 서버에서 실행할 명령어:"
echo "tar -xzf ${PACKAGE_NAME}"
echo "cd pacs-server"
echo "chmod +x scripts/linux-build-test.sh"
echo "./scripts/linux-build-test.sh"
