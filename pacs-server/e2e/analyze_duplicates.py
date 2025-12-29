#!/usr/bin/env python3
"""
중복 Study 분석
"""

import requests
import json
from collections import Counter

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

print(f"✅ 로그인 성공\n")

# /api/dicom/studies 분석
print("=" * 70)
print("/api/dicom/studies?project_id=2 중복 분석")
print("=" * 70)
url = f"{BASE_URL}/api/dicom/studies?project_id=2"
resp = requests.get(url, headers=headers, timeout=10)
data = resp.json() if resp.status_code == 200 else []

if isinstance(data, list):
    study_uids = []
    for study in data:
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        if uid != "N/A":
            study_uids.append(uid)
    
    print(f"Total studies: {len(data)}")
    print(f"Unique Study UIDs: {len(set(study_uids))}")
    print()
    
    # 중복 카운트
    uid_counts = Counter(study_uids)
    duplicates = {uid: count for uid, count in uid_counts.items() if count > 1}
    
    if duplicates:
        print(f"중복된 Study UIDs ({len(duplicates)}개):")
        for uid, count in duplicates.items():
            print(f"  {uid}: {count}번 반복")
    else:
        print("중복 없음 - 모든 Study UID가 고유함")
        print()
        print("그렇다면 21개와 6개의 차이는 다른 이유일 수 있습니다:")
        print("1. /api/dicom/studies는 페이지네이션 없이 전체 반환")
        print("2. /api/me/dicom/studies는 기본 page_size=50이지만 실제로는 6개만 반환")
        print("3. QIDO 호출 결과가 다를 수 있음 (Access Condition 차이)")

print()
print("=" * 70)
print("각 Study UID별 상세 정보")
print("=" * 70)
if isinstance(data, list):
    for i, study in enumerate(data[:10], 1):  # 처음 10개만
        uid = study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in study else "N/A"
        patient_id = study.get("00100020", {}).get("Value", ["N/A"])[0] if "00100020" in study else "N/A"
        study_date = study.get("00080020", {}).get("Value", ["N/A"])[0] if "00080020" in study else "N/A"
        print(f"{i}. Study UID: {uid}")
        print(f"   Patient ID: {patient_id}")
        print(f"   Study Date: {study_date}")
        print()



