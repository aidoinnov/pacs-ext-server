#!/bin/bash

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# 스크립트 디렉토리 (루트에서 실행되므로 scripts 폴더 기준)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}============================================================${NC}"
echo -e "${CYAN}🔗 PACS Database Tunnels - 통합 시작${NC}"
echo -e "${BLUE}============================================================${NC}"
echo -e "${WHITE}📂 Root Directory: ${ROOT_DIR}${NC}"
echo -e "${WHITE}📂 Script Directory: ${SCRIPT_DIR}${NC}"
echo -e "${BLUE}============================================================${NC}"

# Extension DB 터널 시작 (포트 5456)
echo -e "\n${CYAN}🔗 1. Extension DB 터널 시작 (포트 5456)...${NC}"
cd "$ROOT_DIR"
./scripts/db-tunnel.sh -t extension &
EXTENSION_PID=$!
sleep 3

# Postgres DB 터널 시작 (포트 5457 - Dcm4chee용)
echo -e "\n${CYAN}🔗 2. Postgres DB 터널 시작 (포트 5457 - Dcm4chee)...${NC}"
cd "$ROOT_DIR"
./scripts/db-tunnel.sh -t postgres &
POSTGRES_PID=$!
sleep 3

# 터널 상태 확인
echo -e "\n${BLUE}============================================================${NC}"
echo -e "${CYAN}📊 터널 상태 확인${NC}"
echo -e "${BLUE}============================================================${NC}"

# Extension 터널 확인 (포트 5456)
if lsof -i :5456 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Extension DB 터널 실행 중 (포트 5456)${NC}"
    lsof -i :5456 | head -3
else
    echo -e "${RED}❌ Extension DB 터널 실패 (포트 5456)${NC}"
fi

echo ""

# Postgres 터널 확인 (포트 5457)
if lsof -i :5457 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Postgres DB 터널 실행 중 (포트 5457)${NC}"
    lsof -i :5457 | head -3
else
    echo -e "${RED}❌ Postgres DB 터널 실패 (포트 5457)${NC}"
fi

echo -e "\n${BLUE}============================================================${NC}"
echo -e "${GREEN}✨ DB 터널 시작 완료!${NC}"
echo -e "${BLUE}============================================================${NC}"
echo -e "${WHITE}📌 Extension DB:${NC}"
echo -e "   - Host: localhost"
echo -e "   - Port: 5456"
echo -e "   - Database: pacs_db"
echo ""
echo -e "${WHITE}📌 Dcm4chee DB (Postgres):${NC}"
echo -e "   - Host: localhost"
echo -e "   - Port: 5457"
echo -e "   - Database: postgres"
echo ""
echo -e "${YELLOW}🛑 터널 중지:${NC}"
echo -e "   ./scripts/db-tunnel.sh -k -t both"
echo -e "${BLUE}============================================================${NC}"

