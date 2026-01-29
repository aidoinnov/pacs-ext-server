#!/usr/bin/env python3
"""
아카이브에 직접 Instances 엔드포인트 호출 테스트
"""

import requests
import json

BASE_URL = "http://localhost:8080"

# Study UID와 Series UID
STUDY_UID = "1.2.410.200022.500.202205101053010.12252192375"
SERIES_UID = "1.3.12.2.1107.5.1.4.66256.30000022061222050008400009163"

print("=" * 80)
print("아카이브 Instances 엔드포인트 직접 호출 테스트")
print("=" * 80)
print(f"Study UID:  {STUDY_UID}")
print(f"Series UID: {SERIES_UID}")
print()

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

try:
    kc_resp = requests.post(TOKEN_URL, data=token_data, timeout=10)
    if kc_resp.status_code != 200:
        print(f"❌ Keycloak 로그인 실패: {kc_resp.status_code}")
        print(kc_resp.text)
        exit(1)
    
    kc_token = kc_resp.json().get("access_token")
    print(f"✅ Keycloak 로그인 성공 (token length: {len(kc_token)})")
    print()
    
    # 아카이브 직접 호출
    DCM4CHEE_URL = "https://archive.pacs.ai-do.co.kr"
    QIDO_PATH = "/iaid-pacs/aets/iAID_PACS/rs"
    
    # 1. Instances 엔드포인트 호출
    print("=" * 80)
    print("1. 아카이브 Instances 엔드포인트 호출")
    print("=" * 80)
    
    instances_url = f"{DCM4CHEE_URL}{QIDO_PATH}/studies/{STUDY_UID}/series/{SERIES_UID}/instances"
    print(f"URL: {instances_url}")
    print()
    
    qido_headers = {
        "Authorization": f"Bearer {kc_token}",
        "Accept": "application/json"
    }
    
    instances_resp = requests.get(instances_url, headers=qido_headers, timeout=10)
    
    print(f"HTTP Status: {instances_resp.status_code}")
    
    if instances_resp.status_code == 200:
        instances_data = instances_resp.json()
        
        if isinstance(instances_data, list):
            print(f"✅ 성공! Instance 개수: {len(instances_data)}")
            print()
            
            # 첫 번째 Instance 정보 출력
            if len(instances_data) > 0:
                print("첫 번째 Instance 정보:")
                first_instance = instances_data[0]
                
                # SOP Instance UID (00080018)
                sop_instance_uid = first_instance.get("00080018", {}).get("Value", ["N/A"])[0]
                print(f"  - SOP Instance UID (00080018): {sop_instance_uid}")
                
                # Instance Number (00200013)
                instance_number = first_instance.get("00200013", {}).get("Value", ["N/A"])[0]
                print(f"  - Instance Number (00200013): {instance_number}")
                
                # SOP Class UID (00080016)
                sop_class_uid = first_instance.get("00080016", {}).get("Value", ["N/A"])[0]
                print(f"  - SOP Class UID (00080016): {sop_class_uid}")
                
                print()
                print("전체 태그 목록:")
                for tag in sorted(first_instance.keys()):
                    vr = first_instance[tag].get("vr", "N/A")
                    value = first_instance[tag].get("Value", ["N/A"])
                    print(f"  - {tag} ({vr}): {value}")
        else:
            print(f"⚠️  응답 타입이 리스트가 아닙니다: {type(instances_data)}")
            print(f"응답: {instances_data}")
    else:
        print(f"❌ 실패!")
        print(f"응답: {instances_resp.text[:500]}")
    
    print()
    
    # 2. includefield 파라미터 사용
    print("=" * 80)
    print("2. includefield 파라미터 사용")
    print("=" * 80)
    
    instances_url_with_params = (
        f"{DCM4CHEE_URL}{QIDO_PATH}/studies/{STUDY_UID}/series/{SERIES_UID}/instances"
        f"?includefield=00080018&includefield=00200013&limit=1000"
    )
    print(f"URL: {instances_url_with_params}")
    print()
    
    instances_resp2 = requests.get(instances_url_with_params, headers=qido_headers, timeout=10)
    
    print(f"HTTP Status: {instances_resp2.status_code}")
    
    if instances_resp2.status_code == 200:
        instances_data2 = instances_resp2.json()
        
        if isinstance(instances_data2, list):
            print(f"✅ 성공! Instance 개수: {len(instances_data2)}")
            print()
            
            if len(instances_data2) > 0:
                print("첫 번째 Instance (includefield 적용):")
                first_instance2 = instances_data2[0]
                
                sop_instance_uid = first_instance2.get("00080018", {}).get("Value", ["N/A"])[0]
                instance_number = first_instance2.get("00200013", {}).get("Value", ["N/A"])[0]
                
                print(f"  - SOP Instance UID (00080018): {sop_instance_uid}")
                print(f"  - Instance Number (00200013): {instance_number}")
                print()
                print(f"  - 전체 태그 개수: {len(first_instance2)}")
        else:
            print(f"⚠️  응답 타입이 리스트가 아닙니다: {type(instances_data2)}")
            print(f"응답: {instances_data2}")
    else:
        print(f"❌ 실패!")
        print(f"응답: {instances_resp2.text[:500]}")
    
    print()
    print("=" * 80)
    print("결론")
    print("=" * 80)
    print(f"✅ Keycloak Access Token 인증: 성공")
    print(f"✅ 아카이브 Instances 엔드포인트 호출: {'성공' if instances_resp.status_code == 200 else '실패'}")
    if instances_resp.status_code == 200 and isinstance(instances_data, list):
        print(f"✅ Instance 개수: {len(instances_data)}개")

except Exception as e:
    print(f"❌ 에러 발생: {e}")
    import traceback
    traceback.print_exc()

