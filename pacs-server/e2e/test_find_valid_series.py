#!/usr/bin/env python3
"""
아카이브에서 실제로 존재하는 Study와 Series 찾기
"""

import requests
import json

# Keycloak에서 직접 토큰 받기
print("🔐 Keycloak 로그인 중...")
KEYCLOAK_URL = "https://keycloak.pacs.ai-do.co.kr"
REALM = "dcm4che"
CLIENT_ID = "pacs-extension-server"
CLIENT_SECRET = "vYMipExC4DCpesgWMy11FEOMWybxtpfq"
TOKEN_URL = f"{KEYCLOAK_URL}/realms/{REALM}/protocol/openid-connect/token"

token_data = {
    "grant_type": "password",
    "client_id": CLIENT_ID,
    "client_secret": CLIENT_SECRET,
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!",
}

kc_resp = requests.post(TOKEN_URL, data=token_data, timeout=10)
if kc_resp.status_code != 200:
    print(f"❌ Keycloak 로그인 실패: {kc_resp.status_code}")
    exit(1)

kc_token = kc_resp.json().get("access_token")
print(f"✅ Keycloak 로그인 성공\n")

DCM4CHEE_URL = "https://archive.pacs.ai-do.co.kr"
QIDO_PATH = "/iaid-pacs/aets/iAID_PACS/rs"

qido_headers = {
    "Authorization": f"Bearer {kc_token}",
    "Accept": "application/json"
}

# 1. Studies 조회
print("=" * 80)
print("1. Studies 조회 (limit=5)")
print("=" * 80)

studies_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies?limit=5"
studies_resp = requests.get(studies_url, headers=qido_headers, timeout=10)

if studies_resp.status_code != 200:
    print(f"❌ Studies 조회 실패: {studies_resp.status_code}")
    exit(1)

studies = studies_resp.json()
print(f"✅ {len(studies)}개 Study 조회 성공\n")

# 첫 번째 Study 선택
if len(studies) == 0:
    print("❌ Study가 없습니다")
    exit(1)

first_study = studies[0]
study_uid = first_study.get("0020000D", {}).get("Value", ["N/A"])[0]
patient_id = first_study.get("00100020", {}).get("Value", ["N/A"])[0]

print(f"첫 번째 Study:")
print(f"  - Study UID: {study_uid}")
print(f"  - Patient ID: {patient_id}")
print()

# 2. 해당 Study의 Series 조회
print("=" * 80)
print(f"2. Study의 Series 조회")
print("=" * 80)

series_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies/{study_uid}/series"
series_resp = requests.get(series_url, headers=qido_headers, timeout=10)

print(f"HTTP Status: {series_resp.status_code}")

if series_resp.status_code == 200:
    series_list = series_resp.json()
    print(f"✅ {len(series_list)}개 Series 조회 성공\n")
    
    if len(series_list) > 0:
        first_series = series_list[0]
        series_uid = first_series.get("0020000E", {}).get("Value", ["N/A"])[0]
        modality = first_series.get("00080060", {}).get("Value", ["N/A"])[0]
        
        print(f"첫 번째 Series:")
        print(f"  - Series UID: {series_uid}")
        print(f"  - Modality: {modality}")
        print()
        
        # 3. 해당 Series의 Instances 조회
        print("=" * 80)
        print(f"3. Series의 Instances 조회")
        print("=" * 80)
        
        instances_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies/{study_uid}/series/{series_uid}/instances"
        instances_resp = requests.get(instances_url, headers=qido_headers, timeout=10)
        
        print(f"HTTP Status: {instances_resp.status_code}")
        
        if instances_resp.status_code == 200:
            instances = instances_resp.json()
            print(f"✅ {len(instances)}개 Instance 조회 성공\n")
            
            if len(instances) > 0:
                first_instance = instances[0]
                sop_instance_uid = first_instance.get("00080018", {}).get("Value", ["N/A"])[0]
                instance_number = first_instance.get("00200013", {}).get("Value", ["N/A"])[0]
                
                print(f"첫 번째 Instance:")
                print(f"  - SOP Instance UID: {sop_instance_uid}")
                print(f"  - Instance Number: {instance_number}")
                print()
                
                print("=" * 80)
                print("✅ 성공! 사용 가능한 데이터:")
                print("=" * 80)
                print(f"Study UID:  {study_uid}")
                print(f"Series UID: {series_uid}")
                print(f"Instance 개수: {len(instances)}")
        elif instances_resp.status_code == 204:
            print("⚠️  Instance가 없습니다 (204 No Content)")
        else:
            print(f"❌ 실패: {instances_resp.text[:200]}")
    else:
        print("⚠️  Series가 없습니다")
elif series_resp.status_code == 204:
    print("⚠️  Series가 없습니다 (204 No Content)")
else:
    print(f"❌ 실패: {series_resp.text[:200]}")

