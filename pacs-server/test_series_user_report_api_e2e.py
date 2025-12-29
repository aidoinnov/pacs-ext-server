#!/usr/bin/env python3
"""
Series User Report API 시나리오 테스트 스크립트

이 스크립트는 Series User Report API의 다양한 시나리오를 테스트합니다:
1. 프로젝트 종속 Report 생성/조회/수정/삭제
2. 전역 Report 생성/조회/수정/삭제
3. 여러 사용자가 같은 Series에 Report 작성
4. 프로젝트별 Report와 전역 Report 분리
5. 권한 검증 (프로젝트 멤버가 아닌 경우)
6. Report Status 변경 (unread, approval, unapproval)
7. Report 업데이트 (UPSERT 동작)
"""

import requests
import json
import time
import sys
from typing import Optional, Dict, Any

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

def create_test_project() -> Optional[int]:
    """테스트 프로젝트 생성"""
    print_info("Creating test project...")
    from datetime import date, timedelta
    import uuid
    today = date.today()
    # 더 고유한 이름 생성 (타임스탬프 + UUID 일부)
    unique_id = f"{int(time.time() * 1000)}_{str(uuid.uuid4())[:8]}"
    project_data = {
        "name": f"test_project_report_{unique_id}",
        "description": "Series Report API 테스트용 프로젝트",
        "sponsor": "Test Sponsor",
        "start_date": str(today),
        "end_date": str(today + timedelta(days=365))
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/projects",
            json=project_data,
            headers=get_headers()
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

def add_user_to_project(user_id: int, project_id: int, role_id: Optional[int] = None) -> bool:
    """사용자를 프로젝트에 추가"""
    print_info(f"Adding user {user_id} to project {project_id}...")
    try:
        member_data = {
            "user_id": user_id,
            "role_id": role_id
        }
        response = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/members",
            json=member_data,
            headers=get_headers()
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

def create_test_series(project_id: int) -> Optional[int]:
    """테스트 Series 생성"""
    print_info("Creating test series...")
    try:
        study_uid = f"1.2.840.113619.2.1.1.{int(time.time())}"
        study_data = {
            "study_uid": study_uid,
            "study_description": "Test Study for Report API",
            "patient_id": "TEST001",
            "patient_name": "Test Patient",
            "study_date": None
        }
        
        study_response = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/studies/assign",
            json=study_data,
            headers=get_headers()
        )
        
        if study_response.status_code not in [200, 201]:
            print_error(f"Failed to create study: {study_response.status_code} - {study_response.text}")
            return None
        
        study_result = study_response.json()
        study_id = study_result.get("study_id") or study_result.get("data", {}).get("study", {}).get("id")
        
        series_uid = f"1.2.840.113619.2.1.2.{int(time.time())}"
        series_data = {
            "study_uid": study_uid,
            "series_uid": series_uid,
            "series_description": "Test Series for Report API",
            "modality": "CT",
            "series_number": 1
        }
        
        series_response = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/series/assign",
            json=series_data,
            headers=get_headers()
        )
        
        if series_response.status_code in [200, 201]:
            series_result = series_response.json()
            series_id = series_result.get("series_id") or series_result.get("data", {}).get("series", {}).get("id")
            if series_id:
                print_success(f"Series created: {series_id}")
                return series_id
            else:
                print_error(f"Series ID not found in response: {series_result}")
                return None
        else:
            print_error(f"Failed to create series: {series_response.status_code} - {series_response.text}")
            return None
    except Exception as e:
        print_error(f"Error creating series: {e}")
        return None

