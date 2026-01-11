#!/bin/bash

echo "========================================="
echo "QIDO 인증 테스트"
echo "========================================="

# 1. 로그인해서 Keycloak 토큰 받기
echo ""
echo "1. 로그인 중..."
LOGIN_RESPONSE=$(curl -s -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.keycloak_access_token')
echo "✅ 토큰 받음 (길이: ${#TOKEN})"
echo "토큰 미리보기: ${TOKEN:0:50}..."

# 2. 토큰으로 QIDO 호출 (project_id 포함)
echo ""
echo "2. QIDO /studies 호출 중 (project_id=1)..."
QIDO_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
  -X GET "http://localhost:8080/api/dicom/studies?limit=1&project_id=1" \
  -H "Authorization: Bearer $TOKEN")

HTTP_STATUS=$(echo "$QIDO_RESPONSE" | grep "HTTP_STATUS" | cut -d: -f2)
BODY=$(echo "$QIDO_RESPONSE" | sed '/HTTP_STATUS/d')

echo "HTTP 상태: $HTTP_STATUS"
echo "응답 본문:"
echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

# 3. 백엔드 로그에서 최근 QIDO 관련 로그 확인
echo ""
echo "3. 백엔드 로그 확인..."
echo "최근 QIDO 관련 로그:"
tail -100 /Users/aido/Code/pacs-ext-server/backend.log | grep -i "qido\|401\|bearer" | tail -10

echo ""
echo "========================================="
echo "테스트 완료"
echo "========================================="

