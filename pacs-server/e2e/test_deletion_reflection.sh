#!/bin/bash
# 삭제가 즉시 목록에 반영되는지 확인

set -e

BASE_URL="http://localhost:8080"
ADMIN_USER="iaid-pacs-admin"
ADMIN_PASSWORD="Qlalfqjsgh1!"

echo "======================================================================"
echo "삭제 후 목록 반영 확인 테스트"
echo "======================================================================"
echo ""

# 로그인
TOKEN=$(curl -s -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASSWORD\"}" \
  | jq -r '.token')

echo "1️⃣ 초기 상태 확인"
RESPONSE1=$(curl -s -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")
COUNT1=$(echo "$RESPONSE1" | jq '.projects | length')
TOTAL1=$(echo "$RESPONSE1" | jq -r '.pagination.total')
echo "   응답 프로젝트 수: $COUNT1"
echo "   전체 프로젝트 수: $TOTAL1"
echo ""

# 프로젝트 생성
echo "2️⃣ 테스트 프로젝트 생성"
TIMESTAMP=$(date +%s)
PROJECT_ID=$(curl -s -X POST "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Delete Test $TIMESTAMP\",
    \"description\": \"Test\",
    \"sponsor\": \"Test\",
    \"start_date\": \"2026-01-01\",
    \"auto_complete\": false
  }" | jq -r '.id')
echo "   생성된 프로젝트 ID: $PROJECT_ID"
echo ""

# 생성 후 확인
echo "3️⃣ 생성 후 목록 확인"
RESPONSE2=$(curl -s -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")
COUNT2=$(echo "$RESPONSE2" | jq '.projects | length')
TOTAL2=$(echo "$RESPONSE2" | jq -r '.pagination.total')
echo "   응답 프로젝트 수: $COUNT2"
echo "   전체 프로젝트 수: $TOTAL2"

# 생성된 프로젝트가 목록에 있는지 확인
FOUND=$(echo "$RESPONSE2" | jq ".projects[] | select(.id == $PROJECT_ID) | .id")
if [ -n "$FOUND" ]; then
  echo "   ✅ 생성된 프로젝트가 목록에 있음"
else
  echo "   ❌ 생성된 프로젝트가 목록에 없음"
fi
echo ""

# 프로젝트 삭제
echo "4️⃣ 프로젝트 삭제"
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE_URL/api/projects/$PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")
echo "   삭제 HTTP Status: $HTTP_STATUS"
if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "204" ]; then
  echo "   ✅ 삭제 성공"
else
  echo "   ❌ 삭제 실패"
fi
echo ""

# 삭제 후 즉시 확인
echo "5️⃣ 삭제 후 즉시 목록 확인"
RESPONSE3=$(curl -s -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")
COUNT3=$(echo "$RESPONSE3" | jq '.projects | length')
TOTAL3=$(echo "$RESPONSE3" | jq -r '.pagination.total')
echo "   응답 프로젝트 수: $COUNT3"
echo "   전체 프로젝트 수: $TOTAL3"

# 삭제된 프로젝트가 목록에 없는지 확인
FOUND=$(echo "$RESPONSE3" | jq ".projects[] | select(.id == $PROJECT_ID) | .id")
if [ -z "$FOUND" ]; then
  echo "   ✅ 삭제된 프로젝트가 목록에서 사라짐"
else
  echo "   ❌ 삭제된 프로젝트가 여전히 목록에 있음!"
fi
echo ""

# 결과 요약
echo "======================================================================"
echo "결과 요약"
echo "======================================================================"
echo "초기 전체 개수: $TOTAL1"
echo "생성 후 전체 개수: $TOTAL2 (증가: $((TOTAL2 - TOTAL1)))"
echo "삭제 후 전체 개수: $TOTAL3 (감소: $((TOTAL2 - TOTAL3)))"
echo ""

if [ "$TOTAL3" = "$TOTAL1" ]; then
  echo "✅ 삭제가 즉시 목록에 반영되었습니다!"
  echo "   초기 개수($TOTAL1) = 삭제 후 개수($TOTAL3)"
else
  echo "⚠️ 개수가 예상과 다릅니다."
  echo "   초기 개수($TOTAL1) ≠ 삭제 후 개수($TOTAL3)"
fi
echo ""

# 전체 프로젝트 목록 출력
echo "6️⃣ 현재 전체 프로젝트 목록"
echo "$RESPONSE3" | jq -r '.projects[] | "   ID: \(.id), Name: \(.name)"'
echo ""

