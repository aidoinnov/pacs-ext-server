#!/bin/bash

# RECIST Lesion E2E 테스트 실행 스크립트

set -e

echo "=========================================="
echo "RECIST Lesion E2E Test Runner"
echo "=========================================="
echo ""

# 현재 디렉토리 확인
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# 가상환경 활성화 (있는 경우)
if [ -d "venv" ]; then
    echo "Activating virtual environment..."
    source venv/bin/activate
fi

# 환경 변수 확인
if [ -z "$BASE_URL" ]; then
    export BASE_URL="http://localhost:8080"
    echo "Using default BASE_URL: $BASE_URL"
fi

if [ -z "$ADMIN_EMAIL" ]; then
    export ADMIN_EMAIL="admin@example.com"
    echo "Using default ADMIN_EMAIL: $ADMIN_EMAIL"
fi

if [ -z "$ADMIN_PASSWORD" ]; then
    export ADMIN_PASSWORD="admin123"
    echo "Using default ADMIN_PASSWORD: ********"
fi

echo ""
echo "=========================================="
echo "Running RECIST Lesion Tests..."
echo "=========================================="
echo ""

# pytest 실행
pytest test_07_recist_lesion.py -v -s --tb=short

echo ""
echo "=========================================="
echo "Test completed!"
echo "=========================================="

