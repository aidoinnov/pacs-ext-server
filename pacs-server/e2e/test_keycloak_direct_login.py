#!/usr/bin/env python3
"""
Keycloak에 직접 로그인해서 토큰 받기
"""

import requests
import json

# Keycloak 설정
KEYCLOAK_URL = "https://keycloak.pacs.ai-do.kr"
REALM = "dcm4che"
CLIENT_ID = "pacs-extension-server"
CLIENT_SECRET = "85TSWxK8ruF750z0Qzh0tQZ8xH5h3y99"  # 코드에서 확인한 시크릿

# 사용자 정보
USERNAME = "iaid-pacs-admin"
PASSWORD = "Qlalfqjsgh1!"

# Keycloak 토큰 엔드포인트
TOKEN_URL = f"{KEYCLOAK_URL}/realms/{REALM}/protocol/openid-connect/token"

print("=" * 70)
print("Keycloak 직접 로그인 테스트")
print("=" * 70)
print(f"Keycloak URL: {KEYCLOAK_URL}")
print(f"Realm: {REALM}")
print(f"Client ID: {CLIENT_ID}")
print(f"Username: {USERNAME}")
print()

# Keycloak에 직접 로그인
print("🔐 Keycloak에 직접 로그인 중...")
token_data = {
    "grant_type": "password",
    "client_id": CLIENT_ID,
    "client_secret": CLIENT_SECRET,
    "username": USERNAME,
    "password": PASSWORD,
}

try:
    resp = requests.post(TOKEN_URL, data=token_data, timeout=10)
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        token_info = resp.json()
        access_token = token_info.get("access_token")
        print(f"✅ Keycloak 로그인 성공!")
        print(f"Access Token (length): {len(access_token) if access_token else 0}")
        print(f"Token Type: {token_info.get('token_type')}")
        print(f"Expires In: {token_info.get('expires_in')} seconds")
        print()
        
        # 받은 토큰으로 /api/me/dicom/studies 테스트
        print("=" * 70)
        print("Keycloak 토큰으로 /api/me/dicom/studies 테스트")
        print("=" * 70)
        BASE_URL = "http://localhost:8080"
        headers = {"Authorization": f"Bearer {access_token}"}
        
        url = f"{BASE_URL}/api/me/dicom/studies?project_id=2"
        print(f"URL: {url}")
        
        resp2 = requests.get(url, headers=headers, timeout=10)
        print(f"Status: {resp2.status_code}")
        
        if resp2.status_code == 200:
            data = resp2.json()
            if isinstance(data, list):
                print(f"✅ Studies {len(data)}개 반환됨")
                if len(data) > 0:
                    print(f"\n첫 번째 Study:")
                    first_study = data[0]
                    study_uid = first_study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in first_study else "N/A"
                    patient_id = first_study.get("00100020", {}).get("Value", ["N/A"])[0] if "00100020" in first_study else "N/A"
                    print(f"  Study UID: {study_uid}")
                    print(f"  Patient ID: {patient_id}")
                else:
                    print("⚠️  Studies가 0개입니다!")
            else:
                print(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
        else:
            print(f"❌ 에러: {resp2.text[:500]}")
        
        print()
        
        # 비교: 일반 로그인 API로 받은 토큰
        print("=" * 70)
        print("비교: 일반 로그인 API로 받은 토큰")
        print("=" * 70)
        login_resp = requests.post(
            f"{BASE_URL}/api/auth/login",
            json={"username": USERNAME, "password": PASSWORD},
            timeout=5
        )
        
        if login_resp.status_code == 200:
            login_token = login_resp.json().get("token")
            print(f"일반 로그인 토큰 (length): {len(login_token) if login_token else 0}")
            
            headers2 = {"Authorization": f"Bearer {login_token}"}
            resp3 = requests.get(url, headers=headers2, timeout=10)
            print(f"Status: {resp3.status_code}")
            if resp3.status_code == 200:
                data3 = resp3.json()
                if isinstance(data3, list):
                    print(f"Studies {len(data3)}개 반환됨")
                else:
                    print(f"응답: {json.dumps(data3, indent=2, ensure_ascii=False)[:500]}")
            else:
                print(f"❌ 에러: {resp3.text[:500]}")
        
    else:
        print(f"❌ Keycloak 로그인 실패: {resp.status_code}")
        print(f"응답: {resp.text[:500]}")
        print()
        print("💡 다른 클라이언트 ID를 시도해볼 수 있습니다:")
        print("   - pacs-server")
        print("   - dcm4chee-arc")
        print("   - 다른 클라이언트 ID")

except Exception as e:
    print(f"❌ 에러 발생: {e}")

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)

