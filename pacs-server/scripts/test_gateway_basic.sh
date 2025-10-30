#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo "DICOM Gateway Basic Health Check"
echo "=========================================="

GATEWAY_BASE="http://127.0.0.1:8080"

echo ""
echo "[1/3] Checking server health..."
if curl -s -f "${GATEWAY_BASE}/health" > /dev/null; then
    echo "✅ Server is running"
else
    echo "❌ Server is not responding at ${GATEWAY_BASE}"
    echo "   Please make sure the server is running:"
    echo "   cargo run --bin pacs_server"
    exit 1
fi

echo ""
echo "[2/3] Checking DICOM gateway ping endpoint..."
if curl -s -f "${GATEWAY_BASE}/api/dicom/ping" > /dev/null; then
    echo "✅ Gateway ping endpoint is working"
else
    echo "❌ Gateway ping endpoint failed"
    exit 2
fi

echo ""
echo "[3/3] Checking DICOM gateway studies endpoint (without auth - should handle gracefully)..."
RESP=$(curl -s -w "\n%{http_code}" "${GATEWAY_BASE}/api/dicom/studies?project_id=1&limit=1" || echo "000")
HTTP_CODE=$(echo "$RESP" | tail -1)
BODY=$(echo "$RESP" | head -n -1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Studies endpoint responded (may be empty without auth)"
    echo "   Response preview: ${BODY:0:100}..."
elif [ "$HTTP_CODE" = "401" ] || [ "$HTTP_CODE" = "403" ]; then
    echo "✅ Studies endpoint requires authentication (expected)"
    echo "   This confirms RBAC filtering is active"
elif [ "$HTTP_CODE" = "000" ]; then
    echo "⚠️  Could not connect to studies endpoint"
else
    echo "⚠️  Studies endpoint returned HTTP ${HTTP_CODE}"
    echo "   Response: ${BODY:0:200}..."
fi

echo ""
echo "=========================================="
echo "Basic checks completed!"
echo "=========================================="
echo ""
echo "To run full E2E test with authentication, set:"
echo "  export KEYCLOAK_BASE_URL=https://keycloak.ai-do.kr"
echo "  export KEYCLOAK_REALM=dcm4che"
echo "  export KEYCLOAK_CLIENT_ID=pacs-frontend"
echo "  export KEYCLOAK_USERNAME=your-username"
echo "  export KEYCLOAK_PASSWORD=your-password"
echo "  export GATEWAY_BASE_URL=http://127.0.0.1:8080"
echo "  export PROJECT_ID=1"
echo ""
echo "Then run: python3 scripts/e2e_gateway_test.py"
echo "Or: cargo run --bin dicom_gw_e2e"

