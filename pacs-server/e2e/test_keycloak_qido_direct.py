#!/usr/bin/env python3
"""
Keycloak 토큰으로 Dcm4chee QIDO에 직접 요청 테스트
"""

import requests
import json

# Keycloak 설정
KEYCLOAK_URL = "https://keycloak.pacs.ai-do.co.kr"
REALM = "dcm4che"
CLIENT_ID = "pacs-extension-server"
CLIENT_SECRET = "vYMipExC4DCpesgWMy11FEOMWybxtpfq"

# Dcm4chee 설정 (config/development.toml에서 확인)
DCM4CHEE_URL = "https://archive.pacs.ai-do.co.kr"
QIDO_PATH = "/iaid-pacs/aets/iAID_PACS/rs"

# 사용자 정보
USERNAME = "iaid-pacs-admin"
PASSWORD = "Qlalfqjsgh1!"

# Keycloak 토큰 엔드포인트
TOKEN_URL = f"{KEYCLOAK_URL}/realms/{REALM}/protocol/openid-connect/token"

print("=" * 70)
print("Keycloak 토큰으로 Dcm4chee QIDO 직접 테스트")
print("=" * 70)

# 1. Keycloak에서 토큰 받기
print("🔐 Keycloak에서 토큰 받는 중...")
token_data = {
    "grant_type": "password",
    "client_id": CLIENT_ID,
    "client_secret": CLIENT_SECRET,
    "username": USERNAME,
    "password": PASSWORD,
}

try:
    resp = requests.post(TOKEN_URL, data=token_data, timeout=10)
    if resp.status_code != 200:
        print(f"❌ Keycloak 로그인 실패: {resp.status_code}")
        print(resp.text)
        exit(1)
    
    token_info = resp.json()
    access_token = token_info.get("access_token")
    print(f"✅ Keycloak 토큰 받기 성공 (length: {len(access_token)})")
    print()
    
    # 2. Dcm4chee QIDO /studies 직접 호출
    print("=" * 70)
    print("Dcm4chee QIDO /studies 직접 호출")
    print("=" * 70)
    
    qido_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies"
    print(f"QIDO URL: {qido_url}")
    print(f"Bearer Token (length): {len(access_token)}")
    print()
    
    headers = {
        "Authorization": f"Bearer {access_token}",
        "Accept": "application/json"
    }
    
    # limit 파라미터 추가
    params = {
        "limit": "10"
    }
    
    print("🚀 QIDO 요청 전송 중...")
    qido_resp = requests.get(qido_url, headers=headers, params=params, timeout=10)
    
    print(f"Status: {qido_resp.status_code}")
    print(f"Response Headers: {dict(qido_resp.headers)}")
    print()
    
    if qido_resp.status_code == 200:
        try:
            data = qido_resp.json()
            if isinstance(data, list):
                print(f"✅ Studies {len(data)}개 반환됨!")
                if len(data) > 0:
                    print(f"\n첫 번째 Study:")
                    first_study = data[0]
                    print(json.dumps(first_study, indent=2, ensure_ascii=False)[:500])
            else:
                print(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
        except:
            print(f"응답 (텍스트): {qido_resp.text[:500]}")
    else:
        print(f"❌ QIDO 요청 실패")
        print(f"응답: {qido_resp.text[:500]}")
    
    print()
    
    # 3. Gateway API와 비교
    print("=" * 70)
    print("비교: Gateway API (/api/me/dicom/studies)")
    print("=" * 70)
    
    BASE_URL = "http://localhost:8080"
    gateway_url = f"{BASE_URL}/api/me/dicom/studies?project_id=2"
    gateway_headers = {"Authorization": f"Bearer {access_token}"}
    
    print(f"Gateway URL: {gateway_url}")
    print(f"Bearer Token (length): {len(access_token)}")
    print()
    
    gateway_resp = requests.get(gateway_url, headers=gateway_headers, timeout=10)
    print(f"Status: {gateway_resp.status_code}")
    
    if gateway_resp.status_code == 200:
        gateway_data = gateway_resp.json()
        if isinstance(gateway_data, list):
            print(f"Studies {len(gateway_data)}개 반환됨")
        else:
            print(f"응답: {json.dumps(gateway_data, indent=2, ensure_ascii=False)[:500]}")
    else:
        print(f"❌ 에러: {gateway_resp.text[:500]}")
    
except Exception as e:
    print(f"❌ 에러 발생: {e}")
    import traceback
    traceback.print_exc()

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)

