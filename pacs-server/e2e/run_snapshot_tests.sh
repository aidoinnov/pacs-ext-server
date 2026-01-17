#!/bin/bash
# 스냅샷 관련 E2E 테스트 실행 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 스냅샷 E2E 테스트 실행"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
NC='\033[0m'

echo "📋 테스트: 스냅샷 업로드"
echo "=========================================="
python3 test_annotation_snapshot_e2e.py

echo ""
echo -e "${GREEN}🎉 스냅샷 테스트 완료!${NC}"
echo ""

