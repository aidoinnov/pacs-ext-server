#!/usr/bin/env python3
"""
Series User Note API 시나리오 테스트 스크립트

이 스크립트는 Series User Note API의 다양한 시나리오를 테스트합니다:
1. 프로젝트 종속 Note 생성/조회/수정/삭제
2. 전역 Note 생성/조회/수정/삭제
3. 여러 사용자가 같은 Series에 Note 작성
4. 프로젝트별 Note와 전역 Note 분리
5. 권한 검증 (프로젝트 멤버가 아닌 경우)
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
        # 관리자 계정으로 승인 (테스트용)
        # 실제로는 관리자 토큰이 필요하지만, 테스트 환경에서는 허용될 수 있음
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
        # keycloak-token API 사용 (username/password 필요)
        # signup으로 생성된 사용자는 password를 알고 있음
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
    today = date.today()
    project_data = {
        "name": f"test_project_note_{int(time.time())}",
        "description": "Series Note API 테스트용 프로젝트",
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
    timestamp = int(time.time() * 1000)  # 밀리초 단위로 더 고유하게
    username = f"testuser_note_{timestamp}"
    email = f"test_note_{timestamp}@example.com"
    password = "TestPassword123!"  # signup API requires password
    keycloak_id = str(uuid.uuid4())  # UUID 사용
    
    user_data = {
        "username": username,
        "email": email,
        "password": password,
        "full_name": "테스트 사용자"
    }
    
    try:
        # 사용자 생성 (signup API 사용)
        response = requests.post(
            f"{BASE_URL}/api/auth/signup",
            json=user_data,
            headers=get_headers()
        )
        if response.status_code in [200, 201]:
            signup_result = response.json()
            user_id = signup_result.get("user_id") or signup_result.get("id")
            print_success(f"User created: {user_id} ({username})")
            
            # 사용자 승인 (테스트 환경)
            if approve_user(user_id):
                print_success(f"User {user_id} approved")
            
            # 로그인하여 토큰 얻기
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
        # 프로젝트에 사용자 추가 API 호출
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
        # Study 할당
        study_uid = f"1.2.840.113619.2.1.1.{int(time.time())}"
        study_data = {
            "study_uid": study_uid,
            "study_description": "Test Study for Note API",
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
        # 응답 구조 확인 필요 - study_id 또는 data.study.id 등
        study_id = study_result.get("study_id") or study_result.get("data", {}).get("study", {}).get("id")
        
        # Series 할당
        series_uid = f"1.2.840.113619.2.1.2.{int(time.time())}"
        series_data = {
            "study_uid": study_uid,
            "series_uid": series_uid,
            "series_description": "Test Series for Note API",
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
            # 응답 구조 확인 필요 - series_id 또는 data.series.id 등
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

def scenario_1_project_note_crud():
    """시나리오 1: 프로젝트 종속 Note CRUD"""
    print_test("시나리오 1: 프로젝트 종속 Note CRUD")
    
    # 테스트 데이터 준비
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
    
    # 1. Note 생성
    print_info("Step 1: Creating project note...")
    note_data = {
        "note": "이 시리즈는 프로젝트 A에서 분석 중입니다"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            print_success(f"Note created: {result.get('note', {}).get('id')}")
            note_id = result.get('note', {}).get('id')
        else:
            print_error(f"Note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Note creation error: {e}")
        return False
    
    # 2. Note 조회
    print_info("Step 2: Retrieving project note...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("note") and result["note"].get("note") == note_data["note"]:
                print_success("Note retrieved successfully")
            else:
                print_error("Note content mismatch")
                return False
        else:
            print_error(f"Note retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Note retrieval error: {e}")
        return False
    
    # 3. Note 수정
    print_info("Step 3: Updating project note...")
    updated_note_data = {
        "note": "업데이트된 메모: 분석 완료"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=updated_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("note", {}).get("note") == updated_note_data["note"]:
                print_success("Note updated successfully")
            else:
                print_error("Note update content mismatch")
                return False
        else:
            print_error(f"Note update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Note update error: {e}")
        return False
    
    # 4. Note 삭제
    print_info("Step 4: Deleting project note...")
    try:
        response = requests.delete(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Note deleted successfully")
        else:
            print_error(f"Note deletion failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Note deletion error: {e}")
        return False
    
    # 5. 삭제 후 조회 (Note가 없어야 함)
    print_info("Step 5: Verifying note deletion...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("note") is None:
                print_success("Note deletion verified")
            else:
                print_error("Note still exists after deletion")
                return False
        else:
            print_error(f"Note retrieval after deletion failed: {response.status_code}")
            return False
    except Exception as e:
        print_error(f"Note verification error: {e}")
        return False
    
    return True

def scenario_2_global_note_crud():
    """시나리오 2: 전역 Note CRUD"""
    print_test("시나리오 2: 전역 Note CRUD")
    
    # 테스트 데이터 준비
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
    
    # 1. 전역 Note 생성
    print_info("Step 1: Creating global note...")
    note_data = {
        "note": "전역 메모: 모든 프로젝트에서 볼 수 있습니다"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/note",
            json=note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            print_success(f"Global note created: {result.get('note', {}).get('id')}")
        else:
            print_error(f"Global note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note creation error: {e}")
        return False
    
    # 2. 전역 Note 조회
    print_info("Step 2: Retrieving global note...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("note") and result["note"].get("note") == note_data["note"]:
                if result["note"].get("project_id") is None:
                    print_success("Global note retrieved successfully (project_id is null)")
                else:
                    print_error("Global note should have null project_id")
                    return False
            else:
                print_error("Global note content mismatch")
                return False
        else:
            print_error(f"Global note retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note retrieval error: {e}")
        return False
    
    # 3. 전역 Note 수정
    print_info("Step 3: Updating global note...")
    updated_note_data = {
        "note": "업데이트된 전역 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/note",
            json=updated_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("note", {}).get("note") == updated_note_data["note"]:
                print_success("Global note updated successfully")
            else:
                print_error("Global note update content mismatch")
                return False
        else:
            print_error(f"Global note update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note update error: {e}")
        return False
    
    # 4. 전역 Note 삭제
    print_info("Step 4: Deleting global note...")
    try:
        response = requests.delete(
            f"{BASE_URL}/api/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Global note deleted successfully")
        else:
            print_error(f"Global note deletion failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note deletion error: {e}")
        return False
    
    return True

def scenario_3_multiple_users_notes():
    """시나리오 3: 여러 사용자가 같은 Series에 Note 작성"""
    print_test("시나리오 3: 여러 사용자가 같은 Series에 Note 작성")
    
    # 테스트 데이터 준비
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
    
    # 1. User1이 Note 작성
    print_info("Step 1: User1 creating note...")
    note1_data = {
        "note": "User1의 메모입니다"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=note1_data,
            headers=get_headers(user1_token)
        )
        
        if response.status_code == 200:
            print_success("User1 note created")
        else:
            print_error(f"User1 note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"User1 note creation error: {e}")
        return False
    
    # 2. User2가 Note 작성
    print_info("Step 2: User2 creating note...")
    note2_data = {
        "note": "User2의 메모입니다"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=note2_data,
            headers=get_headers(user2_token)
        )
        
        if response.status_code == 200:
            print_success("User2 note created")
        else:
            print_error(f"User2 note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"User2 note creation error: {e}")
        return False
    
    # 3. 모든 Note 목록 조회
    print_info("Step 3: Retrieving all notes...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/notes",
            headers=get_headers(user1_token)
        )
        
        if response.status_code == 200:
            result = response.json()
            notes = result.get("notes", [])
            if len(notes) >= 2:
                print_success(f"Retrieved {len(notes)} notes")
                # 각 사용자의 Note 확인
                user1_note = next((n for n in notes if n.get("user", {}).get("id") == user1_id), None)
                user2_note = next((n for n in notes if n.get("user", {}).get("id") == user2_id), None)
                
                if user1_note and user2_note:
                    print_success("Both users' notes found in list")
                else:
                    print_error("Some user notes missing from list")
                    return False
            else:
                print_error(f"Expected at least 2 notes, got {len(notes)}")
                return False
        else:
            print_error(f"Notes list retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Notes list retrieval error: {e}")
        return False
    
    return True

def scenario_4_project_and_global_notes_separation():
    """시나리오 4: 프로젝트별 Note와 전역 Note 분리"""
    print_test("시나리오 4: 프로젝트별 Note와 전역 Note 분리")
    
    # 테스트 데이터 준비
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
    
    # 1. 프로젝트별 Note 생성
    print_info("Step 1: Creating project note...")
    project_note_data = {
        "note": "프로젝트별 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=project_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Project note created")
        else:
            print_error(f"Project note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Project note creation error: {e}")
        return False
    
    # 2. 전역 Note 생성
    print_info("Step 2: Creating global note...")
    global_note_data = {
        "note": "전역 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/note",
            json=global_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            print_success("Global note created")
        else:
            print_error(f"Global note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note creation error: {e}")
        return False
    
    # 3. 프로젝트별 Note 조회 (프로젝트별만 조회되어야 함)
    print_info("Step 3: Retrieving project note...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            note = result.get("note")
            if note and note.get("note") == project_note_data["note"]:
                if note.get("project_id") == project_id:
                    print_success("Project note retrieved correctly")
                else:
                    print_error("Project note has wrong project_id")
                    return False
            else:
                print_error("Project note content mismatch")
                return False
        else:
            print_error(f"Project note retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Project note retrieval error: {e}")
        return False
    
    # 4. 전역 Note 조회 (전역만 조회되어야 함)
    print_info("Step 4: Retrieving global note...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/series/{series_id}/note",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            note = result.get("note")
            if note and note.get("note") == global_note_data["note"]:
                if note.get("project_id") is None:
                    print_success("Global note retrieved correctly")
                else:
                    print_error("Global note should have null project_id")
                    return False
            else:
                print_error("Global note content mismatch")
                return False
        else:
            print_error(f"Global note retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Global note retrieval error: {e}")
        return False
    
    return True

def scenario_5_permission_validation():
    """시나리오 5: 권한 검증 (프로젝트 멤버가 아닌 경우)"""
    print_test("시나리오 5: 권한 검증 (프로젝트 멤버가 아닌 경우)")
    
    # 테스트 데이터 준비
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
    
    # 멤버만 프로젝트에 추가
    if not add_user_to_project(member_user_id, project_id):
        print_error("Failed to add member user to project")
        return False
    
    series_id = create_test_series(project_id)
    if not series_id:
        print_error("Failed to create test series")
        return False
    
    # 1. 멤버가 Note 생성 (성공해야 함)
    print_info("Step 1: Member creating note (should succeed)...")
    note_data = {
        "note": "멤버의 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=note_data,
            headers=get_headers(member_token)
        )
        
        if response.status_code == 200:
            print_success("Member note creation succeeded (as expected)")
        else:
            print_error(f"Member note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Member note creation error: {e}")
        return False
    
    # 2. 비멤버가 Note 생성 시도 (실패해야 함)
    print_info("Step 2: Non-member creating note (should fail)...")
    non_member_note_data = {
        "note": "비멤버의 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=non_member_note_data,
            headers=get_headers(non_member_token)
        )
        
        if response.status_code in [401, 403]:
            print_success(f"Non-member note creation correctly rejected: {response.status_code}")
        else:
            print_error(f"Non-member note creation should fail but got: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Non-member note creation error: {e}")
        return False
    
    # 3. 비멤버가 전역 Note 생성 (성공해야 함 - 프로젝트 멤버십 불필요)
    print_info("Step 3: Non-member creating global note (should succeed)...")
    global_note_data = {
        "note": "비멤버의 전역 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/series/{series_id}/note",
            json=global_note_data,
            headers=get_headers(non_member_token)
        )
        
        if response.status_code == 200:
            print_success("Non-member global note creation succeeded (as expected)")
        else:
            print_error(f"Non-member global note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Non-member global note creation error: {e}")
        return False
    
    return True

def scenario_6_note_update_upsert():
    """시나리오 6: Note 업데이트 (UPSERT 동작)"""
    print_test("시나리오 6: Note 업데이트 (UPSERT 동작)")
    
    # 테스트 데이터 준비
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
    
    # 1. 첫 번째 Note 생성
    print_info("Step 1: Creating initial note...")
    initial_note_data = {
        "note": "초기 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=initial_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            note_id_1 = result.get("note", {}).get("id")
            created_at_1 = result.get("note", {}).get("created_at")
            print_success(f"Initial note created: {note_id_1}")
        else:
            print_error(f"Initial note creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Initial note creation error: {e}")
        return False
    
    # 2. 같은 Note 업데이트 (UPSERT)
    print_info("Step 2: Updating note (UPSERT)...")
    updated_note_data = {
        "note": "업데이트된 메모"
    }
    
    try:
        response = requests.put(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/note",
            json=updated_note_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            note_id_2 = result.get("note", {}).get("id")
            updated_at = result.get("note", {}).get("updated_at")
            
            # 같은 Note ID여야 함 (업데이트)
            if note_id_2 == note_id_1:
                print_success(f"Note updated (same ID: {note_id_2})")
                if updated_at != created_at_1:
                    print_success("updated_at timestamp changed")
                else:
                    print_error("updated_at should be different from created_at")
                    return False
            else:
                print_error(f"Note ID changed: {note_id_1} -> {note_id_2} (should be same)")
                return False
        else:
            print_error(f"Note update failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Note update error: {e}")
        return False
    
    return True

def scenario_7_empty_notes_list():
    """시나리오 7: Note가 없는 경우 목록 조회"""
    print_test("시나리오 7: Note가 없는 경우 목록 조회")
    
    # 테스트 데이터 준비
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
    
    # Note가 없는 상태에서 목록 조회
    print_info("Retrieving notes list (should be empty)...")
    try:
        response = requests.get(
            f"{BASE_URL}/api/project-data/{project_id}/series/{series_id}/notes",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            notes = result.get("notes", [])
            if len(notes) == 0:
                print_success("Empty notes list returned correctly")
            else:
                print_error(f"Expected empty list, got {len(notes)} notes")
                return False
        else:
            print_error(f"Notes list retrieval failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Notes list retrieval error: {e}")
        return False
    
    return True

def main():
    """메인 테스트 실행"""
    print("="*60)
    print("Series User Note API 시나리오 테스트")
    print("="*60)
    
    # 헬스 체크
    if not test_health():
        print("\n❌ Server is not available. Please start the server first.")
        sys.exit(1)
    
    # 시나리오 테스트 실행
    scenarios = [
        ("프로젝트 종속 Note CRUD", scenario_1_project_note_crud),
        ("전역 Note CRUD", scenario_2_global_note_crud),
        ("여러 사용자 Note 작성", scenario_3_multiple_users_notes),
        ("프로젝트/전역 Note 분리", scenario_4_project_and_global_notes_separation),
        ("권한 검증", scenario_5_permission_validation),
        ("Note 업데이트 (UPSERT)", scenario_6_note_update_upsert),
        ("빈 Note 목록 조회", scenario_7_empty_notes_list),
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

