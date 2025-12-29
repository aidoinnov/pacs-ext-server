#!/usr/bin/env python3
"""
includefield 파라미터로 Study Description 태그 포함 테스트
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
headers = {"Authorization": f"Bearer {token}"}

print("✅ 로그인 성공\n")

# 테스트 1: includefield 없이 호출
print("=" * 70)
print("테스트 1: includefield 없이 호출")
print("=" * 70)
url1 = f"{BASE_URL}/api/dicom/series?project_id=2&page=1&page_size=5"
print(f"URL: {url1}")

resp1 = requests.get(url1, headers=headers, timeout=10)
print(f"Status: {resp1.status_code}")

if resp1.status_code == 200:
    data1 = resp1.json()
    if isinstance(data1, list) and len(data1) > 0:
        first_series = data1[0]
        print(f"첫 번째 Series의 태그 키: {list(first_series.keys())[:10]}...")
        has_study_desc = "00081030" in first_series
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        if has_study_desc:
            print(f"Study Description 값: {first_series.get('00081030')}")
    else:
        print("⚠️  Series가 없습니다")
else:
    print(f"❌ 에러: {resp1.text[:200]}")

print()

# 테스트 2: includefield=00081030 포함하여 호출
print("=" * 70)
print("테스트 2: includefield=00081030 포함하여 호출")
print("=" * 70)
url2 = f"{BASE_URL}/api/dicom/series?project_id=2&page=1&page_size=5&includefield=00081030"
print(f"URL: {url2}")

resp2 = requests.get(url2, headers=headers, timeout=10)
print(f"Status: {resp2.status_code}")

if resp2.status_code == 200:
    data2 = resp2.json()
    if isinstance(data2, list) and len(data2) > 0:
        first_series = data2[0]
        print(f"첫 번째 Series의 태그 키: {list(first_series.keys())[:10]}...")
        has_study_desc = "00081030" in first_series
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        if has_study_desc:
            study_desc = first_series.get("00081030")
            print(f"Study Description 값: {json.dumps(study_desc, indent=2, ensure_ascii=False)}")
        else:
            print("⚠️  Study Description 태그가 포함되지 않았습니다")
    else:
        print("⚠️  Series가 없습니다")
else:
    print(f"❌ 에러: {resp2.text[:200]}")

print()

# 테스트 3: 여러 includefield 포함
print("=" * 70)
print("테스트 3: 여러 includefield 포함 (00081030, 0020000D)")
print("=" * 70)
url3 = f"{BASE_URL}/api/dicom/series?project_id=2&page=1&page_size=5&includefield=00081030&includefield=0020000D"
print(f"URL: {url3}")

resp3 = requests.get(url3, headers=headers, timeout=10)
print(f"Status: {resp3.status_code}")

if resp3.status_code == 200:
    data3 = resp3.json()
    if isinstance(data3, list) and len(data3) > 0:
        first_series = data3[0]
        print(f"첫 번째 Series의 태그 키: {list(first_series.keys())[:10]}...")
        has_study_desc = "00081030" in first_series
        has_study_uid = "0020000D" in first_series
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        print(f"Study UID 태그 (0020000D) 포함 여부: {has_study_uid}")
        if has_study_desc:
            study_desc = first_series.get("00081030")
            print(f"Study Description 값: {json.dumps(study_desc, indent=2, ensure_ascii=False)}")
    else:
        print("⚠️  Series가 없습니다")
else:
    print(f"❌ 에러: {resp3.text[:200]}")

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)