def scenario_1_project_report_crud():
    """시나리오 1: 프로젝트 종속 Report CRUD"""
    print_test("시나리오 1: 프로젝트 종속 Report CRUD")
    
    user = create_test_user()
    if not user:
        print_error("Failed to create test user")
        return False
    
    user_id = user.get("id")
    token = user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id):
        print_error("Failed to add user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. Report 생성
    print_info("Step 1: Creating project report...")
    report_data = {
        "status": "unread",
        "description": "이 시리즈는 프로젝트 A에서 분석 중입니다",
        "conclusion": "추가 검사 필요",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report:
                print_success(f"Report created: {report.get('id')}")
                report_id = report.get("id")
            else:
                print_error("Report not found in response")
                return False
        else:
            print_error(f"Report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report creation error: {e}")
        return False
    
    # 2. Report 조회
    print_info("Step 2: Retrieving project report...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == report_data["description"]:
                if report.get("status") == report_data["status"]:
                    print_success("Report retrieved successfully")
                else:
                    print_error(f"Status mismatch: expected {report_data['status']}, got {report.get('status')}")
                    return False
            else:
                print_error("Report content mismatch")
                return False
        else:
            print_error(f"Report retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report retrieval error: {e}")
        return False
    
    # 3. Report 수정
    print_info("Step 3: Updating project report...")
    updated_report_data = {
        "status": "approval",
        "description": "업데이트된 설명: 분석 완료",
        "conclusion": "정상 소견",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=updated_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == updated_report_data["description"]:
                if report.get("status") == updated_report_data["status"]:
                    print_success("Report updated successfully")
                else:
                    print_error(f"Status mismatch after update: expected {updated_report_data['status']}, got {report.get('status')}")
                    return False
            else:
                print_error("Report update content mismatch")
                return False
        else:
            print_error(f"Report update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report update error: {e}")
        return False
    
    # 4. Report 삭제
    print_info("Step 4: Deleting project report...")
    try:
        response = requests.delete(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Report deleted successfully")
        else:
            print_error(f"Report deletion failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report deletion error: {e}")
        return False
    
    # 5. 삭제 후 조회 (Report가 없어야 함)
    print_info("Step 5: Verifying report deletion...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report is None:
                print_success("Report deletion verified")
            else:
                print_error("Report still exists after deletion")
                return False
        elif response.status_code == 404:
            print_success("Report deletion verified (404)")
        else:
            print_error(f"Report retrieval after deletion failed: {response.status_code}")
            return False
    except Exception as e:
        print_error(f"Report verification error: {e}")
        return False
    
    return True

def scenario_2_global_report_crud():
    """시나리오 2: 전역 Report CRUD"""
    print_test("시나리오 2: 전역 Report CRUD")
    
    user = create_test_user()
    if not user:
        print_error("Failed to create test user")
        return False
    
    user_id = user.get("id")
    token = user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. 전역 Report 생성
    print_info("Step 1: Creating global report...")
    report_data = {
        "status": "unread",
        "description": "전역 리포트: 모든 프로젝트에서 볼 수 있습니다",
        "conclusion": "전역 결론",
        "bodypart": "head"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/report",
            json=report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report:
                print_success(f"Global report created: {report.get('id')}")
            else:
                print_error("Report not found in response")
                return False
        else:
            error_detail = response.text
            print_error(f"Global report creation failed: {response.status_code} - {error_detail}")
            # 404 에러인 경우 Series ID 확인
            if response.status_code == 404:
                print_info(f"Series ID used: {series_id}")
                # Series 존재 확인 시도
                try:
                    check_resp = requests.get(
                        f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
                        headers=get_headers(token)
                    )
                    print_info(f"Series check via note API: {check_resp.status_code}")
                except Exception as e:
                    print_info(f"Series check error: {e}")
            return False
    except Exception as e:
        print_error(f"Global report creation error: {e}")
        return False
    
    # 2. 전역 Report 조회
    print_info("Step 2: Retrieving global report...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == report_data["description"]:
                if report.get("project_id") is None:
                    print_success("Global report retrieved successfully (project_id is null)")
                else:
                    print_error("Global report should have null project_id")
                    return False
            else:
                print_error("Global report content mismatch")
                return False
        else:
            print_error(f"Global report retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global report retrieval error: {e}")
        return False
    
    # 3. 전역 Report 수정
    print_info("Step 3: Updating global report...")
    updated_report_data = {
        "status": "unapproval",
        "description": "업데이트된 전역 리포트",
        "conclusion": "업데이트된 전역 결론",
        "bodypart": "head"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/report",
            json=updated_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == updated_report_data["description"]:
                if report.get("status") == updated_report_data["status"]:
                    print_success("Global report updated successfully")
                else:
                    print_error(f"Status mismatch: expected {updated_report_data['status']}, got {report.get('status')}")
                    return False
            else:
                print_error("Global report update content mismatch")
                return False
        else:
            print_error(f"Global report update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global report update error: {e}")
        return False
    
    # 4. 전역 Report 삭제
    print_info("Step 4: Deleting global report...")
    try:
        response = requests.delete(
            f"{BASE_URL}/api/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Global report deleted successfully")
        else:
            print_error(f"Global report deletion failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global report deletion error: {e}")
        return False
    
    return True

def scenario_3_multiple_users_reports():
    """시나리오 3: 여러 사용자가 같은 Series에 Report 작성"""
    print_test("시나리오 3: 여러 사용자가 같은 Series에 Report 작성")
    
    user1 = create_test_user()
    user2 = create_test_user()
    
    if not user1 or not user2:
        print_error("Failed to create test users")
        return False
    
    user1_id = user1.get("id")
    user1_token = user1.get("token")
    user2_id = user2.get("id")
    user2_token = user2.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user1_id, project_id):
        print_error("Failed to add user1 to project")
        return False
    
    if not add_user_to_project(user2_id, project_id):
        print_error("Failed to add user2 to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. User1이 Report 작성
    print_info("Step 1: User1 creating report...")
    report1_data = {
        "status": "unread",
        "description": "User1의 리포트입니다",
        "conclusion": "User1의 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=report1_data,
            headers=get_headers(user1_token)
        )
        
        if response.status_code == 200:
            print_success("User1 report created")
        else:
            print_error(f"User1 report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"User1 report creation error: {e}")
        return False
    
    # 2. User2가 Report 작성
    print_info("Step 2: User2 creating report...")
    report2_data = {
        "status": "approval",
        "description": "User2의 리포트입니다",
        "conclusion": "User2의 결론",
        "bodypart": "abdomen"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=report2_data,
            headers=get_headers(user2_token)
        )
        
        if response.status_code == 200:
            print_success("User2 report created")
        else:
            print_error(f"User2 report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"User2 report creation error: {e}")
        return False
    
    # 3. 모든 Report 목록 조회
    print_info("Step 3: Retrieving all reports...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/reports",
            headers=get_headers(user1_token)
        )
        
        if response.status_code == 200:
            result = response.json()
            reports = result.get("reports", [])
            if len(reports) >= 2:
                print_success(f"Retrieved {len(reports)} reports")
                # 각 사용자의 Report 확인
                user1_report = next((r for r in reports if r.get("user", {}).get("id") == user1_id), None)
                user2_report = next((r for r in reports if r.get("user", {}).get("id") == user2_id), None)
                
                if user1_report and user2_report:
                    print_success("Both users' reports found in list")
                    # Status 확인
                    if user1_report.get("status") == "unread" and user2_report.get("status") == "approval":
                        print_success("Report statuses are correct")
                    else:
                        print_error("Report statuses mismatch")
                        return False
                else:
                    print_error("Some user reports missing from list")
                    return False
            else:
                print_error(f"Expected at least 2 reports, got {len(reports)}")
                return False
        else:
            print_error(f"Reports list retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Reports list retrieval error: {e}")
        return False
    
    return True

def scenario_4_project_and_global_reports_separation():
    """시나리오 4: 프로젝트별 Report와 전역 Report 분리"""
    print_test("시나리오 4: 프로젝트별 Report와 전역 Report 분리")
    
    user = create_test_user()
    if not user:
        print_error("Failed to create test user")
        return False
    
    user_id = user.get("id")
    token = user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id):
        print_error("Failed to add user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. 프로젝트별 Report 생성
    print_info("Step 1: Creating project report...")
    project_report_data = {
        "status": "unread",
        "description": "프로젝트별 리포트",
        "conclusion": "프로젝트별 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=project_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Project report created")
        else:
            print_error(f"Project report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Project report creation error: {e}")
        return False
    
    # 2. 전역 Report 생성
    print_info("Step 2: Creating global report...")
    global_report_data = {
        "status": "approval",
        "description": "전역 리포트",
        "conclusion": "전역 결론",
        "bodypart": "head"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/report",
            json=global_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Global report created")
        else:
            print_error(f"Global report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global report creation error: {e}")
        return False
    
    # 3. 프로젝트별 Report 조회 (프로젝트별만 조회되어야 함)
    print_info("Step 3: Retrieving project report...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == project_report_data["description"]:
                if report.get("project_id") == project_id:
                    print_success("Project report retrieved correctly")
                else:
                    print_error("Project report has wrong project_id")
                    return False
            else:
                print_error("Project report content mismatch")
                return False
        else:
            print_error(f"Project report retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Project report retrieval error: {e}")
        return False
    
    # 4. 전역 Report 조회 (전역만 조회되어야 함)
    print_info("Step 4: Retrieving global report...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/series/{series_id}/report",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("description") == global_report_data["description"]:
                if report.get("project_id") is None:
                    print_success("Global report retrieved correctly")
                else:
                    print_error("Global report should have null project_id")
                    return False
            else:
                print_error("Global report content mismatch")
                return False
        else:
            print_error(f"Global report retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global report retrieval error: {e}")
        return False
    
    return True

def scenario_5_permission_validation():
    """시나리오 5: 권한 검증 (프로젝트 멤버가 아닌 경우)"""
    print_test("시나리오 5: 권한 검증 (프로젝트 멤버가 아닌 경우)")
    
    member_user = create_test_user()
    non_member_user = create_test_user()
    
    if not member_user or not non_member_user:
        print_error("Failed to create test users")
        return False
    
    member_user_id = member_user.get("id")
    member_token = member_user.get("token")
    non_member_user_id = non_member_user.get("id")
    non_member_token = non_member_user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(member_user_id, project_id):
        print_error("Failed to add member user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. 멤버가 Report 생성 (성공해야 함)
    print_info("Step 1: Member creating report (should succeed)...")
    report_data = {
        "status": "unread",
        "description": "멤버의 리포트",
        "conclusion": "멤버의 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=report_data,
            headers=get_headers(member_token)
        )
        
        if response.status_code == 200:
            print_success("Member report creation succeeded (as expected)")
        else:
            print_error(f"Member report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Member report creation error: {e}")
        return False
    
    # 2. 비멤버가 Report 생성 시도 (실패해야 함)
    print_info("Step 2: Non-member creating report (should fail)...")
    non_member_report_data = {
        "status": "unread",
        "description": "비멤버의 리포트",
        "conclusion": "비멤버의 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=non_member_report_data,
            headers=get_headers(non_member_token)
        )
        
        if response.status_code in [401, 403]:
            print_success(f"Non-member report creation correctly rejected: {response.status_code}")
        else:
            print_error(f"Non-member report creation should fail but got: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Non-member report creation error: {e}")
        return False
    
    # 3. 비멤버가 전역 Report 생성 (성공해야 함 - 프로젝트 멤버십 불필요)
    print_info("Step 3: Non-member creating global report (should succeed)...")
    global_report_data = {
        "status": "unread",
        "description": "비멤버의 전역 리포트",
        "conclusion": "비멤버의 전역 결론",
        "bodypart": "head"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/report",
            json=global_report_data,
            headers=get_headers(non_member_token)
        )
        
        if response.status_code == 200:
            print_success("Non-member global report creation succeeded (as expected)")
        else:
            print_error(f"Non-member global report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Non-member global report creation error: {e}")
        return False
    
    return True

def scenario_6_report_status_changes():
    """시나리오 6: Report Status 변경 (unread, approval, unapproval)"""
    print_test("시나리오 6: Report Status 변경")
    
    user = create_test_user()
    if not user:
        print_error("Failed to create test user")
        return False
    
    user_id = user.get("id")
    token = user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id):
        print_error("Failed to add user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. unread 상태로 생성
    print_info("Step 1: Creating report with 'unread' status...")
    report_data = {
        "status": "unread",
        "description": "초기 리포트",
        "conclusion": "초기 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("status") == "unread":
                print_success("Report created with 'unread' status")
            else:
                print_error(f"Status mismatch: expected 'unread', got {report.get('status') if report else 'None'}")
                return False
        else:
            print_error(f"Report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report creation error: {e}")
        return False
    
    # 2. approval로 변경
    print_info("Step 2: Changing status to 'approval'...")
    updated_report_data = {
        "status": "approval",
        "description": "승인된 리포트",
        "conclusion": "승인된 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=updated_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("status") == "approval":
                print_success("Status changed to 'approval'")
            else:
                print_error(f"Status mismatch: expected 'approval', got {report.get('status') if report else 'None'}")
                return False
        else:
            print_error(f"Report update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report update error: {e}")
        return False
    
    # 3. unapproval로 변경
    print_info("Step 3: Changing status to 'unapproval'...")
    final_report_data = {
        "status": "unapproval",
        "description": "미승인된 리포트",
        "conclusion": "미승인된 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=final_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            if report and report.get("status") == "unapproval":
                print_success("Status changed to 'unapproval'")
            else:
                print_error(f"Status mismatch: expected 'unapproval', got {report.get('status') if report else 'None'}")
                return False
        else:
            print_error(f"Report update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report update error: {e}")
        return False
    
    return True

def scenario_7_report_upsert():
    """시나리오 7: Report 업데이트 (UPSERT 동작)"""
    print_test("시나리오 7: Report 업데이트 (UPSERT 동작)")
    
    user = create_test_user()
    if not user:
        print_error("Failed to create test user")
        return False
    
    user_id = user.get("id")
    token = user.get("token")
    
    project_id = create_test_project()
    if not project_id:
        print_error("Failed to create test project")
        return False
    
    if not add_user_to_project(user_id, project_id):
        print_error("Failed to add user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. 첫 번째 Report 생성
    print_info("Step 1: Creating initial report...")
    initial_report_data = {
        "status": "unread",
        "description": "초기 리포트",
        "conclusion": "초기 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=initial_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            report_id_1 = report.get("id") if report else None
            created_at_1 = report.get("created_at") if report else None
            print_success(f"Initial report created: {report_id_1}")
        else:
            print_error(f"Initial report creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Initial report creation error: {e}")
        return False
    
    # 2. 같은 Report 업데이트 (UPSERT)
    print_info("Step 2: Updating report (UPSERT)...")
    updated_report_data = {
        "status": "approval",
        "description": "업데이트된 리포트",
        "conclusion": "업데이트된 결론",
        "bodypart": "chest"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/report",
            json=updated_report_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            report = result.get("report")
            report_id_2 = report.get("id") if report else None
            updated_at = report.get("updated_at") if report else None
            
            # 같은 Report ID여야 함 (업데이트)
            if report_id_2 == report_id_1:
                print_success(f"Report updated (same ID: {report_id_2})")
                if updated_at != created_at_1:
                    print_success("updated_at timestamp changed")
                else:
                    print_error("updated_at should be different from created_at")
                    return False
            else:
                print_error(f"Report ID changed: {report_id_1} -> {report_id_2} (should be same)")
                return False
        else:
            print_error(f"Report update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Report update error: {e}")
        return False
    
    return True

def main():
    """메인 테스트 실행"""
    print("="*60)
    print("Series User Report API 시나리오 테스트")
    print("="*60)
    
    # 헬스 체크
    if not test_health():
        print("\n❌ Server is not available. Please start the server first.")
        sys.exit(1)
    
    # 시나리오 테스트 실행
    scenarios = [
        ("프로젝트 종속 Report CRUD", scenario_1_project_report_crud),
        ("전역 Report CRUD", scenario_2_global_report_crud),
        ("여러 사용자 Report 작성", scenario_3_multiple_users_reports),
        ("프로젝트/전역 Report 분리", scenario_4_project_and_global_reports_separation),
        ("권한 검증", scenario_5_permission_validation),
        ("Report Status 변경", scenario_6_report_status_changes),
        ("Report 업데이트 (UPSERT)", scenario_7_report_upsert),
    ]
    
    print("\n" + "="*60)
    print("시나리오 테스트 시작")
    print("="*60)
    
    for scenario_name, scenario_func in scenarios:
        try:
            scenario_func()
        except Exception as e:
            print_error(f"Scenario '{scenario_name}' failed with exception: {e}")
    
    # 결과 요약
    print("\n" + "="*60)
    print("테스트 결과 요약")
    print("="*60)
    print(f"총 테스트: {test_results['total']}")
    print(f"✅ 통과: {test_results['passed']}")
    print(f"❌ 실패: {test_results['failed']}")
    
    if test_results['failed'] == 0:
        print("\n🎉 모든 테스트가 통과했습니다!")
        sys.exit(0)
    else:
        print(f"\n⚠️  {test_results['failed']}개의 테스트가 실패했습니다.")
        sys.exit(1)

if __name__ == "__main__":
    main()

