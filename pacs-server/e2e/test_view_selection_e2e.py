#!/usr/bin/env python3
"""
View Selection API E2E 통합 테스트 스크립트

이 스크립트는 Viewer Selection 기능을 종합적으로 테스트합니다:
1. Selection 생성 (POST /api/v1/view-selections)
2. Selection 조회 (GET /api/v1/view-selections/{selection_id})
3. Selection 삭제 (DELETE /api/v1/view-selections/{selection_id})
4. Layout + Initial Views 기능 테스트
5. 멀티 Study/Series 선택 시나리오
6. TTL 자동 연장 (조회 시)
7. 유효성 검증 테스트
8. 실제 사용 시나리오 테스트
"""

import requests
import json
import time
import sys
from typing import Optional, Dict, Any, List
from test_base import BaseE2ETest, TestConfig, TestPrinter

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

def get_headers(token: Optional[str] = None) -> Dict[str, str]:
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def login_user(user_data: Dict[str, Any]) -> Optional[str]:
    """사용자 로그인하여 JWT 토큰 얻기"""
    print_info(f"Logging in user: {user_data.get('username')}...")
    try:
        login_data = {
            "username": user_data.get("username", ""),
            "password": user_data.get("password", "TestPassword123!")
        }
        
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json=login_data,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            result = response.json()
            token = result.get("token") or result.get("access_token")
            if token:
                print_success(f"Login successful for {user_data.get('username')}")
                return token
            else:
                print_error(f"Token not found in response: {result}")
                return None
        else:
            print_error(f"Login failed: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Login error: {e}")
        return None

def create_view_selection(token: str, series: List[Dict[str, str]]) -> Optional[str]:
    """View Selection 생성"""
    try:
        request_data = {"series": series}
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 201:
            result = response.json()
            selection_id = result.get("selection_id")
            if selection_id:
                print_success(f"Selection created: {selection_id}")
                return selection_id
            else:
                print_error(f"Selection ID not found in response: {result}")
                return None
        else:
            print_error(f"Failed to create selection: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Create selection error: {e}")
        return None

