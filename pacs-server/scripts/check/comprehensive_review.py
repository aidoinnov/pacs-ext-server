#!/usr/bin/env python3
"""
종합 검토: 현재 상태 확인
"""
import requests
import json

BASE_URL = "http://localhost:8080"

print("=" * 60)
print("🔍 종합 검토: /api/me/dicom/series?project_id=2")
print("=" * 60)

# 로그인
login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = login_resp.json().get('token')
headers = {'Authorization': f'Bearer {token}'}

print("\n1️⃣ 할당된 Series 확인 (할당 API 테스트)")
print("-" * 60)
# 할당된 Series 중 하나를 직접 조회해보기
test_resp = requests.get(
    f'{BASE_URL}/api/projects/2/series?page=1&page_size=5',
    headers=headers
)
print(f"Status: {test_resp.status_code}")
if test_resp.status_code == 200:
    data = test_resp.json()
    if isinstance(data, list):
        print(f"✅ 할당된 Series 개수: {len(data)}")
        if data:
            print(f"첫 번째 Series:")
            print(json.dumps(data[0], indent=2, ensure_ascii=False)[:300])
    elif isinstance(data, dict):
        series_list = data.get('series', [])
        print(f"✅ 할당된 Series 개수: {len(series_list)}")
        if series_list:
            print(f"첫 번째 Series:")
            print(json.dumps(series_list[0], indent=2, ensure_ascii=False)[:300])
else:
    print(f"❌ Error: {test_resp.text[:200]}")

print("\n2️⃣ /api/me/dicom/series?project_id=2 호출")
print("-" * 60)
series_resp = requests.get(
    f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=10',
    headers=headers
)
print(f"Status: {series_resp.status_code}")
if series_resp.status_code == 200:
    series_data = series_resp.json()
    if isinstance(series_data, list):
        print(f"✅ Series 개수: {len(series_data)}")
        if series_data:
            print(f"첫 번째 Series:")
            print(json.dumps(series_data[0], indent=2, ensure_ascii=False)[:300])
    elif isinstance(series_data, dict):
        series_list = series_data.get('series', [])
        print(f"✅ Series 개수: {len(series_list)}")
        if series_list:
            print(f"첫 번째 Series:")
            print(json.dumps(series_list[0], indent=2, ensure_ascii=False)[:300])
        print(f"Total: {series_data.get('total', 0)}")
else:
    print(f"❌ Error: {series_resp.text[:200]}")

print("\n3️⃣ Dcm4chee QIDO 직접 호출 테스트")
print("-" * 60)
# Dcm4chee QIDO가 작동하는지 확인
qido_resp = requests.get(
    f'{BASE_URL}/api/dicom/studies?limit=1',
    headers=headers
)
print(f"Status: {qido_resp.status_code}")
if qido_resp.status_code == 200:
    studies = qido_resp.json()
    if isinstance(studies, list):
        print(f"✅ Study 개수: {len(studies)}")
        if studies:
            study_uid = studies[0].get('0020000D', {}).get('Value', [None])[0]
            print(f"첫 번째 Study UID: {study_uid}")
            
            # 해당 Study의 Series 조회
            if study_uid:
                series_resp2 = requests.get(
                    f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
                    headers=headers
                )
                if series_resp2.status_code == 200:
                    series_list2 = series_resp2.json()
                    if isinstance(series_list2, list):
                        print(f"✅ Series 개수: {len(series_list2)}")
                        if series_list2:
                            series_uid = series_list2[0].get('0020000E', {}).get('Value', [None])[0]
                            print(f"첫 번째 Series UID: {series_uid}")
    else:
        print(f"응답 형식: {type(studies)}")
        print(json.dumps(studies, indent=2, ensure_ascii=False)[:300])
else:
    print(f"❌ Error: {qido_resp.text[:200]}")

print("\n4️⃣ 로직 분석")
print("-" * 60)
print("""
get_series_all 함수 흐름:
1. Dcm4chee QIDO 호출 (qido_series_all_with_bearer)
   → 모든 Series 조회
   
2. get_allowed_series_uids(project_id) 호출
   → DB에서 허용된 Series UID 목록 조회
   → 쿼리:
     SELECT DISTINCT pdser.series_uid
     FROM project_data pd
     INNER JOIN project_data_study pds ON pd.study_id = pds.id
     INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
     WHERE pd.project_id = 2
       AND pdser.series_uid IS NOT NULL
   
3. 필터링
   → QIDO 결과에서 허용된 Series만 필터링

⚠️  문제 가능성:
- get_allowed_series_uids가 빈 결과를 반환
- Dcm4chee QIDO가 빈 결과를 반환
- 필터링 후 결과가 없음
""")

print("\n5️⃣ 확인 필요 사항")
print("-" * 60)
print("""
1. DB에서 직접 쿼리 실행:
   SELECT DISTINCT pdser.series_uid
   FROM project_data pd
   INNER JOIN project_data_study pds ON pd.study_id = pds.id
   INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
   WHERE pd.project_id = 2
     AND pdser.series_uid IS NOT NULL;

2. 서버 로그 확인:
   - "Gateway /series: Found {} allowed series UIDs for project {}"
   - "Gateway /series: Filtered {} series from {} QIDO results"

3. project_data 테이블 확인:
   SELECT COUNT(*) FROM project_data WHERE project_id = 2;
   SELECT * FROM project_data WHERE project_id = 2 LIMIT 5;
""")

print("\n" + "=" * 60)
print("✅ 검토 완료")
print("=" * 60)

