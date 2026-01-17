#!/bin/bash
# 버전 관리 관련 E2E 테스트 실행 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 버전 관리 E2E 테스트 실행"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
NC='\033[0m'

echo "📋 테스트: 버전 충돌 (Optimistic Locking)"
echo "=========================================="
python3 test_annotation_version_conflict.py

echo ""
echo -e "${GREEN}🎉 버전 관리 테스트 완료!${NC}"
echo ""

