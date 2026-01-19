#!/usr/bin/env python3
"""
실제 데이터가 있는지 확인하고 성능 테스트
"""

import requests
import time
import json

BASE_URL = "http://localhost:8080"

# user_id=1로 로그인 (iaid-pacs-admin)
print("🔐 user_id=1로 로그인 중...")
login_resp = requests.post(
    f"{BASE_URL}/api/auth/login",
    json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
    timeout=5
)

if login_resp.status_code != 200:
    print(f"❌ 로그인 실패: {login_resp.status_code}")
    print(login_resp.text)
    exit(1)

login_data = login_resp.json()
token = login_data.get("token")
user_id = login_data.get("user_id")
headers = {"Authorization": f"Bearer {token}"}

print(f"✅ 로그인 성공 - user_id: {user_id}")
print(f"   username: {login_data.get('username')}")
print()

# 먼저 /api/users/me로 사용자 정보 확인
print("=" * 70)
print("사용자 정보 확인")
print("=" * 70)
me_resp = requests.get(f"{BASE_URL}/api/users/me", headers=headers, timeout=5)
if me_resp.status_code == 200:
    me_data = me_resp.json()
    print(f"현재 로그인한 사용자 ID: {me_data.get('id')}")
    print(f"사용자명: {me_data.get('username')}")
else:
    print(f"❌ 사용자 정보 조회 실패: {me_resp.status_code}")

print()

# Series API 테스트 (user_id 쿼리 파라미터 제거 - JWT에서 추출됨)
url = f"{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=200"

print("=" * 70)
print("Series API 테스트")
print("=" * 70)
print(f"URL: {url}")
print(f"Headers: Authorization: Bearer {token[:20]}...")
print()

start = time.time()
try:
    response = requests.get(url, headers=headers, timeout=60)
    elapsed = time.time() - start
    
    print(f"Status: {response.status_code}")
    print(f"응답 시간: {elapsed:.3f}초 ({elapsed*1000:.0f}ms)")
    
    if response.status_code == 200:
        data = response.json()
        if isinstance(data, list):
            count = len(data)
            print(f"반환된 Series 수: {count}")
            
            if count > 0:
                print(f"\n첫 번째 Series 정보:")
                first = data[0]
                series_uid = first.get("0020000E", {}).get("Value", ["N/A"])[0] if "0020000E" in first else "N/A"
                study_uid = first.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in first else "N/A"
                modality = first.get("00080060", {}).get("Value", ["N/A"])[0] if "00080060" in first else "N/A"
                print(f"  Series UID: {series_uid}")
                print(f"  Study UID: {study_uid}")
                print(f"  Modality: {modality}")
            else:
                print("\n⚠️  Series가 0개입니다!")
                print("   가능한 원인:")
                print("   1. project_id=2에 데이터가 없음")
                print("   2. 사용자가 project_id=2의 멤버가 아님")
                print("   3. RBAC 필터링으로 모든 Series가 제외됨")
        else:
            print(f"응답 타입: {type(data)}")
            print(f"응답 내용 (처음 500자):")
            print(json.dumps(data, indent=2, ensure_ascii=False)[:500])
    else:
        print(f"❌ 에러 응답:")
        print(response.text[:500])
        
except Exception as e:
    elapsed = time.time() - start
    print(f"❌ 에러 발생: {e}")
    print(f"실패까지 걸린 시간: {elapsed:.3f}초")

print()

# project_id=2의 멤버인지 확인
print("=" * 70)
print("프로젝트 멤버십 확인")
print("=" * 70)
projects_resp = requests.get(f"{BASE_URL}/api/users/me/projects", headers=headers, timeout=5)
if projects_resp.status_code == 200:
    projects = projects_resp.json()
    if isinstance(projects, list):
        project_ids = [p.get("id") for p in projects if p.get("id")]
        print(f"사용자가 속한 프로젝트 ID: {project_ids}")
        if 2 in project_ids:
            print("✅ project_id=2의 멤버입니다")
        else:
            print("❌ project_id=2의 멤버가 아닙니다!")
    else:
        print(f"응답 타입: {type(projects)}")
        print(projects)
else:
    print(f"프로젝트 목록 조회 실패: {projects_resp.status_code}")





