#!/bin/bash

echo "🧪 QIDO 캐시 최종 성능 테스트"
echo "========================================"
echo ""

# 토큰 획득
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}' | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "❌ Failed to obtain token"
  exit 1
fi

echo "✅ Token obtained"
echo ""

# Test 1: Cache MISS (첫 요청)
echo "📊 Test 1: Cache MISS (첫 요청)"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
echo ""

sleep 1

# Test 2-5: Cache HIT
for i in {2..5}; do
  echo "📊 Test $i: Cache HIT"
  time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series?project_id=2" \
    -H "Authorization: Bearer $TOKEN" > /dev/null
  echo ""
  sleep 0.5
done

echo "========================================"
echo "✅ 테스트 완료!"
echo ""
echo "📋 로그 확인:"
tail -20 backend.log | grep -E "(⚡|🔄)"
