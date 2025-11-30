#!/bin/bash

# 어노테이션 API에서 user_role_name 필드가 반환되는지 테스트하는 스크립트

API_URL="http://localhost:8080"
PROJECT_ID=${1:-287}  # 기본값: 287

echo "🔍 Testing user_role_name field in Annotation API"
echo "=================================================="
echo ""

# 1. Health check
echo "1. Health Check:"
curl -s "$API_URL/health" | jq '.' 2>/dev/null || echo "Server not responding"
echo ""
echo ""

# 2. 프로젝트별 어노테이션 조회
echo "2. Testing GET /api/projects/$PROJECT_ID/annotations:"
response=$(curl -s "$API_URL/api/projects/$PROJECT_ID/annotations?limit=1" -H "Content-Type: application/json")
echo "$response" | jq '.annotations[0] | {id, user_id, user_name, user_role_name}' 2>/dev/null
if echo "$response" | jq -e '.annotations[0].user_role_name' > /dev/null 2>&1; then
    echo "✅ user_role_name field is present!"
    role_name=$(echo "$response" | jq -r '.annotations[0].user_role_name // "null"')
    echo "   Role name: $role_name"
else
    echo "❌ user_role_name field not found or error occurred"
    echo "   Full response:"
    echo "$response" | jq '.' 2>/dev/null || echo "$response"
fi
echo ""
echo ""

# 3. 단일 어노테이션 조회 (ID 1부터 10까지 시도)
echo "3. Testing GET /api/annotations/{id}:"
for id in 1 2 3 4 5; do
    response=$(curl -s "$API_URL/api/annotations/$id" -H "Content-Type: application/json")
    if echo "$response" | jq -e '.id' > /dev/null 2>&1; then
        echo "   Annotation ID $id:"
        echo "$response" | jq '{id, user_id, user_name, user_role_name}' 2>/dev/null
        if echo "$response" | jq -e '.user_role_name' > /dev/null 2>&1; then
            echo "   ✅ user_role_name field is present!"
        fi
        break
    fi
done
echo ""
echo ""

# 4. Summary API 테스트 (series_uid 필요)
echo "4. Testing GET /api/annotations/summary:"
echo "   (Note: Requires series_instance_uid parameter)"
echo "   Example: curl -s \"$API_URL/api/annotations/summary?project_id=$PROJECT_ID&series_instance_uid=YOUR_SERIES_UID&limit=1\" | jq '.annotations[0] | {id, user_id, user_name, user_role_name}'"
echo ""
echo ""

echo "=================================================="
echo "✅ Test completed!"
echo ""
echo "💡 Tip: If you see authentication errors, you may need to:"
echo "   1. Restart the server to apply new code"
echo "   2. Provide authentication token"
echo "   3. Create test annotations first"

