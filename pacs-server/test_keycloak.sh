#!/bin/bash
echo "=== Keycloak Configuration ==="
echo "URL: $APP_KEYCLOAK_URL"
echo "REALM: $APP_KEYCLOAK_REALM"
echo "ADMIN_USERNAME: $APP_KEYCLOAK_ADMIN_USERNAME"
echo ""
echo "Testing Keycloak connection..."
curl -s -o /dev/null -w "%{http_code}" "$APP_KEYCLOAK_URL/realms/$APP_KEYCLOAK_REALM/.well-known/openid-configuration"
echo ""
