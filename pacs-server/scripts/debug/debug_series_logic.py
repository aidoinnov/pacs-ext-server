#!/usr/bin/env python3
"""
로직 디버깅: get_allowed_series_uids 쿼리와 실제 데이터 비교
"""
import requests
import json

BASE_URL = "http://localhost:8080"

# 로그인
login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = login_resp.json().get('token')
print('✅ 로그인 성공\n')

# 1. Dcm4chee QIDO 호출 (모든 Series)
print("=" * 60)
print("1. Dcm4chee QIDO 호출 (모든 Series)")
print("=" * 60)
headers = {'Authorization': f'Bearer {token}'}
qido_resp = requests.get(f'{BASE_URL}/api/dicom/studies_raw?limit=10', headers=headers)
print(f"Status: {qido_resp.status_code}")
if qido_resp.status_code == 200:
    studies = qido_resp.json()
    if isinstance(studies, list):
        print(f"✅ Study 개수: {len(studies)}")
        if studies:
            study_uid = studies[0].get('0020000D', {}).get('Value', [None])[0]
            print(f"첫 번째 Study UID: {study_uid}")
            
            # 해당 Study의 Series 조회
            series_resp = requests.get(
                f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
                headers=headers
            )
            if series_resp.status_code == 200:
                series_list = series_resp.json()
                if isinstance(series_list, list):
                    print(f"✅ Series 개수: {len(series_list)}")
                    if series_list:
                        series_uid = series_list[0].get('0020000E', {}).get('Value', [None])[0]
                        print(f"첫 번째 Series UID: {series_uid}")
    else:
        print(f"응답 형식: {type(studies)}")
        print(json.dumps(studies, indent=2, ensure_ascii=False)[:500])
else:
    print(f"❌ Error: {qido_resp.text[:200]}")

print("\n" + "=" * 60)
print("2. /api/me/dicom/series?project_id=2 호출")
print("=" * 60)
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
            print(json.dumps(series_data[0], indent=2, ensure_ascii=False)[:500])
    elif isinstance(series_data, dict):
        series_list = series_data.get('series', [])
        print(f"✅ Series 개수: {len(series_list)}")
        if series_list:
            print(f"첫 번째 Series:")
            print(json.dumps(series_list[0], indent=2, ensure_ascii=False)[:500])
else:
    print(f"❌ Error: {series_resp.text[:200]}")

print("\n" + "=" * 60)
print("3. get_allowed_series_uids 쿼리 시뮬레이션")
print("=" * 60)
print("""
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL
""")
print("\n이 쿼리는:")
print("1. project_data에서 project_id=2인 행을 찾음")
print("2. project_data_study와 조인 (pd.study_id = pds.id)")
print("3. project_data_series와 조인 (pds.id = pdser.study_id)")
print("\n⚠️  문제 가능성:")
print("- project_data에 study_id가 NULL이면 조인 실패")
print("- project_data에 series_id만 있고 study_id가 없으면 조인 실패")
print("- resource_level이 'SERIES'인 경우 study_id도 함께 저장되어야 함")

print("\n" + "=" * 60)
print("4. 할당 API 확인")
print("=" * 60)
print("assign_series_to_project는 다음을 수행:")
print("1. project_data_study에 Study 생성/조회")
print("2. project_data_series에 Series 생성/조회")
print("3. project_data에 다음으로 저장:")
print("   INSERT INTO project_data (project_id, resource_level, study_id, series_id)")
print("   VALUES ($1, 'SERIES', $2, $3)")
print("\n✅ study_id와 series_id가 모두 저장되므로 쿼리는 작동해야 함")

