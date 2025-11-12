#!/bin/bash

# 1. 로그인하여 토큰 획득
echo "=== 1. 로그인 ==="
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"dr_kim_seq","password":"Test1234!"}')

echo "Login Response: $LOGIN_RESPONSE"

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token')
echo "Token: ${TOKEN:0:50}..."

# 2. DICOM API 호출 (project_id 포함)
echo ""
echo "=== 2. DICOM API 호출 ==="
curl -v -X GET "http://localhost:8080/api/dicom/studies?project_id=262" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/json"
