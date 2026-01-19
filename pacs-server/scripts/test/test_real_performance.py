#!/usr/bin/env python3
"""
실제 사용자 요청 URL로 성능 테스트
"""

import requests
import time

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

token = login_resp.json().get("token")
headers = {"Authorization": f"Bearer {token}"}
print("✅ 로그인 성공\n")

# 사용자가 제공한 정확한 URL
url = "http://localhost:8080/api/me/dicom/series?project_id=2&page=1&page_size=200&user_id=56"

print("=" * 70)
print("📊 실제 사용자 요청 URL 테스트")
print("=" * 70)
print(f"URL: {url}")
print("\n⏱️  측정 시작...")

start = time.time()
try:
    response = requests.get(url, headers=headers, timeout=60)
    elapsed = time.time() - start
    
    print(f"\n✅ Status: {response.status_code}")
    print(f"⏱️  응답 시간: {elapsed:.3f}초 ({elapsed*1000:.0f}ms)")
    
    if response.status_code == 200:
        data = response.json()
        if isinstance(data, list):
            print(f"📦 반환된 Series 수: {len(data)}")
            if len(data) > 0:
                print(f"   첫 번째 Series UID: {data[0].get('0020000E', {}).get('Value', ['N/A'])[0] if '0020000E' in data[0] else 'N/A'}")
        else:
            print(f"📦 응답 타입: {type(data)}")
            print(f"   응답 내용 (처음 200자): {str(data)[:200]}")
    else:
        print(f"❌ 에러 응답:")
        print(response.text[:500])
        
except Exception as e:
    elapsed = time.time() - start
    print(f"\n❌ 에러 발생: {e}")
    print(f"⏱️  실패까지 걸린 시간: {elapsed:.3f}초")

print("\n" + "=" * 70)
if elapsed > 3:
    print("⚠️  응답 시간이 3초를 초과했습니다!")
    print("   최적화가 제대로 적용되지 않았을 수 있습니다.")
elif elapsed > 1:
    print("⚠️  응답 시간이 1초를 초과했습니다.")
    print("   추가 최적화가 필요할 수 있습니다.")
else:
    print("✅ 응답 시간이 양호합니다.")





