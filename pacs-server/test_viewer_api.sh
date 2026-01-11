#!/bin/bash

# Viewer API 테스트 스크립트
# 
# 사용법:
#   ./test_viewer_api.sh <JWT_TOKEN>
#
# 예시:
#   ./test_viewer_api.sh "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

set -e

if [ -z "$1" ]; then
    echo "❌ Error: JWT token is required"
    echo "Usage: $0 <JWT_TOKEN>"
    exit 1
fi

JWT_TOKEN="$1"
BASE_URL="http://localhost:8080"

echo "🧪 Testing Viewer API Endpoints"
echo "================================"
echo ""

# Test 1: Study Meta API
echo "📋 Test 1: POST /api/v1/viewer/studies/meta"
echo "-------------------------------------------"
curl -X POST "${BASE_URL}/api/v1/viewer/studies/meta" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT_TOKEN}" \
  -d '{
    "study_uids": [
      "1.2.840.113619.2.55.3.604688433.1234",
      "1.2.840.113619.2.55.3.604688433.5678"
    ],
    "max_count": 20
  }' | jq '.'

echo ""
echo ""

# Test 2: Series Meta API
echo "📋 Test 2: POST /api/v1/viewer/series/meta"
echo "-------------------------------------------"
curl -X POST "${BASE_URL}/api/v1/viewer/series/meta" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT_TOKEN}" \
  -d '{
    "series_uids": [
      "1.2.840.113619.2.55.3.604688433.1234.1",
      "1.2.840.113619.2.55.3.604688433.1234.2"
    ],
    "max_count": 50
  }' | jq '.'

echo ""
echo ""

# Test 3: Study Meta API with invalid token
echo "📋 Test 3: POST /api/v1/viewer/studies/meta (Invalid Token)"
echo "------------------------------------------------------------"
curl -X POST "${BASE_URL}/api/v1/viewer/studies/meta" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer invalid_token" \
  -d '{
    "study_uids": ["1.2.840.113619.2.55.3.604688433.1234"]
  }' | jq '.'

echo ""
echo ""

# Test 4: Study Meta API with empty array
echo "📋 Test 4: POST /api/v1/viewer/studies/meta (Empty Array)"
echo "----------------------------------------------------------"
curl -X POST "${BASE_URL}/api/v1/viewer/studies/meta" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT_TOKEN}" \
  -d '{
    "study_uids": []
  }' | jq '.'

echo ""
echo ""

echo "✅ All tests completed!"

