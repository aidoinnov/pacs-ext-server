#!/usr/bin/env python3
"""Dcm4chee 동기화 테스트 스크립트"""

import requests
import json
import time
from datetime import date, timedelta

BASE_URL = 'http://localhost:8080'

def get_headers(token: str = None):
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def extract_series_uid(series: dict) -> str:
    """Series에서 SeriesInstanceUID 추출"""
    try:
        if "0020000E" in series:
            value_obj = series["0020000E"]
            if isinstance(value_obj, dict) and "Value" in value_obj:
                values = value_obj["Value"]
                if isinstance(values, list) and len(values) > 0:
                    return values[0]
    except:
        pass
    return None

def wait_for_series_sync(study_uid: str, series_uid: str, project_id: int, token: str, max_wait: int = 30) -> bool:
    """Series가 DICOM Gateway에서 조회 가능할 때까지 대기"""
    print(f"⏳ Series {series_uid} 동기화 대기 중... (최대 {max_wait * 0.5}초)")
    
    for i in range(max_wait):
        series_response = requests.get(
            f"{BASE_URL}/api/dicom/studies/{study_uid}/series",
            params={"project_id": project_id},
            headers=get_headers(token)
        )
        
        if series_response.status_code == 200:
            series_list = series_response.json()
            if isinstance(series_list, list):
                for series in series_list:
                    found_uid = extract_series_uid(series)
                    if found_uid == series_uid:
                        print(f"✅ Series {series_uid} 동기화 완료! ({i * 0.5:.1f}초 소요)")
                        return True
        
        if i % 4 == 0 and i > 0:  # 2초마다 진행 상황 출력
            print(f"   ... 대기 중 ({i * 0.5:.1f}초 경과)")
        
        time.sleep(0.5)
    
    print(f"❌ Series {series_uid} 동기화 실패 (최대 대기 시간 초과)")
    return False

