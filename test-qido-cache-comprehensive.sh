#!/bin/bash

echo "🧪 QIDO 캐시 종합 성능 테스트"
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

# Test 1: Series 엔드포인트 (project_id 포함)
echo "📊 Test 1: Series with project_id"
echo "----------------------------------------"
echo "Request 1 (Cache MISS):"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Series count:"
echo ""

sleep 0.5

echo "Request 2 (Cache HIT):"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Series count:"
echo ""

sleep 0.5

echo "Request 3 (Cache HIT):"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Series count:"
echo ""

# Test 2: Series 엔드포인트 (project_id 없음)
echo "📊 Test 2: Series without project_id"
echo "----------------------------------------"
echo "Request 1 (Cache MISS):"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Series count:"
echo ""

sleep 0.5

echo "Request 2 (Cache HIT):"
time curl -s "http://localhost:8080/api/me/dicom/studies/1.2.410.200022.500.202205101053010.12252192375/series" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Series count:"
echo ""

# Test 3: Studies 엔드포인트
echo "📊 Test 3: Studies with project_id"
echo "----------------------------------------"
echo "Request 1 (Cache MISS):"
time curl -s "http://localhost:8080/api/me/dicom/studies?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Studies count:"
echo ""

sleep 0.5

echo "Request 2 (Cache HIT):"
time curl -s "http://localhost:8080/api/me/dicom/studies?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Studies count:"
echo ""

sleep 0.5

echo "Request 3 (Cache HIT):"
time curl -s "http://localhost:8080/api/me/dicom/studies?project_id=2" \
  -H "Authorization: Bearer $TOKEN" | jq -r 'length' | xargs echo "Studies count:"
echo ""

echo "========================================"
echo "✅ 테스트 완료!"
echo ""
echo "📋 캐시 로그 확인:"
tail -30 backend.log | grep -E "(⚡|🔄)"
echo ""
echo "📋 Redis 캐시 키 확인:"
redis-cli KEYS "qido:*" | head -10

