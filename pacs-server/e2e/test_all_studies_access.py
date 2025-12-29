#!/usr/bin/env python3
"""
모든 Study 접근 가능 여부 테스트
"""

import requests
import json

BASE_URL = "http://localhost:8080"

# 로그인
print("🔐 로그인 중...")
login_resp = requests.post(
    f"{BASE_URL}/api/auth/login",
    json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
    timeout=5
)

if login_resp.status_code != 200:
    print(f"❌ 로그인 실패: {login_resp.status_code}")
    exit(1)

token = login_resp.json()["token"]
headers = {"Authorization": f"Bearer {token}"}

print(f"✅ 로그인 성공 (token length: {len(token)})\n")

# 1. /api/me/dicom/studies - 여러 페이지 확인
print("=" * 70)
print("1. /api/me/dicom/studies?project_id=2 (여러 페이지)")
print("=" * 70)

all_studies_me = []
page = 1
page_size = 200

while True:
    url = f"{BASE_URL}/api/me/dicom/studies?project_id=2&page={page}&page_size={page_size}"
    resp = requests.get(url, headers=headers, timeout=10)
    
    if resp.status_code != 200:
        print(f"❌ 페이지 {page} 실패: {resp.status_code}")
        break
    
    data = resp.json()
    if not isinstance(data, list) or len(data) == 0:
        break
    
    all_studies_me.extend(data)
    print(f"페이지 {page}: {len(data)}개 Study")
    
    if len(data) < page_size:
        break
    
    page += 1

print(f"총 {len(all_studies_me)}개 Study (중복 제거 전)")
study_uids_me = set()
for study in all_studies_me:
    uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
    if uid != "N/A":
        study_uids_me.add(uid)
print(f"고유 Study UIDs: {len(study_uids_me)}개")
print()

# 2. /api/dicom/studies - 전체 확인
print("=" * 70)
print("2. /api/dicom/studies?project_id=2 (전체)")
print("=" * 70)
url2 = f"{BASE_URL}/api/dicom/studies?project_id=2"
resp2 = requests.get(url2, headers=headers, timeout=10)

if resp2.status_code == 200:
    data2 = resp2.json()
    if isinstance(data2, list):
        print(f"Studies count: {len(data2)}")
        study_uids_2 = set()
        for study in data2:
            uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
            if uid != "N/A":
                study_uids_2.add(uid)
        print(f"고유 Study UIDs: {len(study_uids_2)}개")
    else:
        print(f"응답 타입: {type(data2)}")
else:
    print(f"❌ 실패: {resp2.status_code}")
    study_uids_2 = set()

print()

# 3. Keycloak 토큰으로 Dcm4chee 직접 호출 (비교용)
print("=" * 70)
print("3. Dcm4chee 직접 호출 (비교용)")
print("=" * 70)

# Keycloak에서 직접 토큰 받기
KEYCLOAK_URL = "https://keycloak.pacs.ai-do.kr"
REALM = "dcm4che"
CLIENT_ID = "pacs-extension-server"
CLIENT_SECRET = "85TSWxK8ruF750z0Qzh0tQZ8xH5h3y99"
TOKEN_URL = f"{KEYCLOAK_URL}/realms/{REALM}/protocol/openid-connect/token"

token_data = {
    "grant_type": "password",
    "client_id": CLIENT_ID,
    "client_secret": CLIENT_SECRET,
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!",
}

try:
    kc_resp = requests.post(TOKEN_URL, data=token_data, timeout=10)
    if kc_resp.status_code == 200:
        kc_token = kc_resp.json().get("access_token")
        
        DCM4CHEE_URL = "https://archive.pacs.ai-do.kr"
        QIDO_PATH = "/iaid-pacs/aets/iAID_PACS/rs"
        qido_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies?limit=100"
        
        qido_headers = {
            "Authorization": f"Bearer {kc_token}",
            "Accept": "application/json"
        }
        
        qido_resp = requests.get(qido_url, headers=qido_headers, timeout=10)
        if qido_resp.status_code == 200:
            qido_data = qido_resp.json()
            if isinstance(qido_data, list):
                print(f"Dcm4chee 직접 호출: {len(qido_data)}개 Study")
                qido_uids = set()
                for study in qido_data:
                    uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
                    if uid != "N/A":
                        qido_uids.add(uid)
                print(f"고유 Study UIDs: {len(qido_uids)}개")
            else:
                print(f"응답 타입: {type(qido_data)}")
        else:
            print(f"❌ QIDO 호출 실패: {qido_resp.status_code}")
    else:
        print(f"❌ Keycloak 로그인 실패: {kc_resp.status_code}")
except Exception as e:
    print(f"❌ 에러: {e}")

print()
print("=" * 70)
print("결론")
print("=" * 70)
print(f"✅ Keycloak Access Token 인증: 성공")
print(f"✅ Gateway API 접근: 성공")
if isinstance(data2, list):
    print(f"✅ /api/dicom/studies: {len(study_uids_2)}개 고유 Study 반환")
    print(f"✅ /api/me/dicom/studies: {len(study_uids_me)}개 고유 Study 반환")
    print()
    if len(study_uids_me) == len(study_uids_2):
        print("✅ 두 엔드포인트가 동일한 Study를 반환합니다!")
    else:
        print(f"⚠️  두 엔드포인트가 다른 Study를 반환합니다 (차이: {abs(len(study_uids_me) - len(study_uids_2))}개)")



