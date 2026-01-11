#!/usr/bin/env python3
"""디버그 스크립트: Report Status 필터링 문제 진단"""

import requests
import json
import time
from datetime import date, timedelta

BASE_URL = 'http://localhost:8080'

def main():
    # 사용자 생성 및 로그인
    user_data = {
        'username': f'testuser_debug_{int(time.time() * 1000)}',
        'email': f'test_debug_{int(time.time() * 1000)}@example.com',
        'password': 'TestPassword123!',
        'full_name': '테스트 사용자'
    }

    response = requests.post(f'{BASE_URL}/api/auth/signup', json=user_data)
    print(f'Signup: {response.status_code}')
    if response.status_code not in [200, 201]:
        print(f'Signup failed: {response.text}')
        return
    
    result = response.json()
    user_id = result.get('user_id') or result.get('id')
    print(f'User ID: {user_id}')
    
    # 사용자 승인
    approve_response = requests.post(
        f'{BASE_URL}/api/auth/admin/users/approve',
        json={'user_id': user_id},
        headers={'Content-Type': 'application/json'}
    )
    print(f'Approve: {approve_response.status_code}')
    
    # 로그인
    login_response = requests.post(
        f'{BASE_URL}/api/auth/keycloak-token',
        json={'username': user_data['username'], 'password': user_data['password']},
        headers={'Content-Type': 'application/json'}
    )
    if login_response.status_code != 200:
        print(f'Login failed: {login_response.text}')
        return
    
    token = login_response.json().get('access_token')
    print(f'Token obtained: {token[:50] if token else None}...')
    headers = {'Authorization': f'Bearer {token}', 'Content-Type': 'application/json'}
    
    # 프로젝트 생성
    today = date.today()
    project_data = {
        'name': f'test_project_debug_{int(time.time())}',
        'description': 'Debug test project',
        'sponsor': 'Test Sponsor',
        'start_date': str(today),
        'end_date': str(today + timedelta(days=365))
    }
    project_response = requests.post(
        f'{BASE_URL}/api/projects',
        json=project_data,
        headers=headers
    )
    if project_response.status_code not in [200, 201]:
        print(f'Project creation failed: {project_response.text}')
        return
    
    project_id = project_response.json().get('id') or project_response.json().get('project_id')
    print(f'Project ID: {project_id}')
    
    # 사용자를 프로젝트에 추가
    role_response = requests.get(
        f'{BASE_URL}/api/roles',
        headers=headers
    )
    role_id = None
    if role_response.status_code == 200:
        roles = role_response.json()
        if isinstance(roles, list):
            for role in roles:
                if role.get('name') == 'RESEARCHER':
                    role_id = role.get('id') or role.get('role_id')
                    break
    
    if role_id:
        member_response = requests.post(
            f'{BASE_URL}/api/projects/{project_id}/members',
            json={'user_id': user_id, 'role_id': role_id},
            headers=headers
        )
        print(f'Add member: {member_response.status_code}')
    
    # Study 할당
    study_uid = f'1.2.840.113619.2.1.1.{int(time.time())}'
    study_data = {
        'study_uid': study_uid,
        'study_description': 'Test Study for Debug',
        'patient_id': 'TEST001',
        'patient_name': 'Test Patient',
        'study_date': None
    }
    study_response = requests.post(
        f'{BASE_URL}/api/projects/{project_id}/studies/assign',
        json=study_data,
        headers=headers
    )
    print(f'Study assign: {study_response.status_code}')
    if study_response.status_code not in [200, 201]:
        print(f'Study assign failed: {study_response.text}')
        return
    
    study_result = study_response.json()
    print(f'Study result keys: {list(study_result.keys())}')
    
    # Series 할당
    series_uid = f'1.2.840.113619.2.1.2.{int(time.time())}'
    series_data = {
        'study_uid': study_uid,
        'series_uid': series_uid,
        'series_description': 'Test Series for Debug',
        'modality': 'CT',
        'series_number': 1
    }
    series_response = requests.post(
        f'{BASE_URL}/api/projects/{project_id}/series/assign',
        json=series_data,
        headers=headers
    )
    print(f'Series assign: {series_response.status_code}')
    if series_response.status_code not in [200, 201]:
        print(f'Series assign failed: {series_response.text}')
        return
    
    series_result = series_response.json()
    print(f'Series result: {json.dumps(series_result, indent=2)}')
    series_id = series_result.get('series_id') or series_result.get('data', {}).get('series', {}).get('id')
    print(f'Series ID: {series_id}')
    
    # Report 생성
    report_data = {
        'status': 'approval',
        'description': 'Test description',
        'conclusion': 'Test conclusion'
    }
    report_response = requests.put(
        f'{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report',
        json=report_data,
        headers=headers
    )
    print(f'Report create: {report_response.status_code}')
    if report_response.status_code in [200, 201]:
        print(f'Report created successfully')
    else:
        print(f'Report creation failed: {report_response.text}')
    
    # DICOM Gateway에서 Series 조회 (필터링 없이)
    time.sleep(1)  # DB 동기화 대기
    print(f'\n=== Testing DICOM Gateway without filter ===')
    gateway_response = requests.get(
        f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
        params={'project_id': project_id},
        headers=headers
    )
    print(f'Gateway response (no filter): {gateway_response.status_code}')
    if gateway_response.status_code == 200:
        series_list = gateway_response.json()
        print(f'Series list length: {len(series_list) if isinstance(series_list, list) else 0}')
        if isinstance(series_list, list) and len(series_list) > 0:
            print(f'First series UID: {series_list[0].get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series_list[0] else "N/A"}')
    else:
        print(f'Gateway error: {gateway_response.text}')
    
    # DICOM Gateway에서 Series 조회 (필터링 포함)
    print(f'\n=== Testing DICOM Gateway with report_status=approval ===')
    gateway_response_filtered = requests.get(
        f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
        params={'project_id': project_id, 'report_status': 'approval'},
        headers=headers
    )
    print(f'Gateway response (with filter): {gateway_response_filtered.status_code}')
    if gateway_response_filtered.status_code == 200:
        series_list_filtered = gateway_response_filtered.json()
        print(f'Filtered series list length: {len(series_list_filtered) if isinstance(series_list_filtered, list) else 0}')
        if isinstance(series_list_filtered, list) and len(series_list_filtered) > 0:
            print(f'First filtered series UID: {series_list_filtered[0].get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series_list_filtered[0] else "N/A"}')
    else:
        print(f'Gateway error: {gateway_response_filtered.text}')

if __name__ == '__main__':
    main()





