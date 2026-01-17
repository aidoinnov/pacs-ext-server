#!/bin/bash

# GC Runner E2E Test Runner
# 이 스크립트는 Python E2E 테스트를 실행합니다.

set -e

# 색상 코드
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🧪 GC Runner E2E Test Runner${NC}"
echo "================================"
echo ""

# 1. GC Runner 빌드
echo -e "${YELLOW}📦 Building GC Runner...${NC}"
/Users/aido/.cargo/bin/cargo build --bin gc_runner

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build completed${NC}"
echo ""

# 2. Python 의존성 확인
echo -e "${YELLOW}🐍 Checking Python dependencies...${NC}"

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ python3 not found${NC}"
    exit 1
fi

# psycopg2 설치 확인
if ! python3 -c "import psycopg2" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  psycopg2 not found. Installing...${NC}"
    pip3 install psycopg2-binary
fi

echo -e "${GREEN}✅ Python dependencies OK${NC}"
echo ""

# 3. 환경 변수 설정
export DATABASE_URL="${DATABASE_URL:-postgresql://aido@localhost:5432/pacs_db}"
export GC_RUNNER_PATH="../../target/debug/gc_runner"
export S3_BUCKET="${S3_BUCKET:-test-bucket}"
export S3_REGION="${S3_REGION:-us-east-1}"
export S3_ACCESS_KEY="${S3_ACCESS_KEY:-test}"
export S3_SECRET_KEY="${S3_SECRET_KEY:-test}"
export S3_ENDPOINT="${S3_ENDPOINT:-http://localhost:9000}"

echo -e "${YELLOW}🔧 Environment:${NC}"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  GC_RUNNER_PATH: $GC_RUNNER_PATH"
echo ""

# 4. E2E 테스트 실행
echo -e "${YELLOW}🚀 Running E2E tests...${NC}"
echo ""

python3 test_gc_e2e.py

# 5. 결과 출력
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ All E2E tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ E2E tests failed${NC}"
    exit 1
fi

