#!/bin/bash
# GC Runner CLI 통합 테스트

set -e

echo "🧪 GC Runner CLI Integration Test"
echo "=================================="

# 환경 변수 설정
export DATABASE_URL="postgresql://aido@localhost:5432/pacs_db"
export S3_BUCKET="test-bucket"
export S3_REGION="ap-northeast-2"
export S3_ACCESS_KEY="test-key"
export S3_SECRET_KEY="test-secret"

# 바이너리 경로
GC_RUNNER="./target/debug/gc_runner"

# 빌드 확인
if [ ! -f "$GC_RUNNER" ]; then
    echo "❌ gc_runner binary not found. Building..."
    cargo build --bin gc_runner
fi

echo ""
echo "📋 Test 1: Help 명령어"
echo "----------------------"
$GC_RUNNER --help
echo "✅ Test 1 passed"

echo ""
echo "📋 Test 2: Job A Help"
echo "----------------------"
$GC_RUNNER timeout-pending --help
echo "✅ Test 2 passed"

echo ""
echo "📋 Test 3: Job B Help"
echo "----------------------"
$GC_RUNNER cleanup-failed --help
echo "✅ Test 3 passed"

echo ""
echo "📋 Test 4: Job A Dry-run (기본 파라미터)"
echo "----------------------------------------"
$GC_RUNNER timeout-pending --dry-run
echo "✅ Test 4 passed"

echo ""
echo "📋 Test 5: Job A Dry-run (커스텀 파라미터)"
echo "-------------------------------------------"
$GC_RUNNER timeout-pending --grace-days 5 --batch-size 50 --dry-run
echo "✅ Test 5 passed"

echo ""
echo "📋 Test 6: Job B Dry-run (기본 파라미터)"
echo "----------------------------------------"
$GC_RUNNER cleanup-failed --dry-run
echo "✅ Test 6 passed"

echo ""
echo "📋 Test 7: Job B Dry-run (커스텀 파라미터)"
echo "-------------------------------------------"
$GC_RUNNER cleanup-failed --grace-days 14 --batch-size 100 --dry-run
echo "✅ Test 7 passed"

echo ""
echo "=================================="
echo "✅ All CLI tests passed!"
echo "=================================="

