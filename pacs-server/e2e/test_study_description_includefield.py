#!/usr/bin/env python3
"""
Study Description 태그 (00081030) includefield 테스트
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

# 테스트 1: includefield 없이 호출
print("=" * 70)
print("테스트 1: includefield 없이 호출")
print("=" * 70)
url1 = f"{BASE_URL}/api/dicom/studies?project_id=2"
print(f"URL: {url1}")

resp1 = requests.get(url1, headers=headers, timeout=10)
print(f"Status: {resp1.status_code}")

if resp1.status_code == 200:
    data1 = resp1.json()
    if isinstance(data1, list) and len(data1) > 0:
        first_study = data1[0]
        print(f"첫 번째 Study의 태그 키: {sorted(list(first_study.keys()))[:15]}")
        has_study_desc = "00081030" in first_study
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        if has_study_desc:
            print(f"Study Description 값: {json.dumps(first_study.get('00081030'), indent=2, ensure_ascii=False)}")
    else:
        print("⚠️  Studies가 없습니다")
else:
    print(f"❌ 에러: {resp1.text[:200]}")

print()

# 테스트 2: includefield=00081030 포함하여 호출
print("=" * 70)
print("테스트 2: includefield=00081030 포함하여 호출")
print("=" * 70)
url2 = f"{BASE_URL}/api/dicom/studies?project_id=2&includefield=00081030"
print(f"URL: {url2}")

resp2 = requests.get(url2, headers=headers, timeout=10)
print(f"Status: {resp2.status_code}")

if resp2.status_code == 200:
    data2 = resp2.json()
    if isinstance(data2, list) and len(data2) > 0:
        first_study = data2[0]
        print(f"첫 번째 Study의 태그 키: {sorted(list(first_study.keys()))[:15]}")
        has_study_desc = "00081030" in first_study
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        if has_study_desc:
            study_desc = first_study.get("00081030")
            print(f"Study Description 값:")
            print(json.dumps(study_desc, indent=2, ensure_ascii=False))
        else:
            print("⚠️  Study Description 태그가 포함되지 않았습니다")
    else:
        print("⚠️  Studies가 없습니다")
else:
    print(f"❌ 에러: {resp2.text[:200]}")

print()

# 테스트 3: /api/me/dicom/studies에서 includefield 테스트
print("=" * 70)
print("테스트 3: /api/me/dicom/studies?project_id=2&includefield=00081030")
print("=" * 70)
url3 = f"{BASE_URL}/api/me/dicom/studies?project_id=2&includefield=00081030"
print(f"URL: {url3}")

resp3 = requests.get(url3, headers=headers, timeout=10)
print(f"Status: {resp3.status_code}")

if resp3.status_code == 200:
    data3 = resp3.json()
    if isinstance(data3, list) and len(data3) > 0:
        print(f"Studies {len(data3)}개 반환됨")
        first_study = data3[0]
        print(f"첫 번째 Study의 태그 키: {sorted(list(first_study.keys()))[:15]}")
        has_study_desc = "00081030" in first_study
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        if has_study_desc:
            study_desc = first_study.get("00081030")
            print(f"Study Description 값:")
            print(json.dumps(study_desc, indent=2, ensure_ascii=False))
        
        # 모든 Study에 Study Description이 있는지 확인
        studies_with_desc = sum(1 for s in data3 if "00081030" in s)
        print(f"\n전체 {len(data3)}개 Study 중 Study Description 포함된 Study: {studies_with_desc}개")
    else:
        print("⚠️  Studies가 없습니다")
else:
    print(f"❌ 에러: {resp3.text[:200]}")

print()

# 테스트 4: 여러 includefield 포함
print("=" * 70)
print("테스트 4: 여러 includefield 포함 (00081030, 0020000D)")
print("=" * 70)
url4 = f"{BASE_URL}/api/dicom/studies?project_id=2&includefield=00081030&includefield=0020000D"
print(f"URL: {url4}")

resp4 = requests.get(url4, headers=headers, timeout=10)
print(f"Status: {resp4.status_code}")

if resp4.status_code == 200:
    data4 = resp4.json()
    if isinstance(data4, list) and len(data4) > 0:
        first_study = data4[0]
        has_study_desc = "00081030" in first_study
        has_study_uid = "0020000D" in first_study
        print(f"Study Description 태그 (00081030) 포함 여부: {has_study_desc}")
        print(f"Study UID 태그 (0020000D) 포함 여부: {has_study_uid}")
        if has_study_desc:
            study_desc = first_study.get("00081030")
            print(f"Study Description 값: {json.dumps(study_desc, indent=2, ensure_ascii=False)}")
    else:
        print("⚠️  Studies가 없습니다")
else:
    print(f"❌ 에러: {resp4.text[:200]}")

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)



