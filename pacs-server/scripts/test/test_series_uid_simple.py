#!/usr/bin/env python3
"""
간단한 Series UID API 테스트
"""
import requests
import json

BASE_URL = "http://localhost:8080"
SERIES_UID = "1.2.840.113619.2.311.168624790352053237183428645578553404611"

print("=" * 60)
print("🧪 간단한 Series UID API 테스트")
print("=" * 60)

# 로그인
print("\n1️⃣ 로그인")
resp = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = resp.json().get('token')
print(f"✅ 토큰 획득: {token[:50]}...")

headers = {
    'Authorization': f'Bearer {token}',
    'Content-Type': 'application/json'
}

# Note API 테스트
print(f"\n2️⃣ Note API 테스트")
print(f"   URL: /api/series/{SERIES_UID}/note")
print("-" * 60)

# GET 테스트
resp = requests.get(f'{BASE_URL}/api/series/{SERIES_UID}/note', headers=headers)
print(f"GET Status: {resp.status_code}")
if resp.status_code == 400:
    error_text = resp.text
    if "can not parse" in error_text.lower() or "i32" in error_text.lower():
        print(f"❌ 여전히 i32 파싱 에러!")
        print(f"   {error_text[:300]}")
    else:
        print(f"   {error_text[:200]}")
elif resp.status_code == 200:
    print("✅ 성공!")
    print(f"   {json.dumps(resp.json(), indent=2, ensure_ascii=False)[:200]}")
else:
    print(f"   {resp.text[:200]}")

# Report API 테스트
print(f"\n3️⃣ Report API 테스트")
print(f"   URL: /api/series/{SERIES_UID}/report")
print("-" * 60)

# GET 테스트
resp = requests.get(f'{BASE_URL}/api/series/{SERIES_UID}/report', headers=headers)
print(f"GET Status: {resp.status_code}")
if resp.status_code == 400:
    error_text = resp.text
    if "can not parse" in error_text.lower() or "i32" in error_text.lower():
        print(f"❌ 여전히 i32 파싱 에러!")
        print(f"   {error_text[:300]}")
    else:
        print(f"   {error_text[:200]}")
elif resp.status_code == 200:
    print("✅ 성공!")
    print(f"   {json.dumps(resp.json(), indent=2, ensure_ascii=False)[:200]}")
elif resp.status_code == 404:
    print("⚠️  404 (Series를 찾을 수 없음 - 정상일 수 있음)")
    print(f"   {resp.text[:200]}")
else:
    print(f"   {resp.text[:200]}")

print("\n" + "=" * 60)
print("✅ 테스트 완료")
print("=" * 60)
print("\n⚠️  만약 여전히 i32 파싱 에러가 발생한다면:")
print("   1. 서버를 재시작했는지 확인")
print("   2. 빌드가 최신인지 확인 (cargo build --release)")
print("   3. 다른 엔드포인트가 같은 경로를 사용하는지 확인")

