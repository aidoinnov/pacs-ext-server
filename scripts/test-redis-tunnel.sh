#!/bin/bash

# Redis 터널 연결 테스트 스크립트

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "============================================"
echo "🔴 Redis 터널 연결 테스트"
echo "============================================"

# 1. 기존 터널 종료
echo -e "\n${YELLOW}1. 기존 Redis 터널 종료...${NC}"
"$SCRIPT_DIR/db-tunnel.sh" -k -t redis 2>/dev/null || true
sleep 2

# 2. Redis 터널 시작
echo -e "\n${YELLOW}2. Redis 터널 시작...${NC}"
"$SCRIPT_DIR/db-tunnel.sh" -t redis
sleep 3

# 3. 포트 확인
echo -e "\n${YELLOW}3. 포트 6379 확인...${NC}"
if lsof -i :6379 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 포트 6379 열림${NC}"
    lsof -i :6379 | head -3
else
    echo -e "${RED}❌ 포트 6379 안 열림${NC}"
    exit 1
fi

# 4. Redis PING 테스트 (nc 사용)
echo -e "\n${YELLOW}4. Redis PING 테스트 (1차)...${NC}"
RESPONSE=$(echo -e "PING\r\n" | nc -w 3 localhost 6379 2>/dev/null)
if echo "$RESPONSE" | grep -q "PONG"; then
    echo -e "${GREEN}✅ Redis 응답: PONG${NC}"
else
    echo -e "${RED}❌ Redis 응답 실패: $RESPONSE${NC}"
fi

# 5. 10초 대기
echo -e "\n${YELLOW}5. 10초 대기...${NC}"
for i in {10..1}; do
    echo -ne "\r   남은 시간: ${i}초  "
    sleep 1
done
echo ""

# 6. 포트 재확인
echo -e "\n${YELLOW}6. 포트 6379 재확인...${NC}"
if lsof -i :6379 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 포트 6379 여전히 열림${NC}"
else
    echo -e "${RED}❌ 포트 6379 닫힘 - 터널 끊김!${NC}"
    exit 1
fi

# 7. Redis PING 테스트 (2차)
echo -e "\n${YELLOW}7. Redis PING 테스트 (2차)...${NC}"
RESPONSE=$(echo -e "PING\r\n" | nc -w 3 localhost 6379 2>/dev/null)
if echo "$RESPONSE" | grep -q "PONG"; then
    echo -e "${GREEN}✅ Redis 응답: PONG${NC}"
else
    echo -e "${RED}❌ Redis 응답 실패: $RESPONSE${NC}"
    exit 1
fi

echo -e "\n============================================"
echo -e "${GREEN}✨ 테스트 완료! Redis 터널 안정적으로 유지됨${NC}"
echo "============================================"

