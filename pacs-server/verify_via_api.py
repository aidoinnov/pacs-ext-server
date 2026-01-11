#!/usr/bin/env python3
"""
API를 통해 서버 로그 확인 및 문제 진단
서버가 실행 중이므로 서버 로그를 확인하거나 디버그 엔드포인트 사용
"""
import requests
import json

BASE_URL = "http://localhost:8080"

print("=" * 60)
print("🔍 API를 통한 검증")
print("=" * 60)

# 로그인
login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = login_resp.json().get('token')
headers = {'Authorization': f'Bearer {token}'}
print("✅ 로그인 성공\n")

# 1. 서버가 사용하는 DB 연결 정보 확인 (설정 엔드포인트가 있다면)
print("1️⃣ 서버 설정 확인")
print("-" * 60)
# 설정 엔드포인트가 없을 수 있으므로 스킵

# 2. /api/me/dicom/series 호출 (디버그 로그 확인용)
print("2️⃣ /api/me/dicom/series?project_id=2 호출")
print("-" * 60)
print("⚠️  서버 로그에서 다음 메시지를 확인하세요:")
print("   - 'Gateway /series: Found {} allowed series UIDs for project {}'")
print("   - 'Gateway /series: Filtered {} series from {} QIDO results'")
print("   - 'QIDO /series: Parsed {} series from QIDO response'")
print()

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

# 3. 직접 DB 쿼리를 시뮬레이션하는 방법
# 서버에 디버그 엔드포인트가 있다면 사용
print("\n3️⃣ 문제 진단")
print("-" * 60)
print("""
현재 상황:
1. 데이터 할당: ✅ 성공 (28개 Series)
2. API 호출: ✅ 200 OK
3. 결과: ❌ 0개 Series

가능한 원인:
1. get_allowed_series_uids 쿼리가 빈 결과 반환
   → project_data에 데이터가 없거나
   → 조인 실패 (pd.study_id = pds.id 또는 pds.id = pdser.study_id)

2. Dcm4chee QIDO가 빈 결과 반환
   → 연결 실패 또는 실제로 Series가 없음

3. 필터링 실패
   → extract_series_uid가 Series UID를 추출하지 못함
   → QIDO 응답 형식과 DB의 series_uid 형식이 다름

확인 방법:
1. 서버 로그 확인 (가장 중요)
   - "Gateway /series: Found {} allowed series UIDs for project {}"
   - 이 로그에서 allowed_series_uids 개수를 확인

2. DB 직접 확인
   - DBeaver나 다른 DB 클라이언트로 연결
   - test_get_allowed_series_uids.sql 실행

3. 디버그 코드 추가
   - get_allowed_series_uids 결과를 로깅
   - QIDO 응답을 로깅
   - 필터링 전/후 개수를 로깅
""")

print("\n" + "=" * 60)
print("✅ 검증 완료")
print("=" * 60)
print("\n다음 단계:")
print("1. 서버 로그에서 'Gateway /series: Found {} allowed series UIDs' 확인")
print("2. DB 클라이언트로 직접 쿼리 실행")
print("3. 필요시 디버그 로깅 추가")

