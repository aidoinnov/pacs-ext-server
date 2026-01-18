#!/bin/bash

# Subject & TimePoint E2E 테스트 실행 스크립트

set -e

# 색상 정의
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}🧪 Subject & TimePoint E2E Test${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 가상환경 활성화 확인
if [ -z "$VIRTUAL_ENV" ]; then
    echo -e "${YELLOW}⚠️  가상환경이 활성화되지 않았습니다.${NC}"
    echo "다음 명령어로 활성화하세요:"
    echo "  source venv/bin/activate"
    echo ""
    exit 1
fi

# pytest 설치 확인
if ! command -v pytest &> /dev/null; then
    echo -e "${RED}❌ pytest가 설치되지 않았습니다.${NC}"
    echo "다음 명령어로 설치하세요:"
    echo "  pip install -r requirements.txt"
    echo ""
    exit 1
fi

# 테스트 실행
echo -e "${GREEN}▶ Running Subject & TimePoint E2E tests...${NC}"
echo ""

pytest test_05_subject_timepoint.py -v -s --tb=short

# 결과 확인
if [ $? -eq 0 ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}✅ All tests passed!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
else
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${RED}❌ Some tests failed!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    exit 1
fi