def main():
    print("="*60)
    print("🔄 Dcm4chee 동기화 테스트")
    print("="*60)
    print("\n옵션:")
    print("1. 새 프로젝트 생성 및 Study 할당")
    print("2. 기존 프로젝트 사용 (프로젝트 ID 입력)")
    choice = input("\n선택 (1 또는 2, 기본값: 1): ").strip() or "1"
    
    use_existing_project = (choice == "2")
    existing_project_id = None
    if use_existing_project:
        project_id_str = input("프로젝트 ID를 입력하세요: ").strip()
        if project_id_str:
            try:
                existing_project_id = int(project_id_str)
            except ValueError:
                print("❌ 잘못된 프로젝트 ID")
                return
    
    # 사용자 생성 및 로그인
    user_data = {
        'username': f'testuser_sync_{int(time.time() * 1000)}',
        'email': f'test_sync_{int(time.time() * 1000)}@example.com',
        'password': 'TestPassword123!',
        'full_name': '동기화 테스트 사용자'
    }
    
    print("\n1. 사용자 생성 중...")
    response = requests.post(f'{BASE_URL}/api/auth/signup', json=user_data)
    if response.status_code not in [200, 201]:
        print(f"❌ 사용자 생성 실패: {response.status_code} - {response.text}")
        return
    
    result = response.json()
    user_id = result.get('user_id') or result.get('id')
    print(f"✅ 사용자 생성 완료: {user_id}")
    
    # 사용자 승인
    approve_response = requests.post(
        f'{BASE_URL}/api/auth/admin/users/approve',
        json={'user_id': user_id},
        headers=get_headers()
    )
    print(f"✅ 사용자 승인: {approve_response.status_code}")
    
    # 로그인
    print("\n2. 로그인 중...")
    login_response = requests.post(
        f'{BASE_URL}/api/auth/keycloak-token',
        json={'username': user_data['username'], 'password': user_data['password']},
        headers=get_headers()
    )
    if login_response.status_code != 200:
        print(f"❌ 로그인 실패: {login_response.text}")
        return
    
    token = login_response.json().get('access_token')
    print(f"✅ 로그인 완료")
    
    # 프로젝트 생성 또는 기존 프로젝트 사용
    if use_existing_project and existing_project_id:
        project_id = existing_project_id
        print(f"\n3. 기존 프로젝트 사용: {project_id}")
        
        # 프로젝트 확인
        project_check = requests.get(
            f'{BASE_URL}/api/projects/{project_id}',
            headers=get_headers(token)
        )
        if project_check.status_code != 200:
            print(f"❌ 프로젝트 {project_id}를 찾을 수 없습니다.")
            return
        print(f"✅ 프로젝트 확인 완료")
    else:
        print("\n3. 프로젝트 생성 중...")
        today = date.today()
        project_data = {
            'name': f'test_project_sync_{int(time.time())}',
            'description': '동기화 테스트 프로젝트',
            'sponsor': 'Test Sponsor',
            'start_date': str(today),
            'end_date': str(today + timedelta(days=365))
        }
        project_response = requests.post(
            f'{BASE_URL}/api/projects',
            json=project_data,
            headers=get_headers(token)
        )
        if project_response.status_code not in [200, 201]:
            print(f"❌ 프로젝트 생성 실패: {project_response.text}")
            return
        
        project_id = project_response.json().get('id') or project_response.json().get('project_id')
        print(f"✅ 프로젝트 생성 완료: {project_id}")
    
    # 사용자를 프로젝트에 추가
    print("\n4. 사용자를 프로젝트에 추가 중...")
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
            f'{BASE_URL}/api/projects/{project_id}/members',
            json={'user_id': user_id, 'role_id': role_id},
            headers=get_headers(token)
        )
        print(f"✅ 사용자 추가: {member_response.status_code}")
    
    # Dcm4chee에서 실제 Study 조회
    print("\n5. Dcm4chee에서 Study 조회 중...")
    print("   (프로젝트에 할당된 Study가 있으면 사용, 없으면 전체 조회 시도)")
    
    # 먼저 프로젝트에 할당된 Study 조회
    studies_response = requests.get(
        f'{BASE_URL}/api/dicom/studies',
        params={'project_id': project_id, 'limit': 10},
        headers=get_headers(token)
    )
    
    studies = []
    if studies_response.status_code == 200:
        studies = studies_response.json()
        if not isinstance(studies, list):
            studies = []
        print(f"   프로젝트에 할당된 Study: {len(studies)}개")
    
    # 프로젝트에 할당된 Study가 없으면, 전체 접근 권한으로 조회 시도
    if len(studies) == 0:
        print("   프로젝트에 할당된 Study 없음. 전체 접근 권한으로 조회 시도...")
        studies_response_all = requests.get(
            f'{BASE_URL}/api/dicom/studies',
            params={'limit': 10},
            headers=get_headers(token)
        )
        
        if studies_response_all.status_code == 200:
            studies_all = studies_response_all.json()
            if isinstance(studies_all, list):
                studies = studies_all
                print(f"   Dcm4chee에서 조회된 Study: {len(studies)}개")
            else:
                print(f"   ⚠️  예상치 못한 응답 형식")
        elif studies_response_all.status_code == 400:
            print(f"   ⚠️  전체 접근 권한 없음. 프로젝트에 Study를 먼저 할당해야 합니다.")
        else:
            print(f"   ❌ Study 조회 실패: {studies_response_all.status_code}")
    
    if len(studies) == 0:
        print("\n❌ 사용 가능한 Study가 없습니다.")
        print("   해결 방법:")
        print("   1. Dcm4chee에 실제 DICOM 데이터가 있는지 확인")
        print("   2. 프로젝트에 Study를 먼저 할당")
        print("   3. 전체 접근 권한이 있는 사용자로 테스트")
        return
    
    # 첫 번째 Study 사용
    study = studies[0]
    study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
    if not study_uid:
        print("❌ Study UID를 찾을 수 없습니다.")
        return
    
    print(f"✅ Study 선택: {study_uid}")
    
    # Study를 프로젝트에 할당
    print("\n6. Study를 프로젝트에 할당 중...")
    study_data = {
        'study_uid': study_uid,
        'study_description': study.get("00081030", {}).get("Value", [None])[0] if "00081030" in study else None,
        'patient_id': study.get("00100020", {}).get("Value", [None])[0] if "00100020" in study else None,
        'patient_name': study.get("00100010", {}).get("Value", [None])[0] if "00100010" in study else None,
        'study_date': None
    }
    study_response = requests.post(
        f'{BASE_URL}/api/projects/{project_id}/studies/assign',
        json=study_data,
        headers=get_headers(token)
    )
    if study_response.status_code not in [200, 201]:
        print(f"⚠️  Study 할당 실패 (이미 할당되어 있을 수 있음): {study_response.status_code}")
    else:
        study_result = study_response.json()
        study_id = study_result.get('study_id') or study_result.get('data', {}).get('study', {}).get('id')
        print(f"✅ Study 할당 완료: {study_id}")
    
    # 해당 Study의 Series 조회
    print("\n7. Study의 Series 조회 중...")
    series_response = requests.get(
        f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
        params={'project_id': project_id, 'limit': 10},
        headers=get_headers(token)
    )
    
    if series_response.status_code != 200:
        print(f"❌ Series 조회 실패: {series_response.status_code} - {series_response.text}")
        return
    
    series_list_from_dcm4chee = series_response.json()
    if not isinstance(series_list_from_dcm4chee, list) or len(series_list_from_dcm4chee) == 0:
        print(f"❌ Series가 없습니다.")
        return
    
    print(f"✅ {len(series_list_from_dcm4chee)}개의 Series 조회됨")
    
    # 첫 번째 Series 사용
    series = series_list_from_dcm4chee[0]
    series_uid = series.get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series else None
    
    if not series_uid:
        print("❌ Series UID를 찾을 수 없습니다.")
        return
    
    print(f"✅ Series 선택: {series_uid}")
    
    # Series를 프로젝트에 할당
    print("\n8. Series를 프로젝트에 할당 중...")
    series_data = {
        'study_uid': study_uid,
        'series_uid': series_uid,
        'series_description': series.get("0008103E", {}).get("Value", [None])[0] if "0008103E" in series else None,
        'modality': series.get("00080060", {}).get("Value", [None])[0] if "00080060" in series else "CT",
        'series_number': series.get("00200011", {}).get("Value", [None])[0] if "00200011" in series else 1
    }
    series_response_assign = requests.post(
        f'{BASE_URL}/api/projects/{project_id}/series/assign',
        json=series_data,
        headers=get_headers(token)
    )
    
    if series_response_assign.status_code not in [200, 201]:
        print(f"⚠️  Series 할당 실패 (이미 할당되어 있을 수 있음): {series_response_assign.status_code}")
    else:
        series_result = series_response_assign.json()
        series_id = series_result.get('series_id') or series_result.get('data', {}).get('series', {}).get('id')
        print(f"✅ Series 할당 완료: {series_id}")
    
    # 이미 Dcm4chee에 존재하는 Series이므로 동기화 확인만 수행
    print("\n" + "="*60)
    print("9. 동기화 확인 (이미 Dcm4chee에 존재하는 데이터)")
    print("="*60)
    
    # 동기화 확인: DICOM Gateway에서 조회
    print("\n동기화 확인: DICOM Gateway에서 Series 조회...")
    final_response = requests.get(
        f'{BASE_URL}/api/dicom/studies/{study_uid}/series',
        params={'project_id': project_id},
        headers=get_headers(token)
    )
    
    if final_response.status_code == 200:
        series_list = final_response.json()
        if isinstance(series_list, list):
            print(f"✅ DICOM Gateway에서 {len(series_list)}개의 Series 조회됨")
            found = False
            for s in series_list:
                found_uid = extract_series_uid(s)
                if found_uid == series_uid:
                    found = True
                    print(f"   ✅ 대상 Series 발견: {found_uid}")
                    break
                elif found_uid:
                    print(f"   - Series UID: {found_uid}")
            
            if not found:
                print(f"   ⚠️  대상 Series ({series_uid})를 찾을 수 없습니다.")
                print("   가능한 원인:")
                print("   1. RBAC 필터링으로 인해 접근 불가")
                print("   2. 프로젝트에 할당되지 않음")
        else:
            print(f"⚠️  예상치 못한 응답 형식: {type(series_list)}")
    else:
        print(f"❌ DICOM Gateway 조회 실패: {final_response.status_code} - {final_response.text}")

if __name__ == '__main__':
    main()

