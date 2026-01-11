#!/usr/bin/env python3
"""Dcm4chee에 실제 데이터가 있는지 확인"""

import requests
import json

BASE_URL = 'http://localhost:8080'

# 기존 사용자로 로그인 (또는 테스트 토큰 사용)
# 먼저 실제 Dcm4chee에 어떤 Study가 있는지 확인
response = requests.get(
    f'{BASE_URL}/api/dicom/studies',
    params={'limit': 5},
    headers={'Authorization': 'Bearer test'}  # 개발 모드 토큰
)

print(f'Status: {response.status_code}')
if response.status_code == 200:
    studies = response.json()
    print(f'Studies count: {len(studies) if isinstance(studies, list) else 0}')
    if isinstance(studies, list) and len(studies) > 0:
        study = studies[0]
        study_uid = study.get('0020000D', {}).get('Value', [None])[0] if '0020000D' in study else None
        print(f'First Study UID: {study_uid}')
        
        if study_uid:
            # 해당 Study의 Series 조회
            series_response = requests.get(
                f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
                params={'limit': 5},
                headers={'Authorization': 'Bearer test'}
            )
            print(f'Series response status: {series_response.status_code}')
            if series_response.status_code == 200:
                series_list = series_response.json()
                print(f'Series count: {len(series_list) if isinstance(series_list, list) else 0}')
                if isinstance(series_list, list) and len(series_list) > 0:
                    series_uid = series_list[0].get('0020000E', {}).get('Value', [None])[0] if '0020000E' in series_list[0] else None
                    print(f'First Series UID: {series_uid}')
                    print(f'First Series data: {json.dumps(series_list[0], indent=2)[:500]}')
else:
    print(f'Error: {response.text[:500]}')





