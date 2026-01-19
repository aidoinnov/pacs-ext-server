#!/usr/bin/env python3
import requests
import json
import sys

# 로그인
login_url = 'http://localhost:8080/api/auth/login'
login_data = {
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
}

try:
    print("🔐 로그인 시도 중...")
    response = requests.post(login_url, json=login_data, timeout=10)
    print(f"Status Code: {response.status_code}")
    
    if response.status_code == 200:
        result = response.json()
        token = result.get('token') or result.get('access_token')
        if token:
            print(f"✅ 로그인 성공!")
            print(f"Token: {token[:50]}...")
            # 토큰을 파일에 저장
            with open('/tmp/pacs_token.txt', 'w') as f:
                f.write(token)
            print(f"Token saved to /tmp/pacs_token.txt")
            sys.exit(0)
        else:
            print("❌ Token not found in response")
            print(json.dumps(result, indent=2))
            sys.exit(1)
    else:
        print(f"❌ Login failed: {response.status_code}")
        print(f"Response: {response.text}")
        sys.exit(1)
except Exception as e:
    print(f'❌ Error: {e}')
    if hasattr(e, 'response') and e.response:
        print(f'Response: {e.response.text}')
    sys.exit(1)

