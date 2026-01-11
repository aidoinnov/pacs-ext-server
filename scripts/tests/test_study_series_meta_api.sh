#!/bin/bash

# Test script for GET /api/v1/viewer/studies/{study_uid}/series/meta (with pagination)

BASE_URL="http://localhost:8080"

echo "=== Testing GET /api/v1/viewer/studies/{study_uid}/series/meta (Pagination) ==="
echo ""

# Step 1: Login to get JWT token
echo "Step 1: Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Login failed. Response:"
  echo $LOGIN_RESPONSE | jq .
  exit 1
fi

echo "✅ Login successful. Token obtained."
echo ""

# Replace with an actual StudyInstanceUID from your PACS
STUDY_UID="1.2.840.113619.2.55.3.604688433.1234"

# Test 1: Default pagination (page=1, page_size=50)
echo "=========================================="
echo "Test 1: Default pagination"
echo "=========================================="
echo "Fetching series for study: $STUDY_UID"
echo ""

RESPONSE_1=$(curl -s -X GET "${BASE_URL}/api/v1/viewer/studies/${STUDY_UID}/series/meta" \
  -H "Authorization: Bearer ${TOKEN}")

echo "Response:"
echo $RESPONSE_1 | jq .

if echo $RESPONSE_1 | jq -e '.pagination' > /dev/null 2>&1; then
  echo ""
  echo "✅ Test 1 successful!"
  echo "   Pagination info:"
  echo $RESPONSE_1 | jq '.pagination'
  echo ""
  echo "   Series returned: $(echo $RESPONSE_1 | jq '.series | length')"
fi

echo ""

# Test 2: Custom pagination (page=1, page_size=10)
echo "=========================================="
echo "Test 2: Custom pagination (page=1, page_size=10)"
echo "=========================================="

RESPONSE_2=$(curl -s -X GET "${BASE_URL}/api/v1/viewer/studies/${STUDY_UID}/series/meta?page=1&page_size=10" \
  -H "Authorization: Bearer ${TOKEN}")

echo "Response:"
echo $RESPONSE_2 | jq .

if echo $RESPONSE_2 | jq -e '.pagination' > /dev/null 2>&1; then
  echo ""
  echo "✅ Test 2 successful!"
  echo "   Pagination info:"
  echo $RESPONSE_2 | jq '.pagination'
  echo ""
  echo "   Series returned: $(echo $RESPONSE_2 | jq '.series | length')"

  # Display first series
  if [ "$(echo $RESPONSE_2 | jq '.series | length')" -gt 0 ]; then
    echo ""
    echo "   First series:"
    echo $RESPONSE_2 | jq '.series[0] | {series_uid, series_number, series_description, modality}'
  fi
fi

echo ""

# Test 3: Page 2
echo "=========================================="
echo "Test 3: Page 2 (page=2, page_size=10)"
echo "=========================================="

RESPONSE_3=$(curl -s -X GET "${BASE_URL}/api/v1/viewer/studies/${STUDY_UID}/series/meta?page=2&page_size=10" \
  -H "Authorization: Bearer ${TOKEN}")

echo "Response:"
echo $RESPONSE_3 | jq .

if echo $RESPONSE_3 | jq -e '.pagination' > /dev/null 2>&1; then
  echo ""
  echo "✅ Test 3 successful!"
  echo "   Pagination info:"
  echo $RESPONSE_3 | jq '.pagination'
  echo ""
  echo "   Series returned: $(echo $RESPONSE_3 | jq '.series | length')"

  # Display first series
  if [ "$(echo $RESPONSE_3 | jq '.series | length')" -gt 0 ]; then
    echo ""
    echo "   First series:"
    echo $RESPONSE_3 | jq '.series[0] | {series_uid, series_number, series_description, modality}'
  fi
fi

echo ""

# Test 4: Large page_size (should be clamped to 200)
echo "=========================================="
echo "Test 4: Large page_size (page=1, page_size=500)"
echo "=========================================="

RESPONSE_4=$(curl -s -X GET "${BASE_URL}/api/v1/viewer/studies/${STUDY_UID}/series/meta?page=1&page_size=500" \
  -H "Authorization: Bearer ${TOKEN}")

if echo $RESPONSE_4 | jq -e '.pagination' > /dev/null 2>&1; then
  ACTUAL_PAGE_SIZE=$(echo $RESPONSE_4 | jq '.pagination.page_size')
  echo "✅ Test 4 successful!"
  echo "   Requested page_size: 500"
  echo "   Actual page_size (should be clamped to 200): $ACTUAL_PAGE_SIZE"
  echo "   Series returned: $(echo $RESPONSE_4 | jq '.series | length')"
fi

echo ""
echo "=== All tests completed ==="

