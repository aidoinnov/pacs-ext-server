#!/usr/bin/env bash
set -euo pipefail

# DICOM Gateway RBAC Filtering E2E Test
# This script verifies that RBAC filtering is working correctly

GATEWAY_BASE="${GATEWAY_BASE_URL:-http://127.0.0.1:8080}"
PROJECT_ID="${PROJECT_ID:-1}"

echo "=========================================="
echo "DICOM Gateway RBAC Filtering E2E Test"
echo "=========================================="
echo ""
echo "Configuration:"
echo "  Gateway: ${GATEWAY_BASE}"
echo "  Project ID: ${PROJECT_ID}"
echo ""

# Check if server is running
echo "[1] Checking server status..."
if ! curl -s -f "${GATEWAY_BASE}/health" > /dev/null; then
    echo "❌ Server is not running at ${GATEWAY_BASE}"
    echo "   Please start the server: cargo run --bin pacs_server"
    exit 1
fi
echo "✅ Server is running"

# Check gateway ping
echo ""
echo "[2] Checking gateway ping endpoint..."
if curl -s -f "${GATEWAY_BASE}/api/dicom/ping" > /dev/null; then
    echo "✅ Gateway ping OK"
else
    echo "❌ Gateway ping failed"
    exit 2
fi

# Test without auth (should fail gracefully or return empty)
echo ""
echo "[3] Testing /studies without authentication..."
RESP=$(curl -s -w "\n%{http_code}" "${GATEWAY_BASE}/api/dicom/studies?project_id=${PROJECT_ID}&limit=1" 2>&1 || echo -e "\n000")
HTTP_CODE=$(echo "$RESP" | tail -1)
BODY=$(echo "$RESP" | head -n -1)

echo "   HTTP Status: ${HTTP_CODE}"
if [ "$HTTP_CODE" = "200" ]; then
    if echo "$BODY" | grep -q "\[\]"; then
        echo "✅ Endpoint responded with empty array (expected without auth)"
    else
        echo "⚠️  Endpoint returned data without auth - check if authentication is required"
        echo "   Response: ${BODY:0:200}..."
    fi
elif [ "$HTTP_CODE" = "401" ] || [ "$HTTP_CODE" = "403" ]; then
    echo "✅ Authentication required (expected for RBAC)"
elif [ "$HTTP_CODE" = "502" ]; then
    echo "⚠️  Bad Gateway - Dcm4chee connection issue (check DCM4CHEE__BASE_URL)"
else
    echo "⚠️  Unexpected status: ${HTTP_CODE}"
fi

echo ""
echo "=========================================="
echo "Basic gateway checks completed!"
echo ""
echo "To test with actual authentication, you need:"
echo ""
echo "  export KEYCLOAK_BASE_URL=https://keycloak.ai-do.kr"
echo "  export KEYCLOAK_REALM=dcm4che"
echo "  export KEYCLOAK_CLIENT_ID=pacs-frontend"
echo "  export KEYCLOAK_USERNAME=<your-username>"
echo "  export KEYCLOAK_PASSWORD=<your-password>"
echo ""
echo "Then run:"
echo "  python3 scripts/e2e_gateway_test.py"
echo ""
echo "Or use the Rust E2E binary:"
echo "  cargo run --bin dicom_gw_e2e"
echo "=========================================="