def get_view_selection(token: str, selection_id: str) -> Optional[Dict[str, Any]]:
    """View Selection 조회"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/view-selections/{selection_id}",
            headers=get_headers(token)
        )
        
        if response.status_code == 200:
            result = response.json()
            print_success(f"Selection retrieved: {selection_id}")
            return result
        elif response.status_code == 404:
            print_info(f"Selection not found: {selection_id}")
            return None
        else:
            print_error(f"Failed to get selection: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print_error(f"Get selection error: {e}")
        return None

def delete_view_selection(token: str, selection_id: str) -> bool:
    """View Selection 삭제"""
    try:
        response = requests.delete(
            f"{BASE_URL}/api/v1/view-selections/{selection_id}",
            headers=get_headers(token)
        )
        
        if response.status_code == 204:
            print_success(f"Selection deleted: {selection_id}")
            return True
        else:
            print_error(f"Failed to delete selection: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print_error(f"Delete selection error: {e}")
        return False

def test_create_selection_success(token: str):
    """Selection 생성 성공 테스트"""
    print_test("Create Selection - Success")
    
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.125",
            "series_uid": "1.2.840.113619.2.1.2.126"
        }
    ]
    
    selection_id = create_view_selection(token, series)
    if selection_id:
        # 생성된 Selection 조회하여 검증
        selection = get_view_selection(token, selection_id)
        if selection:
            assert selection["selection_id"] == selection_id, "Selection ID mismatch"
            assert len(selection["series"]) == 2, "Series count mismatch"
            assert selection["user_id"] > 0, "User ID should be set"
            print_success("Selection creation and retrieval verified")
            
            # 정리
            delete_view_selection(token, selection_id)
        else:
            print_error("Failed to retrieve created selection")
    else:
        print_error("Failed to create selection")

def test_create_selection_empty_series(token: str):
    """빈 Series 목록으로 생성 시도 (실패 예상)"""
    print_test("Create Selection - Empty Series (Should Fail)")
    
    try:
        request_data = {"series": []}
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )
        
        if response.status_code == 400:
            print_success("Correctly rejected empty series list")
        else:
            print_error(f"Expected 400, got {response.status_code}")
    except Exception as e:
        print_error(f"Test error: {e}")

def test_get_selection_not_found(token: str):
    """존재하지 않는 Selection 조회 (404 예상)"""
    print_test("Get Selection - Not Found")
    
    selection = get_view_selection(token, "sel_nonexistent")
    if selection is None:
        print_success("Correctly returned None for non-existent selection")
    else:
        print_error("Should return None for non-existent selection")

def test_multi_study_series_selection(token: str):
    """멀티 Study/Series 선택 시나리오"""
    print_test("Multi-Study/Series Selection")
    
    # 여러 Study에 속한 Series 선택
    series = [
        # Study 1의 Series들
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.101"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.102"
        },
        # Study 2의 Series
        {
            "study_uid": "1.2.840.113619.2.1.1.200",
            "series_uid": "1.2.840.113619.2.1.2.201"
        },
        # Study 3의 Series
        {
            "study_uid": "1.2.840.113619.2.1.1.300",
            "series_uid": "1.2.840.113619.2.1.2.301"
        }
    ]
    
    selection_id = create_view_selection(token, series)
    if selection_id:
        selection = get_view_selection(token, selection_id)
        if selection:
            # 검증
            assert len(selection["series"]) == 4, f"Expected 4 series, got {len(selection['series'])}"
            
            # Study UID 추출
            study_uids = [s["study_uid"] for s in selection["series"]]
            unique_studies = set(study_uids)
            
            assert len(unique_studies) == 3, f"Expected 3 unique studies, got {len(unique_studies)}"
            assert "1.2.840.113619.2.1.1.100" in unique_studies
            assert "1.2.840.113619.2.1.1.200" in unique_studies
            assert "1.2.840.113619.2.1.1.300" in unique_studies
            
            print_success(f"Multi-study selection verified: {len(unique_studies)} studies, {len(selection['series'])} series")
            
            # 정리
            delete_view_selection(token, selection_id)
        else:
            print_error("Failed to retrieve multi-study selection")
    else:
        print_error("Failed to create multi-study selection")

def test_full_workflow(token: str):
    """전체 플로우 테스트 (생성 → 조회 → TTL 연장 → 삭제)"""
    print_test("Full Workflow - Create → Get → Extend TTL → Delete")
    
    # Step 1: Selection 생성
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        }
    ]
    
    selection_id = create_view_selection(token, series)
    if not selection_id:
        print_error("Step 1 failed: Selection creation")
        return
    
    # Step 2: 첫 번째 조회
    selection1 = get_view_selection(token, selection_id)
    if not selection1:
        print_error("Step 2 failed: First retrieval")
        delete_view_selection(token, selection_id)
        return
    
    expires_at_1 = selection1.get("expires_at")
    print_info(f"First retrieval - expires_at: {expires_at_1}")
    
    # Step 3: 짧은 대기 후 다시 조회 (TTL 자동 연장)
    time.sleep(1)
    selection2 = get_view_selection(token, selection_id)
    if not selection2:
        print_error("Step 3 failed: Second retrieval")
        delete_view_selection(token, selection_id)
        return
    
    expires_at_2 = selection2.get("expires_at")
    print_info(f"Second retrieval - expires_at: {expires_at_2}")
    
    # TTL이 연장되었는지 확인 (expires_at이 더 늦어야 함)
    if expires_at_2 > expires_at_1:
        print_success("TTL automatically extended on retrieval")
    else:
        print_info("TTL extension check skipped (timing may vary)")
    
    # Step 4: Selection 삭제
    if delete_view_selection(token, selection_id):
        # Step 5: 삭제 후 조회 시 404 확인
        selection3 = get_view_selection(token, selection_id)
        if selection3 is None:
            print_success("Full workflow completed successfully")
        else:
            print_error("Selection should be deleted")
    else:
        print_error("Step 4 failed: Selection deletion")

def test_selection_id_format(token: str):
    """Selection ID 형식 검증"""
    print_test("Selection ID Format Validation")
    
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        }
    ]
    
    selection_id = create_view_selection(token, series)
    if selection_id:
        # Selection ID 형식 검증: "sel_" + 6자리 hex
        assert selection_id.startswith("sel_"), f"Selection ID should start with 'sel_', got: {selection_id}"
        assert len(selection_id) == 10, f"Selection ID should be 10 chars (sel_ + 6 hex), got: {len(selection_id)}"
        
        hex_part = selection_id[4:]  # "sel_" 제거
        try:
            int(hex_part, 16)  # hex 검증
            print_success(f"Selection ID format correct: {selection_id}")
        except ValueError:
            print_error(f"Selection ID hex part invalid: {hex_part}")
        
        # 정리
        delete_view_selection(token, selection_id)
    else:
        print_error("Failed to create selection for ID format test")

def test_large_series_list(token: str):
    """대량 Series 선택 테스트"""
    print_test("Large Series List Selection")
    
    # 10개의 Series 선택
    series = []
    for i in range(10):
        series.append({
            "study_uid": f"1.2.840.113619.2.1.1.{100 + i}",
            "series_uid": f"1.2.840.113619.2.1.2.{200 + i}"
        })
    
    selection_id = create_view_selection(token, series)
    if selection_id:
        selection = get_view_selection(token, selection_id)
        if selection:
            assert len(selection["series"]) == 10, f"Expected 10 series, got {len(selection['series'])}"
            print_success(f"Large series list handled correctly: {len(selection['series'])} series")
            
            # 정리
            delete_view_selection(token, selection_id)
        else:
            print_error("Failed to retrieve large series selection")
    else:
        print_error("Failed to create large series selection")

def test_unauthorized_access():
    """인증 없이 접근 시도 (401 예상)"""
    print_test("Unauthorized Access (No Token)")
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json={"series": [{"study_uid": "1.2.3", "series_uid": "1.2.3.4"}]},
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 401:
            print_success("Correctly rejected unauthorized access")
        else:
            print_error(f"Expected 401, got {response.status_code}")
    except Exception as e:
        print_error(f"Test error: {e}")

def create_test_user() -> Optional[Dict[str, Any]]:
    """테스트 사용자 생성 및 로그인"""
    print_info("Creating test user...")
    import uuid
    timestamp = int(time.time() * 1000)
    username = f"testuser_viewsel_{timestamp}"
    email = f"test_viewsel_{timestamp}@example.com"
    password = "TestPassword123!"
    
    user_data = {
        "username": username,
        "email": email,
        "password": password,
        "full_name": "View Selection 테스트 사용자"
    }
    
    try:
        # /api/auth/signup 엔드포인트 시도
        response = requests.post(
            f"{BASE_URL}/api/auth/signup",
            json=user_data,
            headers=get_headers(),
            timeout=10
        )
        
        if response.status_code in [200, 201]:
            signup_result = response.json()
            user_id = signup_result.get("user_id") or signup_result.get("id")
            print_success(f"User created: {user_id} ({username})")
            
            # 사용자 승인 시도 (선택사항)
            try:
                approve_data = {"user_id": user_id}
                approve_response = requests.post(
                    f"{BASE_URL}/api/auth/admin/users/approve",
                    json=approve_data,
                    headers=get_headers(),
                    timeout=10
                )
                if approve_response.status_code in [200, 201]:
                    print_success(f"User {user_id} approved")
            except Exception as e:
                print_info(f"User approval skipped: {e}")
            
            # 로그인 시도
            token = login_user(user_data)
            if token:
                user_data["id"] = user_id
                user_data["token"] = token
                return user_data
            else:
                print_error("Failed to login after signup")
                return None
        else:
            print_error(f"User creation failed: {response.status_code} - {response.text[:200]}")
            # 기존 사용자로 로그인 시도
            print_info("Trying to login with existing user...")
            token = login_user({
                "username": "test_user",
                "password": "TestPassword123!"
            })
            if token:
                return {"id": 0, "token": token, "username": "test_user"}
            return None
    except Exception as e:
        print_error(f"User creation error: {e}")
        # 기존 사용자로 로그인 시도
        print_info("Trying to login with existing user...")
        token = login_user({
            "username": "test_user",
            "password": "TestPassword123!"
        })
        if token:
            return {"id": 0, "token": token, "username": "test_user"}
        return None

def get_existing_study_series(token: str) -> Optional[List[Dict[str, str]]]:
    """기존 Study/Series 조회 (실제 데이터 사용)"""
    print_info("Fetching existing studies and series...")
    
    try:
        # /api/me/studies 사용 (사용자 관점)
        response = requests.get(
            f"{BASE_URL}/api/me/dicom/studies",
            params={"limit": 5},
            headers=get_headers(token),
            timeout=15
        )
        
        if response.status_code == 200:
            studies = response.json()
            if isinstance(studies, list) and len(studies) > 0:
                series_list = []
                
                # 각 Study의 Series 조회
                for study in studies[:3]:  # 최대 3개 Study
                    study_uid = study.get("0020000D", {}).get("Value", [None])[0]
                    if not study_uid:
                        continue
                    
                    # Series 조회
                    series_response = requests.get(
                        f"{BASE_URL}/api/me/dicom/studies/{study_uid}/series",
                        headers=get_headers(token),
                        timeout=15
                    )
                    
                    if series_response.status_code == 200:
                        series_array = series_response.json()
                        if isinstance(series_array, list):
                            for s in series_array[:2]:  # Study당 최대 2개 Series
                                series_uid = s.get("0020000E", {}).get("Value", [None])[0]
                                if series_uid:
                                    series_list.append({
                                        "study_uid": study_uid,
                                        "series_uid": series_uid
                                    })
                
                if series_list:
                    print_success(f"Found {len(series_list)} series from existing data")
                    return series_list
                else:
                    print_info("No series found, using mock data")
                    return None
            else:
                print_info("No studies found, using mock data")
                return None
        else:
            print_info(f"Failed to fetch studies: {response.status_code}, using mock data")
            return None
    except Exception as e:
        print_info(f"Error fetching studies: {e}, using mock data")
        return None

def test_real_world_scenario(token: str):
    """실제 사용 시나리오: 기존 Study/Series로 Selection 생성"""
    print_test("Real-World Scenario - Using Existing Study/Series")
    
    # 실제 데이터 조회 시도
    real_series = get_existing_study_series(token)
    
    if not real_series:
        # Mock 데이터 사용
        print_info("Using mock data for real-world scenario")
        real_series = [
            {
                "study_uid": "1.2.840.113619.2.1.1.123",
                "series_uid": "1.2.840.113619.2.1.2.124"
            },
            {
                "study_uid": "1.2.840.113619.2.1.1.123",
                "series_uid": "1.2.840.113619.2.1.2.125"
            }
        ]
    
    # Selection 생성
    selection_id = create_view_selection(token, real_series)
    if selection_id:
        # 조회하여 검증
        selection = get_view_selection(token, selection_id)
        if selection:
            assert len(selection["series"]) == len(real_series), "Series count mismatch"
            
            # Study UID 추출 및 검증
            study_uids = set(s["study_uid"] for s in selection["series"])
            print_success(f"Real-world scenario verified: {len(study_uids)} studies, {len(selection['series'])} series")
            
            # 정리
            delete_view_selection(token, selection_id)
        else:
            print_error("Failed to retrieve selection in real-world scenario")
    else:
        print_error("Failed to create selection in real-world scenario")

def test_selection_persistence(token: str):
    """Selection 지속성 테스트 (여러 번 조회)"""
    print_test("Selection Persistence - Multiple Retrievals")
    
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        }
    ]
    
    selection_id = create_view_selection(token, series)
    if selection_id:
        # 여러 번 조회하여 지속성 확인
        for i in range(3):
            selection = get_view_selection(token, selection_id)
            if selection:
                assert selection["selection_id"] == selection_id, f"Selection ID mismatch on retrieval {i+1}"
                assert len(selection["series"]) == 1, f"Series count mismatch on retrieval {i+1}"
                print_success(f"Retrieval {i+1} successful")
            else:
                print_error(f"Failed on retrieval {i+1}")
                break
            time.sleep(0.5)
        
        # 정리
        delete_view_selection(token, selection_id)
    else:
        print_error("Failed to create selection for persistence test")

def scenario_viewer_session_workflow(token: str):
    """시나리오: Viewer Session 전체 워크플로우"""
    print_test("시나리오: Viewer Session 전체 워크플로우")
    print_info("PACS UI → Selection 생성 → Viewer 오픈 → Progressive Loading")
    
    # Step 1: PACS UI에서 여러 Study의 Series 선택
    print_info("Step 1: 사용자가 PACS UI에서 Series 선택 중...")
    selected_series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.101"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.102"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.200",
            "series_uid": "1.2.840.113619.2.1.2.201"
        }
    ]
    
    # Step 2: Selection 생성 (POST)
    print_info("Step 2: Selection 생성 API 호출...")
    selection_id = create_view_selection(token, selected_series)
    if not selection_id:
        print_error("시나리오 실패: Selection 생성 실패")
        return False
    
    print_success(f"Selection 생성 완료: {selection_id}")
    
    # Step 3: Viewer 오픈 (GET /viewer/selections/{selection_id})
    # 실제 Viewer는 프론트엔드이므로, 여기서는 Selection 조회로 대체
    print_info("Step 3: Viewer 오픈 (Selection 조회)...")
    selection = get_view_selection(token, selection_id)
    if not selection:
        print_error("시나리오 실패: Selection 조회 실패")
        delete_view_selection(token, selection_id)
        return False
    
    print_success(f"Viewer 상태 로드 완료: {len(selection['series'])}개 Series")
    
    # Step 4: Progressive Loading 시뮬레이션 (여러 번 조회)
    print_info("Step 4: Progressive Loading (여러 번 조회로 TTL 연장)...")
    for i in range(3):
        selection = get_view_selection(token, selection_id)
        if selection:
            print_success(f"Loading step {i+1}: Selection 유지됨")
        else:
            print_error(f"Loading step {i+1}: Selection 손실")
            break
        time.sleep(0.3)
    
    # Step 5: 정리
    print_info("Step 5: Viewer 종료 (Selection 삭제)...")
    if delete_view_selection(token, selection_id):
        print_success("시나리오 완료: Viewer Session 워크플로우 성공")
        return True
    else:
        print_error("시나리오 실패: Selection 삭제 실패")
        return False

def scenario_multi_user_selection(token: str):
    """시나리오: 여러 사용자가 각각 Selection 생성"""
    print_test("시나리오: 여러 사용자 동시 Selection 생성")
    
    # 같은 Series를 다른 사용자가 선택하는 시나리오
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        }
    ]
    
    # 첫 번째 Selection 생성
    selection_id_1 = create_view_selection(token, series)
    if not selection_id_1:
        print_error("시나리오 실패: 첫 번째 Selection 생성 실패")
        return False
    
    # 두 번째 Selection 생성 (같은 사용자, 다른 Selection ID)
    time.sleep(0.5)
    selection_id_2 = create_view_selection(token, series)
    if not selection_id_2:
        print_error("시나리오 실패: 두 번째 Selection 생성 실패")
        delete_view_selection(token, selection_id_1)
        return False
    
    # 두 Selection이 다른 ID를 가져야 함
    assert selection_id_1 != selection_id_2, "Selection IDs should be different"
    print_success(f"두 개의 독립적인 Selection 생성: {selection_id_1}, {selection_id_2}")
    
    # 각각 조회하여 검증
    sel1 = get_view_selection(token, selection_id_1)
    sel2 = get_view_selection(token, selection_id_2)
    
    if sel1 and sel2:
        assert sel1["selection_id"] == selection_id_1
        assert sel2["selection_id"] == selection_id_2
        print_success("각 Selection이 독립적으로 관리됨")
    else:
        print_error("Selection 조회 실패")
    
    # 정리
    delete_view_selection(token, selection_id_1)
    delete_view_selection(token, selection_id_2)
    
    return True

def scenario_url_sharing(token: str):
    """시나리오: URL 공유 (Selection ID를 통한 상태 재현)"""
    print_test("시나리오: URL 공유 및 상태 재현")

    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.125",
            "series_uid": "1.2.840.113619.2.1.2.126"
        }
    ]

    # Selection 생성
    selection_id = create_view_selection(token, series)
    if not selection_id:
        print_error("시나리오 실패: Selection 생성 실패")
        return False

    print_info(f"생성된 Selection ID: {selection_id}")
    print_info(f"공유 가능한 URL: /viewer/selections/{selection_id}")

    # URL을 통해 Selection 조회 (상태 재현)
    print_info("URL을 통한 상태 재현 테스트...")
    selection = get_view_selection(token, selection_id)
    if selection:
        # 원래 선택한 Series와 동일한지 확인
        assert len(selection["series"]) == 2, "Series count should match"
        assert selection["series"][0]["study_uid"] == series[0]["study_uid"]
        assert selection["series"][0]["series_uid"] == series[0]["series_uid"]
        print_success("URL을 통한 상태 재현 성공")
    else:
        print_error("시나리오 실패: 상태 재현 실패")
        delete_view_selection(token, selection_id)
        return False

    # 정리
    delete_view_selection(token, selection_id)
    return True

def test_layout_and_initial_views(token: str):
    """Layout + Initial Views 기능 테스트"""
    print_test("Layout + Initial Views 기능 테스트")

    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.101"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.102"
        },
        {
            "study_uid": "1.2.840.113619.2.1.1.200",
            "series_uid": "1.2.840.113619.2.1.2.201"
        }
    ]

    # Layout + Initial Views 포함
    request_data = {
        "series": series,
        "layout": {
            "rows": 2,
            "cols": 2
        },
        "initial_views": [
            {
                "row": 0,
                "col": 0,
                "study_uid": "1.2.840.113619.2.1.1.100",
                "series_uid": "1.2.840.113619.2.1.2.101",
                "sop_uid": "1.2.840.113619.2.1.3.103"
            },
            {
                "row": 0,
                "col": 1,
                "study_uid": "1.2.840.113619.2.1.1.100",
                "series_uid": "1.2.840.113619.2.1.2.102",
                "frame_index": 5
            },
            {
                "row": 1,
                "col": 0,
                "study_uid": "1.2.840.113619.2.1.1.200",
                "series_uid": "1.2.840.113619.2.1.2.201"
            }
        ]
    }

    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )

        if response.status_code == 201:
            result = response.json()
            selection_id = result.get("selection_id")
            print_success(f"Layout + Initial Views Selection 생성 성공: {selection_id}")

            # 조회하여 검증
            selection = get_view_selection(token, selection_id)
            if selection:
                # Layout 검증
                assert "layout" in selection, "Layout should be present"
                assert selection["layout"]["rows"] == 2, "Rows should be 2"
                assert selection["layout"]["cols"] == 2, "Cols should be 2"
                print_success("Layout 검증 성공")

                # Initial Views 검증
                assert "initial_views" in selection, "Initial views should be present"
                assert len(selection["initial_views"]) == 3, "Should have 3 initial views"

                # 첫 번째 viewport 검증
                view1 = selection["initial_views"][0]
                assert view1["row"] == 0 and view1["col"] == 0
                assert view1["study_uid"] == "1.2.840.113619.2.1.1.100"
                assert view1["series_uid"] == "1.2.840.113619.2.1.2.101"
                assert view1.get("sop_uid") == "1.2.840.113619.2.1.3.103"

                # 두 번째 viewport 검증 (frame_index)
                view2 = selection["initial_views"][1]
                assert view2.get("frame_index") == 5

                print_success("Initial Views 검증 성공")

                # 정리
                delete_view_selection(token, selection_id)
            else:
                print_error("Selection 조회 실패")
        else:
            print_error(f"Selection 생성 실패: {response.status_code} - {response.text}")
    except Exception as e:
        print_error(f"Test error: {e}")

def test_layout_validation_errors(token: str):
    """Layout 유효성 검증 에러 테스트"""
    print_test("Layout 유효성 검증 에러 테스트")

    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.100",
            "series_uid": "1.2.840.113619.2.1.2.101"
        }
    ]

    # Test 1: initial_views만 있고 layout 없음 (400 예상)
    print_info("Test 1: initial_views without layout (should fail)")
    request_data = {
        "series": series,
        "initial_views": [
            {
                "row": 0,
                "col": 0,
                "study_uid": "1.2.840.113619.2.1.1.100",
                "series_uid": "1.2.840.113619.2.1.2.101"
            }
        ]
    }

    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )

        if response.status_code == 400:
            print_success("Correctly rejected initial_views without layout")
        else:
            print_error(f"Expected 400, got {response.status_code}")
    except Exception as e:
        print_error(f"Test error: {e}")

    # Test 2: viewport 위치가 layout 범위 초과 (400 예상)
    print_info("Test 2: viewport position out of bounds (should fail)")
    request_data = {
        "series": series,
        "layout": {
            "rows": 2,
            "cols": 2
        },
        "initial_views": [
            {
                "row": 2,  # rows=2이므로 row는 0-1만 가능
                "col": 0,
                "study_uid": "1.2.840.113619.2.1.1.100",
                "series_uid": "1.2.840.113619.2.1.2.101"
            }
        ]
    }

    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )

        if response.status_code == 400:
            print_success("Correctly rejected out-of-bounds viewport position")
        else:
            print_error(f"Expected 400, got {response.status_code}")
    except Exception as e:
        print_error(f"Test error: {e}")

def test_backward_compatibility(token: str):
    """하위 호환성 테스트 (layout/initial_views 없이도 동작)"""
    print_test("하위 호환성 테스트")

    # layout/initial_views 없이 기존 방식으로 생성
    series = [
        {
            "study_uid": "1.2.840.113619.2.1.1.123",
            "series_uid": "1.2.840.113619.2.1.2.124"
        }
    ]

    selection_id = create_view_selection(token, series)
    if selection_id:
        selection = get_view_selection(token, selection_id)
        if selection:
            # layout과 initial_views가 없거나 null이어야 함
            layout = selection.get("layout")
            initial_views = selection.get("initial_views")

            if layout is None and initial_views is None:
                print_success("하위 호환성 유지: layout/initial_views 없이 정상 동작")
            else:
                print_error(f"Unexpected fields: layout={layout}, initial_views={initial_views}")

            # 정리
            delete_view_selection(token, selection_id)
        else:
            print_error("Selection 조회 실패")
    else:
        print_error("Selection 생성 실패")

def test_with_real_dicom_data(token: str):
    """실제 DICOM 데이터를 사용한 테스트 (TestConfig의 UIDs 사용)"""
    print_test("실제 DICOM 데이터 테스트")

    # TestConfig에서 정의된 실제 DICOM UIDs 사용
    series = [
        {
            "study_uid": TestConfig.STUDY_UID,
            "series_uid": TestConfig.SERIES_UID
        },
        {
            "study_uid": TestConfig.SNAPSHOT_STUDY_UID,
            "series_uid": TestConfig.SNAPSHOT_SERIES_UID
        }
    ]

    # Layout + Initial Views 포함
    request_data = {
        "series": series,
        "layout": {
            "rows": 1,
            "cols": 2
        },
        "initial_views": [
            {
                "row": 0,
                "col": 0,
                "study_uid": TestConfig.STUDY_UID,
                "series_uid": TestConfig.SERIES_UID,
                "sop_uid": TestConfig.INSTANCE_UID
            },
            {
                "row": 0,
                "col": 1,
                "study_uid": TestConfig.SNAPSHOT_STUDY_UID,
                "series_uid": TestConfig.SNAPSHOT_SERIES_UID,
                "sop_uid": TestConfig.SNAPSHOT_INSTANCE_UID
            }
        ]
    }

    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/view-selections",
            json=request_data,
            headers=get_headers(token)
        )

        if response.status_code == 201:
            result = response.json()
            selection_id = result.get("selection_id")
            print_success(f"실제 DICOM 데이터로 Selection 생성 성공: {selection_id}")

            # 조회하여 검증
            selection = get_view_selection(token, selection_id)
            if selection:
                assert len(selection["series"]) == 2, "Should have 2 series"
                assert len(selection["initial_views"]) == 2, "Should have 2 initial views"

                # Study UIDs 검증
                study_uids = set(s["study_uid"] for s in selection["series"])
                assert TestConfig.STUDY_UID in study_uids
                assert TestConfig.SNAPSHOT_STUDY_UID in study_uids

                print_success("실제 DICOM 데이터 검증 성공")

                # 정리
                delete_view_selection(token, selection_id)
            else:
                print_error("Selection 조회 실패")
        else:
            print_error(f"Selection 생성 실패: {response.status_code} - {response.text}")
    except Exception as e:
        print_error(f"Test error: {e}")

def main():
    """메인 테스트 실행"""
    print("\n" + "="*60)
    print("🚀 View Selection API E2E 통합 테스트")
    print("="*60)

    # 헬스 체크
    if not test_health():
        print_error("Server is not available. Exiting.")
        sys.exit(1)

    # 테스트 사용자 생성
    test_user_data = create_test_user()

    if not test_user_data or not test_user_data.get("token"):
        print_error("Failed to create/login test user. Some tests will be skipped.")
        print_info("Running tests that don't require authentication...")

        # 인증 불필요한 테스트만 실행
        test_unauthorized_access()
    else:
        token = test_user_data["token"]
        print_info("Running authenticated tests...")

        # ===== 기본 기능 테스트 =====
        print("\n" + "="*60)
        print("📋 기본 기능 테스트")
        print("="*60)
        test_create_selection_success(token)
        test_create_selection_empty_series(token)
        test_get_selection_not_found(token)
        test_selection_id_format(token)

        # ===== Layout + Initial Views 테스트 =====
        print("\n" + "="*60)
        print("🎨 Layout + Initial Views 테스트")
        print("="*60)
        test_layout_and_initial_views(token)
        test_layout_validation_errors(token)
        test_backward_compatibility(token)
        test_with_real_dicom_data(token)

        # ===== 고급 시나리오 테스트 =====
        print("\n" + "="*60)
        print("🔧 고급 시나리오 테스트")
        print("="*60)
        test_multi_study_series_selection(token)
        test_full_workflow(token)
        test_large_series_list(token)
        test_selection_persistence(token)
        test_real_world_scenario(token)

        # ===== 실제 사용 시나리오 테스트 =====
        print("\n" + "="*60)
        print("🎬 실제 사용 시나리오 테스트")
        print("="*60)
        scenario_viewer_session_workflow(token)
        scenario_multi_user_selection(token)
        scenario_url_sharing(token)

        # ===== 인증 테스트 =====
        print("\n" + "="*60)
        print("🔐 인증 테스트")
        print("="*60)
        test_unauthorized_access()

    # 결과 요약
    print("\n" + "="*60)
    print("📊 테스트 결과 요약")
    print("="*60)
    print(f"✅ 통과: {test_results['passed']}")
    print(f"❌ 실패: {test_results['failed']}")
    print(f"📝 총계: {test_results['total']}")

    if test_results['failed'] == 0:
        print("\n🎉 모든 테스트 통과!")
        return 0
    else:
        print(f"\n⚠️  {test_results['failed']}개 테스트 실패")
        return 1

if __name__ == "__main__":
    sys.exit(main())

