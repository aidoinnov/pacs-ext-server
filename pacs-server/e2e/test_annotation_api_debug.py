#!/usr/bin/env python3
"""
Annotation API Debug Test
사용자가 보고한 어노테이션 API 오류 디버깅
"""

import requests
import json

BASE_URL = "http://localhost:8080"
USER_ID = 'iaid-pacs-admin'
USER_PASSWORD = 'Qlalfqjsgh1!'

def login():
    """로그인하여 JWT 토큰 획득"""
    print("🔐 로그인 중...")
    login_resp = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": USER_ID, "password": USER_PASSWORD},
        timeout=5
    )

    if login_resp.status_code != 200:
        print(f"❌ 로그인 실패: {login_resp.status_code}")
        print(login_resp.text)
        exit(1)

    token = login_resp.json()["token"]
    print(f"✅ 로그인 성공 (token length: {len(token)})\n")
    return token

def test_annotation_apis(token: str):
    """어노테이션 API 테스트"""
    headers = {"Authorization": f"Bearer {token}"}
    
    print("=" * 70)
    print("📋 Annotation API Debug Test")
    print("=" * 70)
    
    # 1. 어노테이션 생성 (테스트용)
    print("\n1️⃣  테스트용 어노테이션 생성 중...")
    annotation_data = {
        "project_id": 2,
        "study_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661",
        "series_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953",
        "sop_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.217230834888240455035945707219",
        "annotation_data": {
            "type": "test",
            "x": 100,
            "y": 100,
        },
        "tool_name": "Test Tool",
        "tool_version": "1.0.0",
        "viewer_software": "TI-DicomViewer",
        "description": "디버그 테스트용 어노테이션",
    }
    
    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )
    
    print(f"   Status: {response.status_code}")
    if response.status_code == 201:
        annotation = response.json()
        annotation_id = annotation["id"]
        print(f"   ✅ 어노테이션 생성 성공! ID: {annotation_id}")
    else:
        print(f"   ❌ 어노테이션 생성 실패")
        print(f"   Response: {response.text}")
        return
    
    # 2. 테스트 1: DELETE /api/annotations/{id}?user_id=1
    print(f"\n2️⃣  테스트 1: DELETE /api/annotations/{annotation_id}?user_id=1")
    url = f"{BASE_URL}/api/annotations/{annotation_id}?user_id=1"
    print(f"   URL: {url}")
    
    response = requests.delete(url, headers=headers, timeout=10)
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.text[:500]}")
    
    if response.status_code == 200:
        print(f"   ✅ 삭제 성공")
    else:
        print(f"   ❌ 삭제 실패")
    
    # 3. 테스트 2: GET /api/annotations?series_instance_uid=...&user_id=1
    print(f"\n3️⃣  테스트 2: GET /api/annotations?series_instance_uid=...&user_id=1")
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    url = f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}&user_id=1"
    print(f"   URL: {url}")
    
    response = requests.get(url, headers=headers, timeout=10)
    print(f"   Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"   ✅ 조회 성공")
        print(f"   - 어노테이션 개수: {len(data.get('annotations', []))}")
        print(f"   - Response: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
    else:
        print(f"   ❌ 조회 실패")
        print(f"   Response: {response.text[:500]}")
    
    # 4. 테스트 3: GET /api/annotations?sop_instance_uid=...&user_id=1&project_id=2&viewer_software=TI-DicomViewer
    print(f"\n4️⃣  테스트 3: GET /api/annotations?sop_instance_uid=...&user_id=1&project_id=2&viewer_software=TI-DicomViewer")
    sop_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.238273576775187812804817387920"
    url = f"{BASE_URL}/api/annotations?sop_instance_uid={sop_uid}&user_id=1&project_id=2&viewer_software=TI-DicomViewer"
    print(f"   URL: {url}")
    
    response = requests.get(url, headers=headers, timeout=10)
    print(f"   Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"   ✅ 조회 성공")
        print(f"   - 어노테이션 개수: {len(data.get('annotations', []))}")
        print(f"   - Response: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
    else:
        print(f"   ❌ 조회 실패")
        print(f"   Response: {response.text[:500]}")
    
    print("\n" + "=" * 70)
    print("테스트 완료")
    print("=" * 70)

if __name__ == '__main__':
    try:
        print("\n🚀 Annotation API Debug Test 시작...\n")
        token = login()
        test_annotation_apis(token)
        print("\n✅ 테스트 완료!\n")
    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        exit(1)

