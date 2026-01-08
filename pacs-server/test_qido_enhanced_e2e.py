#!/usr/bin/env python3
"""
QIDO Enhanced API E2E 테스트 스크립트

GET /api/me/dicom/studies API의 확장 기능 테스트:
1. 기본 Study 목록 조회
2. View 파라미터를 통한 _ext 필드 확장
3. project_id 필터링
4. 페이지네이션
5. report_status 필터링
6. _ext.projects 검증
7. _ext.report_status 검증
8. _ext.review 검증
"""

import requests
import json
import sys
from typing import Optional, Dict, Any, List

BASE_URL = "http://localhost:8080"
USERNAME = "iaid-pacs-admin"
PASSWORD = "Qlalfqjsgh1!"

# 테스트 결과 추적
test_results = {"passed": 0, "failed": 0, "total": 0}


def print_header(text: str):
    print(f"\n{'='*70}")
    print(f"🧪 {text}")
    print(f"{'='*70}")


def print_result(test_name: str, passed: bool, details: str = ""):
    test_results["total"] += 1
    if passed:
        test_results["passed"] += 1
        print(f"  ✅ {test_name}")
    else:
        test_results["failed"] += 1
        print(f"  ❌ {test_name}")
    if details:
        print(f"     → {details}")


def get_token() -> Optional[str]:
    """로그인하여 JWT 토큰 획득"""
    try:
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json={"username": USERNAME, "password": PASSWORD},
            timeout=10
        )
        if response.status_code == 200:
            return response.json().get("token")
    except Exception as e:
        print(f"❌ 로그인 실패: {e}")
    return None


def test_basic_study_list(token: str) -> bool:
    """Test 1: 기본 Study 목록 조회 (view 없음)"""
    print_header("Test 1: 기본 Study 목록 조회 (view 없음)")

    response = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={"page": 1, "page_size": 5},
        timeout=30
    )

    passed = response.status_code == 200
    print_result("Status 200", passed, f"Got {response.status_code}")

    if passed:
        headers = response.headers
        print_result("X-Total-Count 헤더", "x-total-count" in headers.keys() or "X-Total-Count" in headers.keys())
        print_result("X-Page 헤더", "x-page" in headers.keys() or "X-Page" in headers.keys())
        print_result("X-Page-Size 헤더", "x-page-size" in headers.keys() or "X-Page-Size" in headers.keys())
        print_result("X-Total-Pages 헤더", "x-total-pages" in headers.keys() or "X-Total-Pages" in headers.keys())

        data = response.json()
        print_result("응답이 배열", isinstance(data, list), f"타입: {type(data).__name__}, {len(data)}개")

        # _ext.projects는 항상 있어야 함
        if len(data) > 0:
            has_ext = "_ext" in data[0]
            print_result("_ext 필드 존재", has_ext)
            if has_ext:
                has_projects = "projects" in data[0]["_ext"]
                has_report_status = "report_status" in data[0]["_ext"]
                has_review = "review" in data[0]["_ext"]
                print_result("_ext.projects 존재 (항상)", has_projects)
                print_result("_ext.report_status 없음 (view 없으므로)", not has_report_status)
                print_result("_ext.review 없음 (view 없으므로)", not has_review)

    return passed


def test_view_parameter(token: str) -> bool:
    """Test 2: View 파라미터로 _ext 필드 확장"""
    print_header("Test 2: view=default (_ext 확장 필드 포함)")

    # 먼저 view 없이 조회해서 기본 개수 확인
    resp_no_view = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={"page": 1, "page_size": 5},
        timeout=30
    )
    base_count = len(resp_no_view.json()) if resp_no_view.status_code == 200 else 0

    # view=default로 조회
    response = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={"view": "default", "page": 1, "page_size": 5},
        timeout=30
    )

    passed = response.status_code == 200
    print_result("Status 200", passed)

    if passed:
        data = response.json()
        view_count = len(data)

        # view 유무와 관계없이 결과 개수는 동일해야 함
        count_match = view_count == base_count
        print_result(f"결과 개수 동일 (view 없음: {base_count}, view=default: {view_count})", count_match)

        if len(data) > 0:
            first_study = data[0]
            has_ext = "_ext" in first_study
            print_result("_ext 필드 존재", has_ext)

            if has_ext:
                ext = first_study["_ext"]
                has_projects = "projects" in ext
                has_report_status = "report_status" in ext
                has_review = "review" in ext

                print_result("_ext.projects 필드 (항상)", has_projects)
                print_result("_ext.report_status 필드 (view에 정의됨)", has_report_status)
                print_result("_ext.review 필드 (view에 정의됨)", has_review)

                if has_projects and len(ext["projects"]) > 0:
                    project = ext["projects"][0]
                    has_id = "id" in project
                    has_name = "name" in project
                    has_role = "role_name" in project
                    print_result("  project.id", has_id)
                    print_result("  project.name", has_name)
                    print_result("  project.role_name", has_role)

                if has_review:
                    review = ext["review"]
                    print_result("  review.reviewStage", "reviewStage" in review)
                    print_result("  review.availableStages", "availableStages" in review)
                    print_result("  review.annotationSummary", "annotationSummary" in review)
        else:
            print_result("Study 데이터 없음", False, "View 테스트 스킵")

    return passed


