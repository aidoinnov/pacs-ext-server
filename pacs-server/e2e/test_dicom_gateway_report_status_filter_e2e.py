#!/usr/bin/env python3
"""
DICOM Gateway Report Status 필터링 E2E 시나리오 테스트 스크립트

이 스크립트는 DICOM Gateway Series API의 report_status 필터링 기능을 테스트합니다:
1. 사용자, 프로젝트, Study, Series 생성
2. 각 Series에 대해 다양한 status의 Report 생성 (approved, unread, unapproval)
3. report_status 파라미터로 필터링 테스트
4. Project-dependent와 global report 모두 테스트
5. 여러 status 조합 테스트
6. Report가 없는 Series는 필터링에서 제외되는지 확인
"""

import requests
import json
import time
import sys
from typing import Optional, Dict, Any, List

BASE_URL = "http://localhost:8080"

# 테스트 결과 추적
test_results = {
    "passed": 0,
    "failed": 0,
    "total": 0
}

def print_test(test_name: str):
    """테스트 시작 출력"""
    print(f"\n{'='*60}")
    print(f"🧪 {test_name}")
    print(f"{'='*60}")

def print_success(message: str):
    """성공 메시지 출력"""
    print(f"✅ {message}")
    test_results["passed"] += 1
    test_results["total"] += 1

def print_error(message: str):
    """에러 메시지 출력"""
    print(f"❌ {message}")
    test_results["failed"] += 1
    test_results["total"] += 1

def print_info(message: str):
    """정보 메시지 출력"""
    print(f"ℹ️  {message}")

def test_health():
    """헬스 체크 테스트"""
    print_test("Health Check")
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        if response.status_code == 200:
            print_success(f"Server is healthy: {response.json()}")
            return True
        else:
            print_error(f"Health check failed: {response.status_code}")
            return False
    except Exception as e:
        print_error(f"Health check error: {e}")
        return False

def approve_user(user_id: int) -> bool:
    """사용자 승인"""
    try:
        approve_data = {
            "user_id": user_id
        }
        response = requests.post(
            f"{BASE_URL}/api/auth/admin/users/approve",
            json=approve_data,
            headers=get_headers()
        )
        return response.status_code in [200, 201]
    except:
        return False

