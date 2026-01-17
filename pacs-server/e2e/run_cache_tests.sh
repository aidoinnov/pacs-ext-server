#!/bin/bash
# 캐시 관련 E2E 테스트 실행 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 캐시 관련 E2E 테스트 실행"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
NC='\033[0m'

echo "📋 테스트: HEAD 요청 및 캐시 검증"
echo "=========================================="
python3 test_annotation_head_request.py

echo ""
echo -e "${GREEN}🎉 캐시 테스트 완료!${NC}"
echo ""