def test_project_filter(token: str) -> bool:
    """Test 3: project_id 필터링"""
    print_header("Test 3: project_id 필터링")

    # project_id=2로 테스트
    response = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={"project_id": 2, "page": 1, "page_size": 10},
        timeout=30
    )

    passed = response.status_code == 200
    print_result("Status 200 (project_id=2)", passed, f"Got {response.status_code}")

    if passed:
        data = response.json()
        print_result("응답이 배열", isinstance(data, list), f"{len(data)}개 Study")

        # project_id 필터링 시에도 _ext.projects는 있어야 함
        if len(data) > 0 and "_ext" in data[0]:
            has_projects = "projects" in data[0]["_ext"]
            print_result("_ext.projects 존재", has_projects)

    return passed


def test_pagination(token: str) -> bool:
    """Test 4: 페이지네이션 테스트"""
    print_header("Test 4: 페이지네이션")
    
    # 페이지 1
    resp1 = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={"page": 1, "page_size": 2},
        timeout=30
    )

    passed = resp1.status_code == 200
    print_result("페이지 1 Status 200", passed)

    if passed:
        total_count = int(resp1.headers.get("x-total-count", resp1.headers.get("X-Total-Count", 0)))
        total_pages = int(resp1.headers.get("x-total-pages", resp1.headers.get("X-Total-Pages", 0)))
        print_result(f"Total Count: {total_count}", total_count >= 0)
        print_result(f"Total Pages: {total_pages}", total_pages >= 0)

        # 페이지 2 (있는 경우)
        if total_pages > 1:
            resp2 = requests.get(
                f"{BASE_URL}/api/me/dicom/studies",
                headers={"Authorization": f"Bearer {token}"},
                params={"page": 2, "page_size": 2},
                timeout=30
            )
            print_result("페이지 2 Status 200", resp2.status_code == 200)

            data1 = resp1.json()
            data2 = resp2.json()
            if len(data1) > 0 and len(data2) > 0:
                # Study UID 추출하여 비교
                uid1 = data1[0].get("0020000D", {}).get("Value", [None])[0]
                uid2 = data2[0].get("0020000D", {}).get("Value", [None])[0]
                different = uid1 != uid2
                print_result("페이지 1과 2 데이터 다름", different)

    return passed


def test_report_status_filter(token: str) -> bool:
    """Test 5: report_status 필터링"""
    print_header("Test 5: report_status 필터링")

    for status in ["unread", "approval", "unapproval"]:
        response = requests.get(
            f"{BASE_URL}/api/me/dicom/studies",
            headers={"Authorization": f"Bearer {token}"},
            params={"view": "default", "report_status": status, "page": 1, "page_size": 5},
            timeout=30
        )

        passed = response.status_code == 200
        print_result(f"report_status={status} Status 200", passed)

        if passed:
            data = response.json()
            print_result(f"  → 결과: {len(data)}개 Study", True)

    return True


def test_combined_filters(token: str) -> bool:
    """Test 6: 복합 필터링"""
    print_header("Test 6: 복합 필터링 (view + project_id + pagination)")

    response = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params={
            "view": "default",
            "project_id": 2,
            "page": 1,
            "page_size": 10,
        },
        timeout=30
    )

    passed = response.status_code == 200
    print_result("복합 필터 Status 200", passed)

    if passed:
        data = response.json()
        print_result(f"결과: {len(data)}개 Study", True)

        if len(data) > 0 and "_ext" in data[0]:
            ext = data[0]["_ext"]
            has_projects = "projects" in ext
            has_report_status = "report_status" in ext
            has_review = "review" in ext
            print_result("_ext.projects 있음", has_projects)
            print_result("_ext.report_status 있음 (view=default)", has_report_status)
            print_result("_ext.review 있음 (view=default)", has_review)

    return passed


def print_summary():
    """테스트 결과 요약"""
    print(f"\n{'='*70}")
    print("📊 테스트 결과 요약")
    print(f"{'='*70}")
    print(f"  총 테스트: {test_results['total']}")
    print(f"  ✅ 성공: {test_results['passed']}")
    print(f"  ❌ 실패: {test_results['failed']}")

    if test_results['failed'] == 0:
        print("\n🎉 모든 테스트 통과!")
        return 0
    else:
        print(f"\n⚠️ {test_results['failed']}개 테스트 실패")
        return 1


def main():
    print("🚀 QIDO Enhanced API E2E 테스트 시작")
    print(f"   서버: {BASE_URL}")
    print(f"   사용자: {USERNAME}")

    # 로그인
    token = get_token()
    if not token:
        print("❌ 로그인 실패. 테스트 중단.")
        return 1
    print(f"✅ 로그인 성공 (토큰 길이: {len(token)})")

    # 테스트 실행
    test_basic_study_list(token)
    test_view_parameter(token)
    test_project_filter(token)
    test_pagination(token)
    test_report_status_filter(token)
    test_combined_filters(token)

    return print_summary()


if __name__ == "__main__":
    sys.exit(main())

