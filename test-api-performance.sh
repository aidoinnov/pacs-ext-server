#!/bin/bash

# 토큰 획득
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}' | jq -r '.token')

STUDY_UID="1.2.410.200022.500.202205101053010.12252192375"

echo "=== API 성능 비교 테스트 ==="
echo ""

# Test 1
echo "1️⃣  /api/dicom/studies/{study_uid}/series?project_id=2"
time curl -s "http://localhost:8080/api/dicom/studies/${STUDY_UID}/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" -o /dev/null 2>&1

echo ""

# Test 2
echo "2️⃣  /api/me/dicom/studies/{study_uid}/series?project_id=2"
time curl -s "http://localhost:8080/api/me/dicom/studies/${STUDY_UID}/series?project_id=2" \
  -H "Authorization: Bearer $TOKEN" -o /dev/null 2>&1

echo ""

# Test 3
echo "3️⃣  /api/me/dicom/studies/{study_uid}/series (no project_id)"
time curl -s "http://localhost:8080/api/me/dicom/studies/${STUDY_UID}/series" \
  -H "Authorization: Bearer $TOKEN" -o /dev/null 2>&1

