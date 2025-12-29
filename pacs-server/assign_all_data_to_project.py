#!/usr/bin/env python3
"""project_id 2에 전체 데이터 할당 스크립트"""

import requests
import json
import sys
import os

BASE_URL = 'http://localhost:8080'
PROJECT_ID = 2

# DB 연결 정보 (환경 변수에서 가져오기)
DB_HOST = os.getenv('DB_HOST', 'localhost')
DB_PORT = os.getenv('DB_PORT', '5432')
DB_NAME = os.getenv('DB_NAME', 'pacs_db')
DB_USER = os.getenv('DB_USER', 'pacs_user')
DB_PASSWORD = os.getenv('DB_PASSWORD', 'pacs_password')

def get_headers(token: str = None):
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def get_studies_from_other_projects(token):
    """다른 프로젝트에 할당된 Study 조회"""
    print("   다른 프로젝트에서 Study 조회 시도...")
    
    # 프로젝트 목록 조회
    projects_response = requests.get(
        f'{BASE_URL}/api/projects',
        headers=get_headers(token)
    )
    
    if projects_response.status_code != 200:
        return None
    
    projects = projects_response.json()
    if not isinstance(projects, list):
        return None
    
    all_studies = []
    study_uids_seen = set()
    
    # 각 프로젝트에서 Study 조회
    for project in projects[:10]:  # 최대 10개 프로젝트만 확인
        project_id = project.get('id') or project.get('project_id')
        if not project_id or project_id == PROJECT_ID:
            continue
        
        studies_response = requests.get(
            f'{BASE_URL}/api/dicom/studies',
            params={'project_id': project_id, 'limit': 50},
            headers=get_headers(token)
        )
        
        if studies_response.status_code == 200:
            studies = studies_response.json()
            if isinstance(studies, list):
                for study in studies:
                    study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
                    if study_uid and study_uid not in study_uids_seen:
                        study_uids_seen.add(study_uid)
                        all_studies.append({
                            'study_uid': study_uid,
                            'study_description': study.get("00081030", {}).get("Value", [None])[0] if "00081030" in study else None,
                            'patient_id': study.get("00100020", {}).get("Value", [None])[0] if "00100020" in study else None,
                            'patient_name': study.get("00100010", {}).get("Value", [None])[0] if "00100010" in study else None,
                            'study_date': None
                        })
    
    return all_studies if len(all_studies) > 0 else None

def get_studies_from_api(token):
    """API를 통해 전체 Study 조회 시도 (전체 접근 권한 필요)"""
    print("   API를 통해 Study 조회 시도...")
    studies_response = requests.get(
        f'{BASE_URL}/api/dicom/studies',
        params={'limit': 100},
        headers=get_headers(token)
    )
    
    if studies_response.status_code == 200:
        studies = studies_response.json()
        if isinstance(studies, list):
            result = []
            for study in studies:
                study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
                if study_uid:
                    result.append({
                        'study_uid': study_uid,
                        'study_description': study.get("00081030", {}).get("Value", [None])[0] if "00081030" in study else None,
                        'patient_id': study.get("00100020", {}).get("Value", [None])[0] if "00100020" in study else None,
                        'patient_name': study.get("00100010", {}).get("Value", [None])[0] if "00100010" in study else None,
                        'study_date': None
                    })
            return result
    
    return None

