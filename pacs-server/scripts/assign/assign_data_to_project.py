#!/usr/bin/env python3
"""project_id 2에 Study/Series 할당 스크립트"""

import requests
import json
import sys

BASE_URL = 'http://localhost:8080'
PROJECT_ID = 2

def get_headers(token: str = None):
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def main():
    print("="*60)
    print(f"📦 project_id {PROJECT_ID}에 데이터 할당")
    print("="*60)
    
    # 사용자 생성 및 로그인 (또는 기존 사용자 사용)
    print("\n1. 사용자 생성 및 로그인...")
    user_data = {
        'username': f'assign_user_{int(__import__("time").time() * 1000)}',
        'email': f'assign_{int(__import__("time").time() * 1000)}@example.com',
        'password': 'TestPassword123!',
        'full_name': '데이터 할당 사용자'
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
    
    # 사용자를 프로젝트에 추가
    print(f"\n3. 사용자를 프로젝트에 추가...")
    role_response = requests.get(
        f'{BASE_URL}/api/roles',
        headers=get_headers(token)
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
            f'{BASE_URL}/api/projects/{PROJECT_ID}/members',
            json={'user_id': user_id, 'role_id': role_id},
            headers=get_headers(token)
        )
        print(f"✅ 사용자 추가: {member_response.status_code}")
    
    # 프로젝트에 할당된 Study 확인
    print(f"\n4. 프로젝트에 할당된 Study 확인...")
    studies_response = requests.get(
        f'{BASE_URL}/api/dicom/studies',
        params={'project_id': PROJECT_ID, 'limit': 10},
        headers=get_headers(token)
    )
    
    if studies_response.status_code == 200:
        studies = studies_response.json()
        if isinstance(studies, list):
            print(f"✅ 이미 할당된 Study: {len(studies)}개")
            if len(studies) > 0:
                print("   할당된 Study 목록:")
                for i, study in enumerate(studies[:5]):
                    study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
                    print(f"   {i+1}. {study_uid}")
                return
        else:
            studies = []
    else:
        studies = []
        print(f"⚠️  Study 조회 실패: {studies_response.status_code}")
    
    # Study가 없으면, 동기화된 Study UID를 직접 사용
    if len(studies) == 0:
        print("\n5. 동기화된 Study UID 사용...")
        print("   (동기화가 완료되었다면 DB에 Study가 있을 것입니다)")
        print("   Study UID를 직접 입력하거나, 알려진 Study UID를 사용합니다.")
        
        # 예시: 동기화 테스트에서 사용한 Study UID 패턴
        # 실제로는 사용자가 Study UID를 입력하거나, DB에서 조회해야 함
        print("\n   Study UID를 입력하세요 (엔터를 누르면 예시 UID 사용):")
        study_uid_input = input("   Study UID: ").strip()
        
        if not study_uid_input:
            print("   ⚠️  Study UID가 입력되지 않았습니다.")
            print("   동기화가 완료되었다면, DB에서 Study UID를 조회해야 합니다.")
            print("\n   대안: DICOM Gateway를 통해 Study를 조회하려면")
            print("   전체 접근 권한이 있는 사용자로 실행해야 합니다.")
            return
        
        # 입력받은 Study UID로 Study 객체 생성
        studies = [{
            "0020000D": {"Value": [study_uid_input]},
            "00081030": {"Value": ["Test Study"]},
            "00100020": {"Value": ["TEST001"]},
            "00100010": {"Value": ["Test Patient"]}
        }]
        print(f"   ✅ Study UID 사용: {study_uid_input}")
    
    # Study 할당
    print(f"\n6. Study를 프로젝트 {PROJECT_ID}에 할당...")
    assigned_count = 0
    
    for i, study in enumerate(studies[:5]):  # 최대 5개만 할당
        study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
        if not study_uid:
            continue
        
        study_data = {
            'study_uid': study_uid,
            'study_description': study.get("00081030", {}).get("Value", [None])[0] if "00081030" in study else None,
            'patient_id': study.get("00100020", {}).get("Value", [None])[0] if "00100020" in study else None,
            'patient_name': study.get("00100010", {}).get("Value", [None])[0] if "00100010" in study else None,
            'study_date': None
        }
        
        study_response = requests.post(
            f'{BASE_URL}/api/projects/{PROJECT_ID}/studies/assign',
            json=study_data,
            headers=get_headers(token)
        )
        
        if study_response.status_code in [200, 201]:
            print(f"   ✅ Study {i+1} 할당: {study_uid}")
            assigned_count += 1
            
            # 해당 Study의 Series 조회 및 할당
            series_response = requests.get(
                f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
                params={'project_id': PROJECT_ID, 'limit': 10},
                headers=get_headers(token)
            )
            
            if series_response.status_code == 200:
                series_list = series_response.json()
                if isinstance(series_list, list) and len(series_list) > 0:
                    # 첫 번째 Series 할당
                    series = series_list[0]
                    series_uid = series.get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series else None
                    
                    if series_uid:
                        series_data = {
                            'study_uid': study_uid,
                            'series_uid': series_uid,
                            'series_description': series.get("0008103E", {}).get("Value", [None])[0] if "0008103E" in series else None,
                            'modality': series.get("00080060", {}).get("Value", [None])[0] if "00080060" in series else "CT",
                            'series_number': series.get("00200011", {}).get("Value", [None])[0] if "00200011" in series else 1
                        }
                        
                        series_assign_response = requests.post(
                            f'{BASE_URL}/api/projects/{PROJECT_ID}/series/assign',
                            json=series_data,
                            headers=get_headers(token)
                        )
                        
                        if series_assign_response.status_code in [200, 201]:
                            print(f"      ✅ Series 할당: {series_uid}")
        else:
            print(f"   ⚠️  Study {i+1} 할당 실패 (이미 할당되어 있을 수 있음): {study_response.status_code}")
    
    print(f"\n✅ 완료: {assigned_count}개의 Study 할당됨")

if __name__ == '__main__':
    main()

