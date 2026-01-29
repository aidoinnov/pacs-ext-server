#!/usr/bin/env python3
"""
Study 할당/해제 후 check_assignment_for_project API 즉시 반영 테스트

테스트 시나리오:
1. Study 할당 전: is_assigned = false 확인
2. Study 할당: POST /api/projects/{project_id}/studies/assign
3. 할당 후 즉시 확인: is_assigned = true 확인
4. Study 할당 해제: DELETE /api/projects/{project_id}/studies/{study_uid}/unassign
5. 해제 후 즉시 확인: is_assigned = false 확인
6. 재할당: POST /api/projects/{project_id}/studies/assign
7. 재할당 후 즉시 확인: is_assigned = true 확인
"""

import requests
import sys
import time
from typing import Optional

BASE_URL = "http://localhost:8080"
ADMIN_USER = "iaid-pacs-admin"
ADMIN_PASSWORD = "Qlalfqjsgh1!"

# 테스트할 Study UID
TEST_STUDY_UID = "1.2.410.200022.500.12252244131"
PROJECT_ID = 2


class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'


def print_success(msg):
    print(f"{Colors.GREEN}✅ {msg}{Colors.RESET}")


def print_error(msg):
    print(f"{Colors.RED}❌ {msg}{Colors.RESET}")


def print_info(msg):
    print(f"{Colors.BLUE}ℹ️  {msg}{Colors.RESET}")


def print_warning(msg):
    print(f"{Colors.YELLOW}⚠️  {msg}{Colors.RESET}")