def main():
    print("="*60)
    print(f"📦 project_id {PROJECT_ID}에 전체 데이터 할당")
    print("="*60)
    
    # 사용자 생성 및 로그인
    print("\n1. 사용자 생성 및 로그인...")
    import time
    user_data = {
        'username': f'assign_all_{int(time.time() * 1000)}',
        'email': f'assign_all_{int(time.time() * 1000)}@example.com',
        'password': 'TestPassword123!',
        'full_name': '전체 데이터 할당 사용자'
    }
    
    response = requests.post(f'{BASE_URL}/api/auth/signup', json=user_data)
    if response.status_code not in [200, 201]:
        print(f"❌ 사용자 생성 실패: {response.status_code}")
        return
    
    user_id = response.json().get('user_id') or response.json().get('id')
    print(f"✅ 사용자 생성: {user_id}")
    
    # 사용자 승인
    requests.post(
        f'{BASE_URL}/api/auth/admin/users/approve',
        json={'user_id': user_id},
        headers=get_headers()
    )
    
    # 로그인
    login_response = requests.post(
        f'{BASE_URL}/api/auth/keycloak-token',
        json={'username': user_data['username'], 'password': user_data['password']},
        headers=get_headers()
    )
    if login_response.status_code != 200:
        print(f"❌ 로그인 실패")
        return
    
    token = login_response.json().get('access_token')
    print(f"✅ 로그인 완료")
    
    # 프로젝트 확인
    print(f"\n2. 프로젝트 {PROJECT_ID} 확인...")
    project_response = requests.get(
        f'{BASE_URL}/api/projects/{PROJECT_ID}',
        headers=get_headers(token)
    )
    if project_response.status_code != 200:
        print(f"❌ 프로젝트 {PROJECT_ID}를 찾을 수 없습니다.")
        return
    print(f"✅ 프로젝트 확인 완료")
    
    # 사용자를 프로젝트에 추가 (ADMIN role로 전체 접근 권한 획득)
    print(f"\n3. 사용자를 프로젝트에 추가 (ADMIN role로 전체 접근 권한 획득)...")
    role_response = requests.get(
        f'{BASE_URL}/api/roles',
        headers=get_headers(token)
    )
    
    admin_role_id = None
    researcher_role_id = None
    
    if role_response.status_code == 200:
        roles = role_response.json()
        if isinstance(roles, list):
            for role in roles:
                role_name = role.get('name')
                role_id = role.get('id') or role.get('role_id')
                if role_name == 'ADMIN':
                    admin_role_id = role_id
                elif role_name == 'RESEARCHER':
                    researcher_role_id = role_id
    
    # ADMIN role이 있으면 사용 (전체 접근 권한)
    if admin_role_id:
        member_response = requests.post(
            f'{BASE_URL}/api/projects/{PROJECT_ID}/members',
            json={'user_id': user_id, 'role_id': admin_role_id},
            headers=get_headers(token)
        )
        print(f"✅ 사용자를 ADMIN role로 추가: {member_response.status_code}")
    elif researcher_role_id:
        member_response = requests.post(
            f'{BASE_URL}/api/projects/{PROJECT_ID}/members',
            json={'user_id': user_id, 'role_id': researcher_role_id},
            headers=get_headers(token)
        )
        print(f"✅ 사용자를 RESEARCHER role로 추가: {member_response.status_code}")
    
    # Study 조회
    print(f"\n4. Study 조회...")
    studies = None
    
    # 방법 1: 다른 프로젝트에서 Study 조회
    print("   방법 1: 다른 프로젝트에서 Study 조회 시도...")
    studies = get_studies_from_other_projects(token)
    
    if studies and len(studies) > 0:
        print(f"   ✅ 다른 프로젝트에서 {len(studies)}개의 Study 조회됨")
    else:
        # 방법 2: API를 통해 전체 조회 (전체 접근 권한 필요)
        studies = get_studies_from_api(token)
        if studies and len(studies) > 0:
            print(f"   ✅ API를 통해 {len(studies)}개의 Study 조회됨")
        else:
            print(f"   ❌ Study를 찾을 수 없습니다.")
            print(f"   해결 방법:")
            print(f"   1. 다른 프로젝트에 Study가 할당되어 있는지 확인")
            print(f"   2. 전체 접근 권한이 있는 사용자로 실행")
            return
    
    # Study 할당
    print(f"\n5. Study를 프로젝트 {PROJECT_ID}에 할당...")
    assigned_studies = 0
    assigned_series = 0
    
    for i, study in enumerate(studies):
        study_uid = study['study_uid']
        print(f"\n   [{i+1}/{len(studies)}] Study: {study_uid}")
        
        study_data = {
            'study_uid': study_uid,
            'study_description': study.get('study_description'),
            'patient_id': study.get('patient_id'),
            'patient_name': study.get('patient_name'),
            'study_date': study.get('study_date')
        }
        
        study_response = requests.post(
            f'{BASE_URL}/api/projects/{PROJECT_ID}/studies/assign',
            json=study_data,
            headers=get_headers(token)
        )
        
        if study_response.status_code in [200, 201]:
            print(f"      ✅ Study 할당 성공")
            assigned_studies += 1
            
            # 해당 Study의 Series 조회 및 할당
            series_response = requests.get(
                f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
                params={'project_id': PROJECT_ID, 'limit': 50},
                headers=get_headers(token)
            )
            
            if series_response.status_code == 200:
                series_list = series_response.json()
                if isinstance(series_list, list) and len(series_list) > 0:
                    print(f"      📋 {len(series_list)}개의 Series 발견")
                    
                    for j, series in enumerate(series_list[:10]):  # 최대 10개만 할당
                        series_uid = series.get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series else None
                        
                        if series_uid:
                            series_data = {
                                'study_uid': study_uid,
                                'series_uid': series_uid,
                                'series_description': series.get("0008103E", {}).get("Value", [None])[0] if "0008103E" in series else None,
                                'modality': series.get("00080060", {}).get("Value", [None])[0] if "00080060" in series else "CT",
                                'series_number': series.get("00200011", {}).get("Value", [None])[0] if "00200011" in series else j + 1
                            }
                            
                            series_assign_response = requests.post(
                                f'{BASE_URL}/api/projects/{PROJECT_ID}/series/assign',
                                json=series_data,
                                headers=get_headers(token)
                            )
                            
                            if series_assign_response.status_code in [200, 201]:
                                assigned_series += 1
                                if j < 3:  # 처음 3개만 출력
                                    print(f"         ✅ Series {j+1} 할당: {series_uid}")
                else:
                    print(f"      ⚠️  Series 없음")
            else:
                print(f"      ⚠️  Series 조회 실패: {series_response.status_code}")
        else:
            print(f"      ⚠️  Study 할당 실패 (이미 할당되어 있을 수 있음): {study_response.status_code}")
    
    print(f"\n" + "="*60)
    print(f"✅ 완료!")
    print(f"   할당된 Study: {assigned_studies}개")
    print(f"   할당된 Series: {assigned_series}개")
    print("="*60)

if __name__ == '__main__':
    main()

