#!/usr/bin/env python3
"""
Study List View API E2E 시나리오 테스트 스크립트

이 스크립트는 Study List View 관리 기능을 테스트합니다:
1. View 목록 조회 (GET /api/study-list-views)
2. View 상세 조회 (GET /api/study-list-views/{view_id})
3. View 생성 (POST /api/study-list-views)
4. View 수정 (PUT /api/study-list-views/{view_id})
5. View 삭제 (DELETE /api/study-list-views/{view_id})
6. 필드 정의 목록 조회 (GET /api/study-list-views/field-defs)
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

def get_headers(token: Optional[str] = None) -> Dict[str, str]:
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

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

def login_user(username: str = "iaid-pacs-admin", password: str = "Qlalfqjsgh1!") -> Optional[str]:
    """사용자 로그인하여 JWT 토큰 얻기"""
    print_info(f"Logging in user: {username}...")
    try:
        login_data = {"username": username, "password": password}
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json=login_data,
            headers={"Content-Type": "application/json"},
            timeout=10
        )

        if response.status_code == 200:
            result = response.json()
            token = result.get("token") or result.get("access_token")
            if token:
                print_success(f"Login successful for {username}")
                return token
            else:
                print_error(f"Token not found in response: {result}")
                return None
        else:
            print_error(f"Login failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"Login error: {e}")
        return None

# ========================================
# View API 헬퍼 함수
# ========================================

def list_views(token: str, scope_type: str = None, scope_id: str = None) -> Optional[Dict]:
    """View 목록 조회"""
    try:
        params = {}
        if scope_type:
            params["scopeType"] = scope_type
        if scope_id:
            params["scopeId"] = scope_id

        response = requests.get(
            f"{BASE_URL}/api/study-list-views",
            params=params,
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 200:
            return response.json()
        else:
            print_error(f"List views failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"List views error: {e}")
        return None

def get_view(token: str, view_id: str) -> Optional[Dict]:
    """View 상세 조회"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/study-list-views/{view_id}",
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 200:
            return response.json()
        elif response.status_code == 404:
            print_info(f"View not found: {view_id}")
            return None
        else:
            print_error(f"Get view failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"Get view error: {e}")
        return None

def create_view(token: str, view_data: Dict) -> Optional[Dict]:
    """View 생성"""
    try:
        response = requests.post(
            f"{BASE_URL}/api/study-list-views",
            json=view_data,
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 201:
            result = response.json()
            print_success(f"View created: {result.get('viewId')}")
            return result
        else:
            print_error(f"Create view failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"Create view error: {e}")
        return None


def update_view(token: str, view_id: str, update_data: Dict) -> Optional[Dict]:
    """View 수정"""
    try:
        response = requests.put(
            f"{BASE_URL}/api/study-list-views/{view_id}",
            json=update_data,
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 200:
            result = response.json()
            print_success(f"View updated: {view_id}")
            return result
        else:
            print_error(f"Update view failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"Update view error: {e}")
        return None

def delete_view(token: str, view_id: str) -> bool:
    """View 삭제"""
    try:
        response = requests.delete(
            f"{BASE_URL}/api/study-list-views/{view_id}",
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 204:
            print_success(f"View deleted: {view_id}")
            return True
        elif response.status_code == 403:
            print_info(f"View deletion forbidden (system view): {view_id}")
            return False
        else:
            print_error(f"Delete view failed: {response.status_code} - {response.text[:200]}")
            return False
    except Exception as e:
        print_error(f"Delete view error: {e}")
        return False

def list_field_defs(token: str, source: str = None, level: str = None) -> Optional[Dict]:
    """필드 정의 목록 조회"""
    try:
        params = {}
        if source:
            params["source"] = source
        if level:
            params["level"] = level

        response = requests.get(
            f"{BASE_URL}/api/study-list-views/field-defs",
            params=params,
            headers=get_headers(token),
            timeout=10
        )

        if response.status_code == 200:
            return response.json()
        else:
            print_error(f"List field defs failed: {response.status_code} - {response.text[:200]}")
            return None
    except Exception as e:
        print_error(f"List field defs error: {e}")
        return None

# ========================================
# 테스트 케이스
# ========================================

def test_list_field_defs(token: str):
    """필드 정의 목록 조회 테스트"""
    print_test("List Field Definitions")

    # 전체 필드 정의 조회
    result = list_field_defs(token)
    if result:
        print_success(f"Total field definitions: {result.get('total', 0)}")
        items = result.get("items", [])
        if items:
            print_info(f"First 3 fields: {[f.get('key') for f in items[:3]]}")
    else:
        print_error("Failed to list field definitions")
        return

    # DICOM 필드만 조회
    dicom_result = list_field_defs(token, source="dicom")
    if dicom_result:
        print_success(f"DICOM field definitions: {dicom_result.get('total', 0)}")

    # Extension 필드만 조회
    ext_result = list_field_defs(token, source="extension")
    if ext_result:
        print_success(f"Extension field definitions: {ext_result.get('total', 0)}")

    # Study 레벨 필드만 조회
    study_result = list_field_defs(token, level="study")
    if study_result:
        print_success(f"Study-level field definitions: {study_result.get('total', 0)}")

def test_list_views(token: str):
    """View 목록 조회 테스트"""
    print_test("List Views")

    result = list_views(token)
    if result:
        print_success(f"Total views: {result.get('total', 0)}")
        items = result.get("items", [])
        for view in items[:3]:
            print_info(f"  - {view.get('viewId')}: {view.get('viewName')} (system: {view.get('isSystem')})")
    else:
        print_error("Failed to list views")

def test_create_view(token: str) -> Optional[str]:
    """View 생성 테스트"""
    print_test("Create View")

    timestamp = int(time.time())
    view_data = {
        "viewId": f"test_view_{timestamp}",
        "viewName": f"테스트 View {timestamp}",
        "description": "E2E 테스트용 View",
        "scopeType": "user",
        "fields": [
            {"source": "dicom", "key": "PatientName", "displayOrder": 1, "visible": True, "pinned": False},
            {"source": "dicom", "key": "StudyDate", "displayOrder": 2, "visible": True, "pinned": True},
            {"source": "dicom", "key": "Modality", "displayOrder": 3, "visible": True, "pinned": False}
        ]
    }

    result = create_view(token, view_data)
    if result:
        view_id = result.get("viewId")
        print_success(f"View created successfully: {view_id}")
        return view_id
    else:
        print_error("Failed to create view")
        return None

def test_get_view(token: str, view_id: str):
    """View 상세 조회 테스트"""
    print_test("Get View Detail")

    result = get_view(token, view_id)
    if result:
        print_success(f"View retrieved: {result.get('viewId')}")
        print_info(f"  Name: {result.get('viewName')}")
        print_info(f"  Fields count: {len(result.get('fields', []))}")
        print_info(f"  Scope: {result.get('scopeType')}")
    else:
        print_error(f"Failed to get view: {view_id}")

def test_update_view(token: str, view_id: str):
    """View 수정 테스트"""
    print_test("Update View")

    update_data = {
        "viewName": f"수정된 View {int(time.time())}",
        "description": "수정된 설명",
        "fields": [
            {"source": "dicom", "key": "PatientName", "displayOrder": 1, "visible": True, "pinned": True},
            {"source": "dicom", "key": "StudyDate", "displayOrder": 2, "visible": True, "pinned": True},
            {"source": "dicom", "key": "Modality", "displayOrder": 3, "visible": True, "pinned": False},
            {"source": "dicom", "key": "PatientID", "displayOrder": 4, "visible": True, "pinned": False}
        ]
    }

    result = update_view(token, view_id, update_data)
    if result:
        print_success(f"View updated: {result.get('viewId')}")
        print_info(f"  New name: {result.get('viewName')}")
        print_info(f"  Fields count: {len(result.get('fields', []))}")
    else:
        print_error(f"Failed to update view: {view_id}")

def test_delete_view(token: str, view_id: str):
    """View 삭제 테스트"""
    print_test("Delete View")

    if delete_view(token, view_id):
        # 삭제 후 조회 시도
        result = get_view(token, view_id)
        if result is None:
            print_success("View properly deleted (not found after deletion)")
        else:
            print_error("View should not exist after deletion")
    else:
        print_error(f"Failed to delete view: {view_id}")

def test_view_not_found(token: str):
    """존재하지 않는 View 조회 테스트"""
    print_test("Get Non-Existent View")

    result = get_view(token, "nonexistent_view_123")
    if result is None:
        print_success("Correctly returned None for non-existent view")
    else:
        print_error("Should return None for non-existent view")

def test_full_crud_workflow(token: str):
    """전체 CRUD 워크플로우 테스트"""
    print_test("Full CRUD Workflow")

    # 1. Create
    print_info("Step 1: Create View...")
    timestamp = int(time.time())
    view_data = {
        "viewId": f"workflow_test_{timestamp}",
        "viewName": f"Workflow Test View {timestamp}",
        "description": "Full CRUD workflow test",
        "scopeType": "user",
        "fields": [
            {"source": "dicom", "key": "PatientName", "displayOrder": 1, "visible": True, "pinned": False}
        ]
    }

    created = create_view(token, view_data)
    if not created:
        print_error("Workflow failed at Create step")
        return
    view_id = created.get("viewId")

    # 2. Read
    print_info("Step 2: Read View...")
    fetched = get_view(token, view_id)
    if not fetched:
        print_error("Workflow failed at Read step")
        delete_view(token, view_id)
        return

    # 3. Update
    print_info("Step 3: Update View...")
    update_data = {
        "viewName": f"Updated Workflow Test {timestamp}",
        "fields": [
            {"source": "dicom", "key": "PatientName", "displayOrder": 1, "visible": True, "pinned": True},
            {"source": "dicom", "key": "StudyDate", "displayOrder": 2, "visible": True, "pinned": False}
        ]
    }
    updated = update_view(token, view_id, update_data)
    if not updated:
        print_error("Workflow failed at Update step")
        delete_view(token, view_id)
        return

    # 4. Delete
    print_info("Step 4: Delete View...")
    if delete_view(token, view_id):
        print_success("Full CRUD workflow completed successfully!")
    else:
        print_error("Workflow failed at Delete step")

def main():
    """메인 테스트 실행"""
    print("\n" + "="*60)
    print("🚀 Study List View API E2E 시나리오 테스트")
    print("="*60)

    # 헬스 체크
    if not test_health():
        print_error("Server is not available. Exiting.")
        sys.exit(1)

    # 로그인
    token = login_user()
    if not token:
        print_error("Failed to login. Exiting.")
        sys.exit(1)

    # 테스트 실행
    test_list_field_defs(token)
    test_list_views(token)
    test_view_not_found(token)

    # CRUD 테스트
    view_id = test_create_view(token)
    if view_id:
        test_get_view(token, view_id)
        test_update_view(token, view_id)
        test_delete_view(token, view_id)

    # 전체 워크플로우 테스트
    test_full_crud_workflow(token)

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