def login() -> str:
    """로그인하여 토큰 반환"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": ADMIN_USER, "password": ADMIN_PASSWORD}
    )
    if response.status_code != 200:
        raise Exception(f"로그인 실패: {response.status_code}")
    return response.json()["token"]


def check_study_assignment(token: str, study_uid: str) -> Optional[bool]:
    """Study 할당 여부 확인"""
    response = requests.get(
        f"{BASE_URL}/api/dicom/studies",
        params={
            "check_assignment_for_project": PROJECT_ID,
            "page": 1,
            "page_size": 50
        },
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if response.status_code != 200:
        raise Exception(f"Study 조회 실패: {response.status_code}")
    
    studies = response.json()
    for study in studies:
        if study.get("0020000D", {}).get("Value", [None])[0] == study_uid:
            return study.get("is_assigned")
    
    return None


def assign_study(token: str, study_uid: str) -> bool:
    """Study 할당"""
    response = requests.post(
        f"{BASE_URL}/api/projects/{PROJECT_ID}/studies/assign",
        json={"study_uid": study_uid},
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if response.status_code == 409:
        print_warning("이미 할당된 Study")
        return True
    
    if response.status_code not in [200, 201]:
        raise Exception(f"Study 할당 실패: {response.status_code} - {response.text}")
    
    return True


def unassign_study(token: str, study_uid: str) -> bool:
    """Study 할당 해제"""
    response = requests.delete(
        f"{BASE_URL}/api/projects/{PROJECT_ID}/studies/{study_uid}/unassign",
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if response.status_code not in [200, 204]:
        raise Exception(f"Study 할당 해제 실패: {response.status_code} - {response.text}")
    
    return True


def run_test():
    """E2E 테스트 실행"""
    print("=" * 70)
    print("Study 할당/해제 즉시 반영 E2E 테스트")
    print("=" * 70)
    print()
    
    test_results = []
    
    try:
        # 1. 로그인
        print("1️⃣  로그인...")
        token = login()
        print_success("로그인 성공")
        print()
        
        print_info(f"Test Study UID: {TEST_STUDY_UID}")
        print_info(f"Project ID: {PROJECT_ID}")
        print()

        # 2. 초기 상태 확인 (할당 해제되어 있어야 함)
        print("2️⃣  초기 상태 확인 (할당 해제)...")
        try:
            unassign_study(token, TEST_STUDY_UID)
            print_info("기존 할당 해제 완료")
        except:
            print_info("이미 할당 해제되어 있음")

        time.sleep(0.5)  # 약간의 대기

        is_assigned = check_study_assignment(token, TEST_STUDY_UID)
        if is_assigned is None:
            print_error("Study를 찾을 수 없음")
            test_results.append(("초기 상태 확인", False))
        elif is_assigned == False:
            print_success("초기 상태: is_assigned = false ✅")
            test_results.append(("초기 상태 확인", True))
        else:
            print_error(f"초기 상태: is_assigned = {is_assigned} (예상: false)")
            test_results.append(("초기 상태 확인", False))
        print()

        # 3. Study 할당
        print("3️⃣  Study 할당...")
        assign_study(token, TEST_STUDY_UID)
        print_success("Study 할당 완료")
        print()

        # 4. 할당 후 즉시 확인 (is_assigned = true 예상)
        print("4️⃣  할당 후 즉시 확인...")
        is_assigned_after_assign = check_study_assignment(token, TEST_STUDY_UID)

        if is_assigned_after_assign is None:
            print_error("Study를 찾을 수 없음")
            test_results.append(("할당 후 즉시 반영", False))
        elif is_assigned_after_assign == True:
            print_success("할당 후: is_assigned = true ✅ (즉시 반영됨!)")
            test_results.append(("할당 후 즉시 반영", True))
        else:
            print_error(f"할당 후: is_assigned = {is_assigned_after_assign} (예상: true)")
            test_results.append(("할당 후 즉시 반영", False))
        print()

        # 5. Study 할당 해제
        print("5️⃣  Study 할당 해제...")
        unassign_study(token, TEST_STUDY_UID)
        print_success("Study 할당 해제 완료")
        print()

        # 6. 할당 해제 후 즉시 확인 (is_assigned = false 예상)
        print("6️⃣  할당 해제 후 즉시 확인...")
        is_assigned_after_unassign = check_study_assignment(token, TEST_STUDY_UID)

        if is_assigned_after_unassign is None:
            print_error("Study를 찾을 수 없음")
            test_results.append(("할당 해제 후 즉시 반영", False))
        elif is_assigned_after_unassign == False:
            print_success("할당 해제 후: is_assigned = false ✅ (즉시 반영됨!)")
            test_results.append(("할당 해제 후 즉시 반영", True))
        else:
            print_error(f"할당 해제 후: is_assigned = {is_assigned_after_unassign} (예상: false)")
            test_results.append(("할당 해제 후 즉시 반영", False))
        print()

        # 7. 재할당
        print("7️⃣  Study 재할당...")
        assign_study(token, TEST_STUDY_UID)
        print_success("Study 재할당 완료")
        print()

        # 8. 재할당 후 즉시 확인 (is_assigned = true 예상)
        print("8️⃣  재할당 후 즉시 확인...")
        is_assigned_after_reassign = check_study_assignment(token, TEST_STUDY_UID)

        if is_assigned_after_reassign is None:
            print_error("Study를 찾을 수 없음")
            test_results.append(("재할당 후 즉시 반영", False))
        elif is_assigned_after_reassign == True:
            print_success("재할당 후: is_assigned = true ✅ (즉시 반영됨!)")
            test_results.append(("재할당 후 즉시 반영", True))
        else:
            print_error(f"재할당 후: is_assigned = {is_assigned_after_reassign} (예상: true)")
            test_results.append(("재할당 후 즉시 반영", False))
        print()

    except Exception as e:
        print_error(f"테스트 실행 중 오류: {e}")
        import traceback
        traceback.print_exc()
        return False

    # 결과 요약
    print("=" * 70)
    print("테스트 결과 요약")
    print("=" * 70)
    print()

    for test_name, result in test_results:
        if result:
            print_success(f"{test_name}: 성공")
        else:
            print_error(f"{test_name}: 실패")

    print()
    total_tests = len(test_results)
    passed_tests = sum(1 for _, result in test_results if result)

    print(f"총 테스트: {total_tests}")
    print(f"성공: {passed_tests}")
    print(f"실패: {total_tests - passed_tests}")
    print()

    if passed_tests == total_tests:
        print_success(f"🎉 모든 테스트 통과! ({passed_tests}/{total_tests})")
        print()
        print_success("✅ Study 할당/해제가 check_assignment_for_project API에 즉시 반영됩니다!")
        return True
    else:
        print_error(f"일부 테스트 실패 ({passed_tests}/{total_tests})")
        return False


if __name__ == "__main__":
    success = run_test()
    sys.exit(0 if success else 1)

