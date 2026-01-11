#!/bin/bash

# 서버가 준비될 때까지 대기
echo "⏳ 서버 시작 대기 중..."
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 서버 준비 완료!"
        break
    fi
    echo "   대기 중... ($i/30)"
    sleep 2
done

# 로그인
echo ""
echo "🔐 로그인 중..."
TOKEN=$(curl -s -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}' \
  | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    echo "❌ 로그인 실패"
    exit 1
fi

echo "✅ 로그인 성공"
echo "   Token: ${TOKEN:0:50}..."

# Instances 엔드포인트 테스트
echo ""
echo "📊 Instances 엔드포인트 테스트..."
STUDY_UID="1.3.6.1.4.1.14519.5.2.1.6655.2359.333291521405118551454226683121"
SERIES_UID="1.3.6.1.4.1.14519.5.2.1.6655.2359.237178662106209442672676231119"

URL="http://localhost:8080/api/me/dicom/studies/${STUDY_UID}/series/${SERIES_UID}/instances?project_id=2&orderby=InstanceNumber"

echo "   URL: $URL"
echo ""

RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$URL" \
  -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "   HTTP Status: $HTTP_CODE"

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ 요청 성공!"
    echo ""
    echo "📄 응답 (처음 500자):"
    echo "$BODY" | jq '.' 2>/dev/null | head -20 || echo "$BODY" | head -20
    
    # 인스턴스 개수 확인
    COUNT=$(echo "$BODY" | jq 'length' 2>/dev/null)
    if [ ! -z "$COUNT" ] && [ "$COUNT" != "null" ]; then
        echo ""
        echo "📊 총 인스턴스 개수: $COUNT"
    fi
else
    echo "❌ 요청 실패!"
    echo ""
    echo "📄 에러 응답:"
    echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
fi

