#!/usr/bin/env python3
"""
두 엔드포인트 비교 분석
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

# 1. /api/me/dicom/studies
print("=" * 70)
print("1. /api/me/dicom/studies?project_id=2")
print("=" * 70)
url1 = f"{BASE_URL}/api/me/dicom/studies?project_id=2"
resp1 = requests.get(url1, headers=headers, timeout=10)
data1 = resp1.json() if resp1.status_code == 200 else []
print(f"Status: {resp1.status_code}")
print(f"Studies count: {len(data1) if isinstance(data1, list) else 0}")
if isinstance(data1, list) and len(data1) > 0:
    study_uids_1 = set()
    for study in data1:
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        study_uids_1.add(uid)
    print(f"Unique Study UIDs: {len(study_uids_1)}")
    print(f"First 3 Study UIDs: {list(study_uids_1)[:3]}")

print()

# 2. /api/dicom/studies
print("=" * 70)
print("2. /api/dicom/studies?project_id=2")
print("=" * 70)
url2 = f"{BASE_URL}/api/dicom/studies?project_id=2"
resp2 = requests.get(url2, headers=headers, timeout=10)
data2 = resp2.json() if resp2.status_code == 200 else []
print(f"Status: {resp2.status_code}")
print(f"Studies count: {len(data2) if isinstance(data2, list) else 0}")
if isinstance(data2, list) and len(data2) > 0:
    study_uids_2 = set()
    for study in data2:
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        study_uids_2.add(uid)
    print(f"Unique Study UIDs: {len(study_uids_2)}")
    print(f"First 3 Study UIDs: {list(study_uids_2)[:3]}")

print()

# 비교
print("=" * 70)
print("비교 분석")
print("=" * 70)
if isinstance(data1, list) and isinstance(data2, list):
    study_uids_1 = set()
    for study in data1:
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        if uid != "N/A":
            study_uids_1.add(uid)
    
    study_uids_2 = set()
    for study in data2:
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        if uid != "N/A":
            study_uids_2.add(uid)
    
    print(f"/api/me/dicom/studies: {len(study_uids_1)} unique studies")
    print(f"/api/dicom/studies: {len(study_uids_2)} unique studies")
    print()
    
    only_in_me = study_uids_1 - study_uids_2
    only_in_dicom = study_uids_2 - study_uids_1
    common = study_uids_1 & study_uids_2
    
    print(f"공통 Study UIDs: {len(common)}")
    print(f"/api/me/dicom/studies에만 있는 Study UIDs: {len(only_in_me)}")
    print(f"/api/dicom/studies에만 있는 Study UIDs: {len(only_in_dicom)}")
    
    if only_in_me:
        print(f"\n/me에만 있는 Study UIDs (처음 5개): {list(only_in_me)[:5]}")
    if only_in_dicom:
        print(f"\n/dicom에만 있는 Study UIDs (처음 5개): {list(only_in_dicom)[:5]}")

print()
print("=" * 70)
print("차이점 요약")
print("=" * 70)
print("""
1. /api/me/dicom/studies:
   - 프로젝트별로 QIDO 호출 (각 프로젝트의 Access Condition 적용)
   - 각 Study에 대해 RBAC 평가 + project_data_access 확인
   - 중복 제거 (study_uids_seen)
   - 페이지네이션 적용 (기본 page_size=50)
   - Study Date로 정렬

2. /api/dicom/studies:
   - 단일 QIDO 호출
   - project_id가 있으면 Access Condition 적용
   - RBAC 필터링 + project_data_access 확인
   - 페이지네이션 없음 (전체 반환)
   - 중복 제거 없음
""")





