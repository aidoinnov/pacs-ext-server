#!/usr/bin/env python3
"""
로깅이 추가된 서버에서 API 호출하여 문제 진단
"""
import requests
import time

BASE_URL = "http://localhost:8080"

print("=" * 60)
print("🔍 로깅을 통한 문제 진단")
print("=" * 60)
print("\n⚠️  서버를 재시작한 후 이 스크립트를 실행하세요!")
print("   서버 로그에서 다음 메시지를 확인하세요:")
print("   - '🔍 Gateway /series: Found {} allowed series UIDs for project {}'")
print("   - '🔍 Gateway /series: QIDO returned {} series'")
print("   - '🔍 Gateway /series: Filtered {} series from {} QIDO results'")
print()

input("서버를 재시작했으면 Enter를 누르세요...")

# 로그인
print("🔐 로그인 중...")
login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = login_resp.json().get('token')
headers = {'Authorization': f'Bearer {token}'}
print("✅ 로그인 성공\n")

# API 호출
print("📡 /api/me/dicom/series?project_id=2 호출 중...")
series_resp = requests.get(
    f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=10',
    headers=headers
)
print(f"Status: {series_resp.status_code}")

if series_resp.status_code == 200:
    series_data = series_resp.json()
    if isinstance(series_data, list):
        print(f"✅ Series 개수: {len(series_data)}")
    elif isinstance(series_data, dict):
        series_list = series_data.get('series', [])
        print(f"✅ Series 개수: {len(series_list)}")
        print(f"Total: {series_data.get('total', 0)}")
else:
    print(f"❌ Error: {series_resp.text[:200]}")

print("\n" + "=" * 60)
print("✅ API 호출 완료")
print("=" * 60)
print("\n이제 서버 로그를 확인하세요:")
print("1. '🔍 Gateway /series: Found {} allowed series UIDs' - 허용된 Series UID 개수")
print("2. '🔍 Gateway /series: QIDO returned {} series' - QIDO에서 반환된 Series 개수")
print("3. '🔍 Gateway /series: Filtered {} series' - 필터링 후 남은 Series 개수")
print("\n만약:")
print("- allowed_series_uids가 0이면 → DB 쿼리 문제")
print("- QIDO returned가 0이면 → Dcm4chee 연결 문제")
print("- Filtered가 0이면 → 필터링 로직 문제")
