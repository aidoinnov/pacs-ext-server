#!/usr/bin/env python3
"""
includefield 파라미터 전달 확인 테스트 (로그 확인용)
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

# includefield 파라미터 포함하여 호출
print("=" * 70)
print("includefield=00081030 파라미터 포함하여 호출")
print("=" * 70)
url = f"{BASE_URL}/api/dicom/series?project_id=2&page=1&page_size=3&includefield=00081030"
print(f"URL: {url}")
print(f"요청 파라미터: includefield=00081030")
print()

resp = requests.get(url, headers=headers, timeout=10)
print(f"Status: {resp.status_code}")

if resp.status_code == 200:
    data = resp.json()
    if isinstance(data, list) and len(data) > 0:
        print(f"✅ Series {len(data)}개 반환됨")
        first_series = data[0]
        print(f"\n첫 번째 Series의 모든 태그 키:")
        for key in sorted(first_series.keys()):
            if key == "00081030":
                print(f"  ✅ {key} (Study Description) - 포함됨!")
                print(f"     값: {json.dumps(first_series[key], indent=6, ensure_ascii=False)}")
            elif key == "thumbnail_url":
                print(f"  📷 {key} (thumbnail_url)")
            else:
                print(f"  📋 {key}")
        
        has_study_desc = "00081030" in first_series
        print(f"\n결과: Study Description 태그 (00081030) 포함 여부 = {has_study_desc}")
    else:
        print("⚠️  Series가 없습니다")
        print(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
else:
    print(f"❌ 에러 응답:")
    print(resp.text[:500])
    print("\n⚠️  QIDO 호출이 실패했습니다. 서버 로그에서 'Query params'를 확인하세요.")
    print("   tail -f backend.log | grep 'Query params'")

print()
print("=" * 70)
print("테스트 완료")
print("=" * 70)
print("\n💡 참고:")
print("   - includefield 파라미터는 build_qido_params_from_user_query에서")
print("     자동으로 QIDO 파라미터로 전달됩니다 (패스스루)")
print("   - 서버 로그에서 '📊 Query params'를 확인하면 includefield가")
print("     포함되어 있는지 확인할 수 있습니다")



