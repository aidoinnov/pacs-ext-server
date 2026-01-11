#!/usr/bin/env python3
"""Series API 확인"""
import requests
import json

# 로그인
login_resp = requests.post('http://localhost:8080/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = login_resp.json().get('token')
print('✅ 로그인 성공')

# Series 조회
headers = {'Authorization': f'Bearer {token}'}
resp = requests.get('http://localhost:8080/api/me/dicom/series?project_id=2&page=1&page_size=10', headers=headers)
print(f'Status: {resp.status_code}')

if resp.status_code == 200:
    data = resp.json()
    if isinstance(data, list):
        print(f'✅ Series 개수: {len(data)}')
        if data:
            print(f'\n첫 번째 Series:')
            print(json.dumps(data[0], indent=2, ensure_ascii=False))
    elif isinstance(data, dict):
        series_list = data.get('series', [])
        print(f'✅ Series 개수: {len(series_list)}')
        if series_list:
            print(f'\n첫 번째 Series:')
            print(json.dumps(series_list[0], indent=2, ensure_ascii=False))
        print(f'Total: {data.get("total", 0)}')
else:
    print(f'❌ Error: {resp.text[:200]}')

