#!/usr/bin/env python3
"""Role 목록 확인 스크립트"""

import requests
import json

BASE_URL = "http://localhost:8080"

# 로그인
login_resp = requests.post(
    f"{BASE_URL}/api/auth/login",
    json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
    timeout=5
)

if login_resp.status_code == 200:
    token = login_resp.json()["token"]
    print(f"✅ 로그인 성공\n")
    
    # Role 목록 조회
    roles_resp = requests.get(
        f"{BASE_URL}/api/roles",
        headers={"Authorization": f"Bearer {token}"},
        timeout=5
    )
    
    print(f"Status Code: {roles_resp.status_code}")
    print(f"Response:\n{json.dumps(roles_resp.json(), indent=2, ensure_ascii=False)}")
else:
    print(f"❌ 로그인 실패: {login_resp.status_code}")

