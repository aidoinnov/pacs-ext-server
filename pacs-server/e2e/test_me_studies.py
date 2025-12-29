#!/usr/bin/env python3
"""
/api/me/dicom/studies 엔드포인트 테스트
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
    print(login_resp.text)
    exit(1)

token = login_resp.json()["token"]
user_id = login_resp.json().get("user_id", "N/A")
headers = {"Authorization": f"Bearer {token}"}

print(f"✅ 로그인 성공 - user_id: {user_id}\n")

# /api/me/dicom/studies?project_id=2 호출
print("=" * 70)
print("테스트: /api/me/dicom/studies?project_id=2")
print("=" * 70)
url = f"{BASE_URL}/api/me/dicom/studies?project_id=2"
print(f"URL: {url}")

resp = requests.get(url, headers=headers, timeout=10)
print(f"Status: {resp.status_code}")

if resp.status_code == 200:
    data = resp.json()
    if isinstance(data, list):
        print(f"✅ Studies {len(data)}개 반환됨")
        if len(data) > 0:
            print(f"\n첫 번째 Study:")
            first_study = data[0]
            study_uid = first_study.get("0020000D", {}).get("Value", ["N/A"])[0] if "0020000D" in first_study else "N/A"
            patient_id = first_study.get("00100020", {}).get("Value", ["N/A"])[0] if "00100020" in first_study else "N/A"
            study_date = first_study.get("00080020", {}).get("Value", ["N/A"])[0] if "00080020" in first_study else "N/A"
            print(f"  Study UID: {study_uid}")
            print(f"  Patient ID: {patient_id}")
            print(f"  Study Date: {study_date}")
        else:
            print("⚠️  Studies가 0개입니다!")
    else:
        print(f"응답 타입: {type(data)}")
        print(f"응답 내용: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
else:
    print(f"❌ 에러 응답:")
    print(resp.text[:500])

print()

# 비교: /api/dicom/studies?project_id=2 호출
print("=" * 70)
print("비교: /api/dicom/studies?project_id=2")
print("=" * 70)
url2 = f"{BASE_URL}/api/dicom/studies?project_id=2"
print(f"URL: {url2}")

resp2 = requests.get(url2, headers=headers, timeout=10)
print(f"Status: {resp2.status_code}")

if resp2.status_code == 200:
    data2 = resp2.json()
    if isinstance(data2, list):
        print(f"✅ Studies {len(data2)}개 반환됨")
    else:
        print(f"응답 타입: {type(data2)}")
        print(f"응답 내용: {json.dumps(data2, indent=2, ensure_ascii=False)[:500]}")
else:
    print(f"❌ 에러 응답:")
    print(resp2.text[:500])

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)



