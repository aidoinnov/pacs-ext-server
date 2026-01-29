#!/bin/bash

echo "🧪 QIDO 캐시 성능 테스트"
echo "================================"

# 로그인
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}' | jq -r '.token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ 로그인 실패"
  exit 1
fi

echo "✅ 로그인 성공"
echo ""

# 테스트 URL
STUDY_UID="1.2.410.200022.500.202205101053010.12252192375"
URL="http://localhost:8080/api/me/dicom/studies/${STUDY_UID}/series?project_id=2"

echo "📊 테스트 1: 첫 번째 요청 (Cache MISS 예상)"
echo "URL: $URL"
time curl -s "$URL" \
  -H "Authorization: Bearer $TOKEN" \
  | jq 'length' > /dev/null

echo ""
echo "📊 테스트 2: 두 번째 요청 (Cache HIT 예상)"
time curl -s "$URL" \
  -H "Authorization: Bearer $TOKEN" \
  | jq 'length' > /dev/null

echo ""
echo "📊 테스트 3: 세 번째 요청 (Cache HIT 예상)"
time curl -s "$URL" \
  -H "Authorization: Bearer $TOKEN" \
  | jq 'length' > /dev/null

echo ""
echo "================================"
echo "✅ 테스트 완료"
echo ""
echo "📝 백엔드 로그 확인:"
echo "tail -50 backend.log | grep -E '(Cache|QIDO)'"
