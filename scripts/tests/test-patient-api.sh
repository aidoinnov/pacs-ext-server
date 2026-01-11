#!/bin/bash

# Patient API 테스트 스크립트
# Keycloak 토큰을 받아서 /api/dicom/patients 엔드포인트 테스트

set -e

API_URL="http://localhost:8080"
PROJECT_ID=2

echo "========================================="
echo "🧪 Patient API 테스트"
echo "========================================="

# 1. Keycloak 토큰 획득
echo ""
echo "📝 1단계: Keycloak 토큰 획득 중..."
TOKEN_RESPONSE=$(curl -s -X POST "${API_URL}/api/auth/keycloak-token" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_super_admin",
    "password": "TestAdmin123!"
  }')

echo "Token Response: $TOKEN_RESPONSE"

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

if [ "$ACCESS_TOKEN" == "null" ] || [ -z "$ACCESS_TOKEN" ]; then
  echo "❌ 토큰 획득 실패!"
  echo "Response: $TOKEN_RESPONSE"
  exit 1
fi

echo "✅ 토큰 획득 성공!"
echo "   토큰 길이: ${#ACCESS_TOKEN}"
echo "   토큰 미리보기: ${ACCESS_TOKEN:0:50}..."

# 2. Patient API 호출 (전체 조회)
echo ""
echo "📝 2단계: Patient 전체 조회 (project_id=$PROJECT_ID)"
PATIENTS_RESPONSE=$(curl -s -X GET "${API_URL}/api/dicom/patients?project_id=${PROJECT_ID}" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Accept: application/json")

echo "Patients Response:"
echo "$PATIENTS_RESPONSE" | jq '.'

PATIENT_COUNT=$(echo "$PATIENTS_RESPONSE" | jq '. | length')
echo ""
echo "✅ Patient 개수: $PATIENT_COUNT"

# 3. Patient API 호출 (limit=1)
echo ""
echo "📝 3단계: Patient 조회 (limit=1)"
PATIENTS_LIMITED=$(curl -s -X GET "${API_URL}/api/dicom/patients?project_id=${PROJECT_ID}&limit=1" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Accept: application/json")

echo "Patients (limit=1):"
echo "$PATIENTS_LIMITED" | jq '.'

LIMITED_COUNT=$(echo "$PATIENTS_LIMITED" | jq '. | length')
echo ""
echo "✅ 반환된 Patient 개수: $LIMITED_COUNT (예상: 1)"

# 4. Patient API 호출 (필터링)
echo ""
echo "📝 4단계: Patient 조회 (PatientName 필터)"
PATIENTS_FILTERED=$(curl -s -X GET "${API_URL}/api/dicom/patients?project_id=${PROJECT_ID}&PatientName=*" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Accept: application/json")

echo "Patients (filtered):"
echo "$PATIENTS_FILTERED" | jq '.'

FILTERED_COUNT=$(echo "$PATIENTS_FILTERED" | jq '. | length')
echo ""
echo "✅ 필터링된 Patient 개수: $FILTERED_COUNT"

echo ""
echo "========================================="
echo "✅ 모든 테스트 완료!"
echo "========================================="

