#!/bin/bash
# DB에서 직접 삭제 후 API 응답 확인 테스트

set -e

BASE_URL="http://localhost:8080"
ADMIN_USER="iaid-pacs-admin"
ADMIN_PASSWORD="Qlalfqjsgh1!"

echo "======================================================================"
echo "DB 직접 삭제 후 API 응답 확인 테스트"
echo "======================================================================"
echo ""

# 1. 로그인
echo "1️⃣ 로그인..."
TOKEN=$(curl -s -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASSWORD\"}" \
  | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "❌ 로그인 실패"
  exit 1
fi
echo "✅ 로그인 성공"
echo ""

# 2. 초기 상태 확인
echo "2️⃣ 초기 프로젝트 목록 조회..."
RESPONSE1=$(curl -s -i -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")

ETAG1=$(echo "$RESPONSE1" | grep -i "^etag:" | cut -d' ' -f2 | tr -d '\r')
COUNT1=$(echo "$ETAG1" | sed 's/.*-\([0-9]*\)".*/\1/')

echo "   초기 ETag: $ETAG1"
echo "   초기 개수: $COUNT1"
echo ""

# 3. 테스트 프로젝트 생성
echo "3️⃣ 테스트 프로젝트 생성..."
TIMESTAMP=$(date +%s)
PROJECT_ID=$(curl -s -X POST "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"DB Delete Test $TIMESTAMP\",
    \"description\": \"Test\",
    \"sponsor\": \"Test\",
    \"start_date\": \"2026-01-01\",
    \"auto_complete\": false
  }" | jq -r '.id')

echo "   생성된 프로젝트 ID: $PROJECT_ID"
echo ""

# 4. 생성 후 ETag 확인
echo "4️⃣ 생성 후 ETag 확인..."
RESPONSE2=$(curl -s -i -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")

ETAG2=$(echo "$RESPONSE2" | grep -i "^etag:" | cut -d' ' -f2 | tr -d '\r')
COUNT2=$(echo "$ETAG2" | sed 's/.*-\([0-9]*\)".*/\1/')

echo "   생성 후 ETag: $ETAG2"
echo "   생성 후 개수: $COUNT2"
echo ""

# 5. DB에서 직접 삭제
echo "5️⃣ DB에서 직접 삭제..."
echo "   프로젝트 ID $PROJECT_ID 를 DB에서 삭제합니다..."

PGPASSWORD=admin123 psql -h localhost -p 5432 -U admin -d pacs_db -c \
  "DELETE FROM security_project WHERE id = $PROJECT_ID;" 2>&1 | grep -v "^$"

echo "   ✅ DB에서 직접 삭제 완료"
echo ""

# 6. 삭제 후 즉시 조회 (ETag 없이)
echo "6️⃣ 삭제 후 즉시 조회 (ETag 없이)..."
RESPONSE3=$(curl -s -i -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN")

ETAG3=$(echo "$RESPONSE3" | grep -i "^etag:" | cut -d' ' -f2 | tr -d '\r')
COUNT3=$(echo "$ETAG3" | sed 's/.*-\([0-9]*\)".*/\1/')

echo "   삭제 후 ETag: $ETAG3"
echo "   삭제 후 개수: $COUNT3"
echo ""

# 7. 이전 ETag로 조회 (If-None-Match)
echo "7️⃣ 이전 ETag로 조회 (If-None-Match)..."
echo "   If-None-Match: $ETAG2"

HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/api/projects" \
  -H "Authorization: Bearer $TOKEN" \
  -H "If-None-Match: $ETAG2")

echo "   HTTP Status: $HTTP_STATUS"

if [ "$HTTP_STATUS" = "200" ]; then
  echo "   ✅ 200 OK - ETag 변경 감지!"
elif [ "$HTTP_STATUS" = "304" ]; then
  echo "   ❌ 304 Not Modified - ETag가 변경되지 않음!"
fi
echo ""

# 8. 결과 요약
echo "======================================================================"
echo "결과 요약"
echo "======================================================================"
echo "초기 개수: $COUNT1"
echo "생성 후 개수: $COUNT2 (증가: $((COUNT2 - COUNT1)))"
echo "삭제 후 개수: $COUNT3 (감소: $((COUNT2 - COUNT3)))"
echo ""

if [ "$COUNT3" = "$COUNT1" ]; then
  echo "✅ DB에서 직접 삭제한 것이 즉시 반영되었습니다!"
  echo "✅ ETag가 변경되어 클라이언트가 감지할 수 있습니다!"
else
  echo "⚠️ 개수가 예상과 다릅니다."
fi
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
  echo "✅ If-None-Match 헤더 사용 시 200 OK 반환 (정상)"
  echo "   → 클라이언트는 새 데이터를 받습니다"
elif [ "$HTTP_STATUS" = "304" ]; then
  echo "❌ If-None-Match 헤더 사용 시 304 Not Modified 반환 (문제!)"
  echo "   → 클라이언트는 오래된 캐시를 사용합니다"
fi
echo ""

