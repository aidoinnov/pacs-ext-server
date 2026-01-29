#!/usr/bin/env python3
"""
Study 개수 문제 확인 테스트
DB에는 8개인데 API에서 많이 나오는 문제 확인
"""

import requests

BASE_URL = "http://localhost:8080"

print("=" * 70)
print("Study 개수 문제 확인")
print("=" * 70)

# 로그인
print("\n🔐 로그인 중...")
login_resp = requests.post(
    f"{BASE_URL}/api/auth/login",
    json={"username": "reader1_user", "password": "Qlalfqjsgh1!"},
    timeout=5
)

if login_resp.status_code != 200:
    print(f"❌ 로그인 실패: {login_resp.status_code}")
    print(login_resp.text)
    exit(1)

token = login_resp.json()["token"]
headers = {"Authorization": f"Bearer {token}"}
print(f"✅ 로그인 성공")

print("\n1️⃣ check_assignment_for_project=2 로 조회")
print("-" * 70)

response = requests.get(
    f"{BASE_URL}/api/dicom/studies",
    params={
        "check_assignment_for_project": 2,
        "page": 1,
        "page_size": 100
    },
    headers=headers
)

if response.status_code == 200:
    studies = response.json()
    print(f"📊 반환된 Study 개수: {len(studies)}개")
    
    print(f"\n📋 Study 목록:")
    for i, study in enumerate(studies, 1):
        study_uid = study.get("0020000D", {}).get("Value", ["N/A"])[0]
        patient_id = study.get("00100020", {}).get("Value", ["N/A"])[0]
        study_date = study.get("00080020", {}).get("Value", ["N/A"])[0]
        is_assigned = study.get("is_assigned", "N/A")
        
        print(f"  {i:3d}. {study_uid[:50]:50s} | {patient_id:20s} | {study_date} | assigned={is_assigned}")
    
    # is_assigned 통계
    assigned_count = sum(1 for s in studies if s.get("is_assigned") == True)
    not_assigned_count = sum(1 for s in studies if s.get("is_assigned") == False)
    
    print(f"\n📊 is_assigned 통계:")
    print(f"  - assigned=true:  {assigned_count}개")
    print(f"  - assigned=false: {not_assigned_count}개")
    print(f"  - 합계: {assigned_count + not_assigned_count}개")
    
else:
    print(f"❌ 에러: {response.status_code}")
    print(response.text)

print("\n" + "=" * 70)
print("2️⃣ project_id=2 로 조회 (비교)")
print("-" * 70)

response2 = requests.get(
    f"{BASE_URL}/api/dicom/studies",
    params={
        "project_id": 2,
        "page": 1,
        "page_size": 100
    },
    headers=headers
)

if response2.status_code == 200:
    studies2 = response2.json()
    print(f"📊 반환된 Study 개수: {len(studies2)}개")
else:
    print(f"❌ 에러: {response2.status_code}")
    print(response2.text)

print("\n" + "=" * 70)
print("결론")
print("=" * 70)

print(f"DB에 저장된 Study: 8개")
print(f"check_assignment_for_project=2: {len(studies)}개")
print(f"project_id=2: {len(studies2)}개")

if len(studies) > 8:
    print(f"\n⚠️  문제 발견!")
    print(f"   check_assignment_for_project=2 가 {len(studies)}개를 반환했습니다.")
    print(f"   이 중 is_assigned=false 인 Study도 포함되어 있을 가능성이 있습니다.")
    print(f"\n🔍 확인 필요:")
    print(f"   - check_assignment_for_project 파라미터가 필터링이 아니라")
    print(f"     is_assigned 필드만 추가하는 것인지 확인")