def login_user(user_data: Dict[str, Any]) -> Optional[str]:
    """사용자 로그인하여 JWT 토큰 얻기"""
    print_info(f"Logging in user: {user_data.get('username')}...")
    try:
        login_data = {
            "username": user_data.get("username", ""),
            "password": user_data.get("password", "TestPassword123!")
        }
        
        response = requests.post(
            f"{BASE_URL}/api/auth/keycloak-token",
            json=login_data,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            result = response.json()
            token = result.get("access_token")
            if token:
                print_success(f"Login successful for user: {user_data.get('username')}")
                return token
            else:
                print_error("Token not found in keycloak-token response")
                return None
        else:
            print_error(f"Login failed: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Login error: {e}")
        return None

def get_headers(token: Optional[str] = None) -> Dict[str, str]:
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def create_test_project(token: str) -> Optional[int]:
    """테스트 프로젝트 생성"""
    print_info("Creating test project...")
    from datetime import date, timedelta
    today = date.today()
    project_data = {
        "name": f"test_project_report_filter_{int(time.time())}",
        "description": "Report Status Filter 테스트용 프로젝트",
        "sponsor": "Test Sponsor",
        "start_date": str(today),
        "end_date": str(today + timedelta(days=365))
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/projects",
            json=project_data,
            headers=get_headers(token)
        )
        if response.status_code in [200, 201]:
            project = response.json()
            project_id = project.get("id") or project.get("project_id")
            print_success(f"Project created: {project_id}")
            return project_id
        else:
            print_error(f"Project creation failed: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Project creation error: {e}")
        return None

def create_test_user() -> Optional[Dict[str, Any]]:
    """테스트 사용자 생성 및 로그인"""
    print_info("Creating test user...")
    import uuid
    timestamp = int(time.time() * 1000)
    username = f"testuser_report_{timestamp}"
    email = f"test_report_{timestamp}@example.com"
    password = "TestPassword123!"
    keycloak_id = str(uuid.uuid4())
    
    user_data = {
        "username": username,
        "email": email,
        "password": password,
        "full_name": "테스트 사용자"
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/auth/signup",
            json=user_data,
            headers=get_headers()
        )
        if response.status_code in [200, 201]:
            signup_result = response.json()
            user_id = signup_result.get("user_id") or signup_result.get("id")
            print_success(f"User created: {user_id} ({username})")
            
            if approve_user(user_id):
                print_success(f"User {user_id} approved")
            
            token = login_user({
                "username": username,
                "email": email,
                "password": password,
                "keycloak_id": keycloak_id
            })
            
            user_data["id"] = user_id
            user_data["token"] = token
            user_data["keycloak_id"] = keycloak_id
            return user_data
        else:
            print_error(f"User creation failed: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"User creation error: {e}")
        return None

def get_role_id(token: str, role_name: str = "RESEARCHER") -> Optional[int]:
    """역할 ID 조회"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/roles",
            headers=get_headers(token)
        )
        if response.status_code == 200:
            roles = response.json()
            if isinstance(roles, list):
                for role in roles:
                    if role.get("name") == role_name or role.get("role_name") == role_name:
                        return role.get("id") or role.get("role_id")
            elif isinstance(roles, dict) and "roles" in roles:
                for role in roles["roles"]:
                    if role.get("name") == role_name or role.get("role_name") == role_name:
                        return role.get("id") or role.get("role_id")
        return None
    except:
        return None

def add_user_to_project(user_id: int, project_id: int, token: str, role_id: Optional[int] = None) -> bool:
    """사용자를 프로젝트에 추가"""
    print_info(f"Adding user {user_id} to project {project_id}...")
    try:
        if role_id is None:
            role_id = get_role_id(token)
        
        member_data = {
            "user_id": user_id,
            "role_id": role_id
        }
        response = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/members",
            json=member_data,
            headers=get_headers(token)
        )
        if response.status_code in [200, 201]:
            print_success(f"User {user_id} added to project {project_id}")
            return True
        else:
            print_error(f"Failed to add user to project: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Error adding user to project: {e}")
        return False

def wait_for_series_sync(study_uid: str, series_uid: str, project_id: int, token: str, max_wait: int = 30) -> bool:
    """Series가 DICOM Gateway에서 조회 가능할 때까지 대기 (Dcm4chee 동기화 대기)"""
    print_info(f"Waiting up to {max_wait * 0.5} seconds for Dcm4chee sync...")
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
                        print_success(f"Series {series_uid} found in Dcm4chee after {i * 0.5:.1f} seconds")
                        return True
        
        if i % 4 == 0 and i > 0:  # 2초마다 진행 상황 출력
            print_info(f"Still waiting... ({i * 0.5:.1f}s)")
        
        time.sleep(0.5)
    
    print_error(f"Series {series_uid} not found in Dcm4chee after {max_wait * 0.5} seconds")
    return False

def get_existing_studies_and_series(project_id: int, token: str, count: int = 5) -> List[Dict[str, Any]]:
    """Dcm4chee에서 실제 Study/Series 조회 및 프로젝트에 할당"""
    print_info(f"Fetching {count} existing studies and series from Dcm4chee...")
    series_list = []
    
    try:
        # 1. 먼저 프로젝트에 할당된 Study 목록 조회 시도
        studies_response = requests.get(
            f"{BASE_URL}/api/dicom/studies",
            params={"project_id": project_id, "limit": count * 3},
            headers=get_headers(token)
        )
        
        studies = []
        if studies_response.status_code == 200:
            studies = studies_response.json()
            if not isinstance(studies, list):
                studies = []
            print_info(f"Found {len(studies)} studies already assigned to project")
        
        # 2. 프로젝트에 할당된 Study가 없으면, Study를 먼저 할당한 후 동기화 대기
        if len(studies) == 0:
            print_info("No studies in project. Creating test studies and waiting for Dcm4chee sync...")
            
            # 테스트용 Study/Series 생성 및 할당
            for i in range(count):
                study_uid = f"1.2.840.113619.2.1.1.{int(time.time())}.{i}"
                study_data = {
                    "study_uid": study_uid,
                    "study_description": f"Test Study {i} for Report Filter",
                    "patient_id": f"TEST{i:03d}",
                    "patient_name": f"Test Patient {i}",
                    "study_date": None
                }
                
                study_response = requests.post(
                    f"{BASE_URL}/api/projects/{project_id}/studies/assign",
                    json=study_data,
                    headers=get_headers(token)
                )
                
                if study_response.status_code not in [200, 201]:
                    print_error(f"Failed to assign study {i}: {study_response.status_code}")
                    continue
                
                study_result = study_response.json()
                study_id = study_result.get("study_id") or study_result.get("data", {}).get("study", {}).get("id")
                
                if not study_id:
                    print_error(f"Study ID not found in response")
                    continue
                
                # Series 할당
                series_uid = f"1.2.840.113619.2.1.2.{int(time.time())}.{i}"
                series_data = {
                    "study_uid": study_uid,
                    "series_uid": series_uid,
                    "series_description": f"Test Series {i} for Report Filter",
                    "modality": "CT",
                    "series_number": i + 1
                }
                
                series_response = requests.post(
                    f"{BASE_URL}/api/projects/{project_id}/series/assign",
                    json=series_data,
                    headers=get_headers(token)
                )
                
                if series_response.status_code in [200, 201]:
                    series_result = series_response.json()
                    series_id = series_result.get("series_id") or series_result.get("data", {}).get("series", {}).get("id")
                    if series_id:
                        # Dcm4chee 동기화 대기 (더 긴 대기 시간)
                        print_info(f"Waiting for series {series_uid} to sync with Dcm4chee...")
                        if wait_for_series_sync(study_uid, series_uid, project_id, token, max_wait=40):
                            series_list.append({
                                "series_id": series_id,
                                "series_uid": series_uid,
                                "study_uid": study_uid,
                                "study_id": study_id
                            })
                            print_success(f"Test Series {i} synced: {series_id} (UID: {series_uid})")
                        else:
                            print_error(f"Series {series_uid} not found in Dcm4chee after sync wait")
            
            # 할당한 Study를 다시 조회
            if len(series_list) > 0:
                studies_response = requests.get(
                    f"{BASE_URL}/api/dicom/studies",
                    params={"project_id": project_id, "limit": count * 3},
                    headers=get_headers(token)
                )
                
                if studies_response.status_code == 200:
                    studies = studies_response.json()
                    if not isinstance(studies, list):
                        studies = []
                    print_info(f"Found {len(studies)} studies after assignment")
        
        if len(studies) == 0 and len(series_list) == 0:
            print_error("No studies found. Please ensure Dcm4chee has data or wait for sync.")
            return []
        
        # 2. 각 Study의 Series를 DICOM Gateway를 통해 조회 (Dcm4chee에 실제 존재하는 데이터)
        for i, study in enumerate(studies):
            if len(series_list) >= count:
                break
                
            study_uid = study.get("0020000D", {}).get("Value", [None])[0] if "0020000D" in study else None
            if not study_uid:
                print_error(f"Study {i} has no StudyInstanceUID")
                continue
            
            # 해당 Study의 Series를 DICOM Gateway를 통해 조회 (Dcm4chee에서 직접)
            series_response = requests.get(
                f"{BASE_URL}/api/dicom/studies/{study_uid}/series",
                params={"project_id": project_id, "limit": 10},
                headers=get_headers(token)
            )
            
            if series_response.status_code != 200:
                print_error(f"Failed to fetch series for study {study_uid}: {series_response.status_code}")
                continue
            
            series_list_from_dcm4chee = series_response.json()
            if not isinstance(series_list_from_dcm4chee, list) or len(series_list_from_dcm4chee) == 0:
                print_error(f"No series found for study {study_uid} in Dcm4chee")
                continue
            
            print_info(f"Found {len(series_list_from_dcm4chee)} series in study {study_uid}")
            
            # 각 Series를 프로젝트에 할당하고 DB에서 series_id 조회
            for series in series_list_from_dcm4chee:
                if len(series_list) >= count:
                    break
                    
                series_uid = series.get("0020000E", {}).get("Value", [None])[0] if "0020000E" in series else None
                
                if not series_uid:
                    print_error(f"Series has no SeriesInstanceUID")
                    continue
                
                # Series를 프로젝트에 할당 시도 (이미 할당되어 있을 수 있음)
                series_data = {
                    "study_uid": study_uid,
                    "series_uid": series_uid,
                    "series_description": series.get("0008103E", {}).get("Value", [None])[0] if "0008103E" in series else None,
                    "modality": series.get("00080060", {}).get("Value", [None])[0] if "00080060" in series else "CT",
                    "series_number": series.get("00200011", {}).get("Value", [None])[0] if "00200011" in series else len(series_list) + 1
                }
                
                series_assign_response = requests.post(
                    f"{BASE_URL}/api/projects/{project_id}/series/assign",
                    json=series_data,
                    headers=get_headers(token)
                )
                
                if series_assign_response.status_code in [200, 201]:
                    series_result = series_assign_response.json()
                    series_id = series_result.get("series_id") or series_result.get("data", {}).get("series", {}).get("id")
                    
                    if series_id:
                        # Study ID 조회
                        study_assign_response = requests.post(
                            f"{BASE_URL}/api/projects/{project_id}/studies/assign",
                            json={
                                "study_uid": study_uid,
                                "study_description": study.get("00081030", {}).get("Value", [None])[0] if "00081030" in study else None,
                                "patient_id": study.get("00100020", {}).get("Value", [None])[0] if "00100020" in study else None,
                                "patient_name": study.get("00100010", {}).get("Value", [None])[0] if "00100010" in study else None,
                                "study_date": None
                            },
                            headers=get_headers(token)
                        )
                        study_id = None
                        if study_assign_response.status_code in [200, 201]:
                            study_result = study_assign_response.json()
                            study_id = study_result.get("study_id") or study_result.get("data", {}).get("study", {}).get("id")
                        
                        # 동기화 대기 (DICOM Gateway에서 조회 가능할 때까지)
                        print_info(f"Waiting for series {series_uid} to sync...")
                        if wait_for_series_sync(study_uid, series_uid, project_id, token):
                            series_list.append({
                                "series_id": series_id,
                                "series_uid": series_uid,
                                "study_uid": study_uid,
                                "study_id": study_id
                            })
                            print_success(f"Series {len(series_list)} ready: {series_id} (UID: {series_uid})")
                        else:
                            print_error(f"Series {series_uid} not found in DICOM Gateway after sync wait")
                else:
                    print_error(f"Failed to assign series {series_uid}: {series_assign_response.status_code}")
        
        return series_list
    except Exception as e:
        print_error(f"Error fetching studies/series: {e}")
        import traceback
        traceback.print_exc()
        return []

def create_report(series_id: int, user_id: int, token: str, project_id: Optional[int] = None, 
                  status: str = "unread", description: str = "Test description", 
                  conclusion: str = "Test conclusion") -> Optional[Dict[str, Any]]:
    # DB 스키마에 맞게 status 변환: 'approved' -> 'approval'
    if status == "approved":
        status = "approval"
    """Report 생성"""
    try:
        report_data = {
            "status": status,
            "description": description,
            "conclusion": conclusion
        }
        
        if project_id:
            url = f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report"
        else:
            url = f"{BASE_URL}/api/series/{series_id}/report"
        
        response = requests.put(
            url,
            json=report_data,
            headers=get_headers(token)
        )
        
        if response.status_code in [200, 201]:
            result = response.json()
            report = result.get("report") or result
            print_success(f"Report created for series {series_id} with status '{status}'")
            return report
        else:
            print_error(f"Failed to create report: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Error creating report: {e}")
        return None

def get_series_from_gateway(study_uid: str, token: str, project_id: Optional[int] = None, 
                           report_status: Optional[str] = None) -> Optional[List[Dict[str, Any]]]:
    """DICOM Gateway에서 Series 조회"""
    try:
        params = {}
        if project_id:
            params["project_id"] = project_id
        if report_status:
            params["report_status"] = report_status
        
        url = f"{BASE_URL}/api/dicom/studies/{study_uid}/series"
        response = requests.get(
            url,
            params=params,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            series_list = response.json()
            if isinstance(series_list, list):
                return series_list
            else:
                print_error(f"Unexpected response format: {series_list}")
                return None
        else:
            print_error(f"Failed to get series: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Error getting series from gateway: {e}")
        return None

def extract_series_uid(series: Dict[str, Any]) -> Optional[str]:
    """Series에서 SeriesInstanceUID 추출"""
    try:
        # QIDO-RS 형식: {"0020000E": {"Value": ["1.2.3.4"], "vr": "UI"}}
        if "0020000E" in series:
            value_obj = series["0020000E"]
            if isinstance(value_obj, dict) and "Value" in value_obj:
                values = value_obj["Value"]
                if isinstance(values, list) and len(values) > 0:
                    return values[0]
        return None
    except:
        return None

def scenario_1_single_status_filter():
    """시나리오 1: 단일 status 필터링 (approval)"""
    print_test("시나리오 1: 단일 status 필터링 (approval)")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    # Dcm4chee에서 실제 Study와 Series 조회 및 할당
    series_list = get_existing_studies_and_series(project_id, token, count=5)
    if len(series_list) < 3:
        print_error(f"Failed to create enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    # Report 생성: 첫 번째는 approval, 두 번째는 unread, 세 번째는 unapproval
    create_report(series_list[0]["series_id"], user_id, token, project_id, status="approval")
    create_report(series_list[1]["series_id"], user_id, token, project_id, status="unread")
    create_report(series_list[2]["series_id"], user_id, token, project_id, status="unapproval")
    # 네 번째, 다섯 번째는 Report 없음
    
    time.sleep(1.0)  # DB 동기화 대기 (Dcm4chee 동기화 포함)
    
    # approval만 필터링
    filtered_series = get_series_from_gateway(study_uid, token, project_id, report_status="approval")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    # 결과 검증
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uid = series_list[0]["series_uid"]
    
    if expected_uid in series_uids:
        print_success(f"Filtered series contains expected UID: {expected_uid}")
        if len(series_uids) == 1:
            print_success("Only one series returned (correct filtering)")
        else:
            print_error(f"Expected 1 series, got {len(series_uids)}")
            return False
    else:
        print_error(f"Expected UID {expected_uid} not found in filtered results")
        return False
    
    return True

def scenario_2_multiple_status_filter():
    """시나리오 2: 다중 status 필터링 (approval,unread)"""
    print_test("시나리오 2: 다중 status 필터링 (approval,unread)")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    series_list = get_existing_studies_and_series(project_id, token, count=5)
    if len(series_list) < 3:
        print_error(f"Failed to create enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    # Report 생성
    create_report(series_list[0]["series_id"], user_id, token, project_id, status="approval")
    create_report(series_list[1]["series_id"], user_id, token, project_id, status="unread")
    create_report(series_list[2]["series_id"], user_id, token, project_id, status="unapproval")
    
    time.sleep(0.5)
    
    # approval,unread 필터링
    filtered_series = get_series_from_gateway(study_uid, token, project_id, report_status="approval,unread")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uids = [series_list[0]["series_uid"], series_list[1]["series_uid"]]
    
    found_count = sum(1 for uid in expected_uids if uid in series_uids)
    if found_count == 2:
        print_success(f"Both expected UIDs found in filtered results")
        if len(series_uids) == 2:
            print_success("Exactly 2 series returned (correct filtering)")
        else:
            print_error(f"Expected 2 series, got {len(series_uids)}")
            return False
    else:
        print_error(f"Expected 2 UIDs, found {found_count}")
        return False
    
    return True

def scenario_3_no_report_excluded():
    """시나리오 3: Report가 없는 Series는 필터링에서 제외"""
    print_test("시나리오 3: Report가 없는 Series는 필터링에서 제외")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    series_list = get_existing_studies_and_series(project_id, token, count=3)
    if len(series_list) < 3:
        print_error(f"Failed to create enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    # 첫 번째만 Report 생성
    create_report(series_list[0]["series_id"], user_id, token, project_id, status="approval")
    # 두 번째, 세 번째는 Report 없음
    
    time.sleep(0.5)
    
    # approval 필터링
    filtered_series = get_series_from_gateway(study_uid, token, project_id, report_status="approval")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uid = series_list[0]["series_uid"]
    
    if expected_uid in series_uids:
        print_success(f"Only series with report is included")
        if len(series_uids) == 1:
            print_success("Report가 없는 Series는 제외됨 (correct)")
        else:
            print_error(f"Expected 1 series, got {len(series_uids)}")
            return False
    else:
        print_error(f"Expected UID {expected_uid} not found")
        return False
    
    return True

def scenario_4_global_vs_project_dependent():
    """시나리오 4: Global report vs Project-dependent report 우선순위"""
    print_test("시나리오 4: Global report vs Project-dependent report 우선순위")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    series_list = get_existing_studies_and_series(project_id, token, count=2)
    if len(series_list) < 1:
        print_error(f"Failed to get enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    # Global report 생성 (unread) - project_id 없이
    create_report(series_list[0]["series_id"], user_id, token, project_id=None, status="unread")
    # Project-dependent report 생성 (approval) - 우선순위가 높아야 함
    create_report(series_list[0]["series_id"], user_id, token, project_id=project_id, status="approval")
    
    time.sleep(0.5)
    
    # approval 필터링 (project-dependent가 우선되어야 함)
    filtered_series = get_series_from_gateway(study_uid, token, project_id, report_status="approval")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uid = series_list[0]["series_uid"]
    
    if expected_uid in series_uids:
        print_success("Project-dependent report가 우선되어 필터링됨")
        return True
    else:
        print_error(f"Expected UID {expected_uid} not found (project-dependent should take priority)")
        return False

def scenario_5_all_status_values():
    """시나리오 5: 모든 status 값 필터링"""
    print_test("시나리오 5: 모든 status 값 필터링")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    series_list = get_existing_studies_and_series(project_id, token, count=4)
    if len(series_list) < 4:
        print_error(f"Failed to create enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    # 모든 status 값 생성
    create_report(series_list[0]["series_id"], user_id, token, project_id, status="approval")
    create_report(series_list[1]["series_id"], user_id, token, project_id, status="unread")
    create_report(series_list[2]["series_id"], user_id, token, project_id, status="unapproval")
    # 네 번째는 Report 없음
    
    time.sleep(0.5)
    
    # 모든 status 필터링
    filtered_series = get_series_from_gateway(study_uid, token, project_id, 
                                               report_status="approval,unread,unapproval")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uids = [series_list[0]["series_uid"], series_list[1]["series_uid"], series_list[2]["series_uid"]]
    
    found_count = sum(1 for uid in expected_uids if uid in series_uids)
    if found_count == 3:
        print_success("All three status values are correctly filtered")
        if len(series_uids) == 3:
            print_success("Exactly 3 series returned (correct filtering)")
        else:
            print_error(f"Expected 3 series, got {len(series_uids)}")
            return False
    else:
        print_error(f"Expected 3 UIDs, found {found_count}")
        return False
    
    return True

def scenario_6_case_insensitive_filter():
    """시나리오 6: 대소문자 무시 필터링"""
    print_test("시나리오 6: 대소문자 무시 필터링")
    
    user = create_test_user()
    if not user or not user.get("token"):
        print_error("Failed to create test user")
        return False
    
    token = user["token"]
    user_id = user["id"]
    
    project_id = create_test_project(token)
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id, token):
        print_error("Failed to add user to project")
        return False
    
    series_list = get_existing_studies_and_series(project_id, token, count=2)
    if len(series_list) < 2:
        print_error(f"Failed to create enough series (got {len(series_list)})")
        return False
    
    study_uid = series_list[0]["study_uid"]
    
    create_report(series_list[0]["series_id"], user_id, token, project_id, status="approval")
    
    time.sleep(0.5)
    
    # 대문자로 필터링
    filtered_series = get_series_from_gateway(study_uid, token, project_id, report_status="APPROVAL")
    if filtered_series is None:
        print_error("Failed to get filtered series")
        return False
    
    series_uids = [extract_series_uid(s) for s in filtered_series if extract_series_uid(s)]
    expected_uid = series_list[0]["series_uid"]
    
    if expected_uid in series_uids:
        print_success("Case-insensitive filtering works correctly")
        return True
    else:
        print_error(f"Case-insensitive filtering failed")
        return False

def main():
    """메인 테스트 실행"""
    print("\n" + "="*60)
    print("🚀 DICOM Gateway Report Status 필터링 E2E 테스트 시작")
    print("="*60)
    print("\n⚠️  중요: 이 테스트는 실제 Dcm4chee에 DICOM 데이터가 있어야 합니다.")
    print("   Dcm4chee에 데이터가 없으면 테스트가 실패할 수 있습니다.")
    print("   테스트는 DB에 Study/Series를 할당하지만, DICOM Gateway는")
    print("   Dcm4chee의 QIDO-RS API를 통해 실제 DICOM 데이터를 조회합니다.\n")
    
    # 헬스 체크
    if not test_health():
        print_error("Server is not available. Please start the server first.")
        sys.exit(1)
    
    # 시나리오 실행
    scenarios = [
        scenario_1_single_status_filter,
        scenario_2_multiple_status_filter,
        scenario_3_no_report_excluded,
        scenario_4_global_vs_project_dependent,
        scenario_5_all_status_values,
        scenario_6_case_insensitive_filter,
    ]
    
    for scenario in scenarios:
        try:
            scenario()
        except Exception as e:
            print_error(f"Scenario failed with exception: {e}")
            import traceback
            traceback.print_exc()
    
    # 결과 출력
    print("\n" + "="*60)
    print("📊 테스트 결과 요약")
    print("="*60)
    print(f"총 테스트: {test_results['total']}")
    print(f"✅ 통과: {test_results['passed']}")
    print(f"❌ 실패: {test_results['failed']}")
    print("="*60)
    
    if test_results['failed'] > 0:
        sys.exit(1)
    else:
        print("\n🎉 모든 테스트가 통과했습니다!")
        sys.exit(0)

if __name__ == "__main__":
    main()

