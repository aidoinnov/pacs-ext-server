#!/usr/bin/env python3
"""
프로젝트 생성/삭제 후 목록 조회 즉시 반영 테스트

테스트 시나리오:
1. 초기 프로젝트 목록 조회 (개수 확인)
2. 새 프로젝트 생성
3. 목록 조회 → 개수 증가 확인 (즉시 반영)
4. 생성된 프로젝트가 목록에 있는지 확인
5. 프로젝트 삭제
6. 목록 조회 → 개수 감소 확인 (즉시 반영)
7. 삭제된 프로젝트가 목록에 없는지 확인
8. 여러 프로젝트 생성/삭제 반복 테스트
"""

import requests
import sys
import time
from typing import List, Dict, Optional

BASE_URL = "http://localhost:8080"
ADMIN_USER = "iaid-pacs-admin"
ADMIN_PASSWORD = "Qlalfqjsgh1!"


class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    RESET = '\033[0m'


def print_success(msg):
    print(f"{Colors.GREEN}✅ {msg}{Colors.RESET}")


def print_error(msg):
    print(f"{Colors.RED}❌ {msg}{Colors.RESET}")


def print_info(msg):
    print(f"{Colors.BLUE}ℹ️  {msg}{Colors.RESET}")


def print_warning(msg):
    print(f"{Colors.YELLOW}⚠️  {msg}{Colors.RESET}")


def print_step(msg):
    print(f"{Colors.CYAN}{msg}{Colors.RESET}")


def login() -> str:
    """로그인하여 토큰 반환"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": ADMIN_USER, "password": ADMIN_PASSWORD}
    )
    if response.status_code != 200:
        raise Exception(f"로그인 실패: {response.status_code}")
    return response.json()["token"]


def get_projects(token: str) -> tuple[List[Dict], int]:
    """프로젝트 목록 조회 (프로젝트 리스트, 전체 개수 반환)"""
    response = requests.get(
        f"{BASE_URL}/api/projects",
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if response.status_code != 200:
        raise Exception(f"프로젝트 목록 조회 실패: {response.status_code}")
    
    data = response.json()
    projects = data.get("projects", [])
    total = data.get("pagination", {}).get("total", len(projects))
    
    return projects, total


def create_project(token: str, name: str, description: str = "") -> int:
    """프로젝트 생성 (프로젝트 ID 반환)"""
    from datetime import date, timedelta

    today = date.today()
    end_date = today + timedelta(days=365)

    response = requests.post(
        f"{BASE_URL}/api/projects",
        json={
            "name": name,
            "description": description or f"E2E 테스트 프로젝트: {name}",
            "sponsor": "E2E Test Sponsor",
            "start_date": today.isoformat(),
            "end_date": end_date.isoformat(),
            "auto_complete": False
        },
        headers={"Authorization": f"Bearer {token}"}
    )

    if response.status_code not in [200, 201]:
        raise Exception(f"프로젝트 생성 실패: {response.status_code} - {response.text}")

    return response.json()["id"]


def delete_project(token: str, project_id: int) -> bool:
    """프로젝트 삭제"""
    response = requests.delete(
        f"{BASE_URL}/api/projects/{project_id}",
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if response.status_code not in [200, 204]:
        raise Exception(f"프로젝트 삭제 실패: {response.status_code} - {response.text}")
    
    return True


def find_project_by_id(projects: List[Dict], project_id: int) -> Optional[Dict]:
    """프로젝트 ID로 프로젝트 찾기"""
    for project in projects:
        if project.get("id") == project_id:
            return project
    return None


def run_test():
    """E2E 테스트 실행"""
    print("=" * 70)
    print("프로젝트 생성/삭제 즉시 반영 E2E 테스트")
    print("=" * 70)
    print()
    
    test_results = []
    created_project_ids = []
    
    try:
        # 1. 로그인
        print_step("1️⃣  로그인...")
        token = login()
        print_success("로그인 성공")
        print()
        
        # 2. 초기 프로젝트 목록 조회
        print_step("2️⃣  초기 프로젝트 목록 조회...")
        initial_projects, initial_count = get_projects(token)
        print_info(f"초기 프로젝트 개수: {initial_count}")
        print()

        # 3. 새 프로젝트 생성
        print_step("3️⃣  새 프로젝트 생성...")
        test_project_name = f"E2E_Test_Project_{int(time.time())}"
        new_project_id = create_project(token, test_project_name)
        created_project_ids.append(new_project_id)
        print_success(f"프로젝트 생성 완료 (ID: {new_project_id}, Name: {test_project_name})")
        print()

        # 4. 생성 후 즉시 목록 조회 (개수 증가 확인)
        print_step("4️⃣  생성 후 즉시 목록 조회...")
        after_create_projects, after_create_count = get_projects(token)

        print_info(f"생성 후 프로젝트 개수: {after_create_count}")
        print_info(f"개수 변화: {initial_count} → {after_create_count} (증가: {after_create_count - initial_count})")

        if after_create_count == initial_count + 1:
            print_success("개수가 1 증가했습니다! ✅ (즉시 반영됨)")
            test_results.append(("프로젝트 생성 후 개수 증가", True))
        else:
            print_error(f"개수가 예상과 다릅니다 (예상: {initial_count + 1}, 실제: {after_create_count})")
            test_results.append(("프로젝트 생성 후 개수 증가", False))
        print()

        # 5. 생성된 프로젝트가 목록에 있는지 확인
        print_step("5️⃣  생성된 프로젝트가 목록에 있는지 확인...")
        found_project = find_project_by_id(after_create_projects, new_project_id)

        if found_project:
            print_success(f"생성된 프로젝트를 목록에서 찾았습니다! ✅")
            print_info(f"  - ID: {found_project.get('id')}")
            print_info(f"  - Name: {found_project.get('name')}")
            test_results.append(("생성된 프로젝트 목록에 존재", True))
        else:
            print_error("생성된 프로젝트를 목록에서 찾을 수 없습니다")
            test_results.append(("생성된 프로젝트 목록에 존재", False))
        print()

        # 6. 프로젝트 삭제
        print_step("6️⃣  프로젝트 삭제...")
        delete_project(token, new_project_id)
        print_success(f"프로젝트 삭제 완료 (ID: {new_project_id})")
        created_project_ids.remove(new_project_id)
        print()

        # 7. 삭제 후 즉시 목록 조회 (개수 감소 확인)
        print_step("7️⃣  삭제 후 즉시 목록 조회...")
        after_delete_projects, after_delete_count = get_projects(token)

        print_info(f"삭제 후 프로젝트 개수: {after_delete_count}")
        print_info(f"개수 변화: {after_create_count} → {after_delete_count} (감소: {after_create_count - after_delete_count})")

        if after_delete_count == initial_count:
            print_success("개수가 초기 상태로 돌아왔습니다! ✅ (즉시 반영됨)")
            test_results.append(("프로젝트 삭제 후 개수 감소", True))
        else:
            print_error(f"개수가 예상과 다릅니다 (예상: {initial_count}, 실제: {after_delete_count})")
            test_results.append(("프로젝트 삭제 후 개수 감소", False))
        print()

        # 8. 삭제된 프로젝트가 목록에 없는지 확인
        print_step("8️⃣  삭제된 프로젝트가 목록에 없는지 확인...")
        deleted_project = find_project_by_id(after_delete_projects, new_project_id)

        if deleted_project is None:
            print_success("삭제된 프로젝트가 목록에 없습니다! ✅")
            test_results.append(("삭제된 프로젝트 목록에 없음", True))
        else:
            print_error("삭제된 프로젝트가 여전히 목록에 있습니다")
            test_results.append(("삭제된 프로젝트 목록에 없음", False))
        print()

        # 9. 여러 프로젝트 생성/삭제 반복 테스트
        print_step("9️⃣  여러 프로젝트 생성/삭제 반복 테스트...")
        print_info("3개의 프로젝트를 생성하고 삭제합니다...")
        print()

        batch_test_passed = True

        # 3개 프로젝트 생성
        batch_project_ids = []
        for i in range(3):
            project_name = f"E2E_Batch_Test_{int(time.time())}_{i}"
            project_id = create_project(token, project_name)
            batch_project_ids.append(project_id)
            created_project_ids.append(project_id)
            print_info(f"  프로젝트 {i+1} 생성: ID={project_id}")

        # 생성 후 개수 확인
        batch_after_create_projects, batch_after_create_count = get_projects(token)
        expected_count = initial_count + 3

        print_info(f"  생성 후 개수: {batch_after_create_count} (예상: {expected_count})")

        if batch_after_create_count == expected_count:
            print_success("  3개 프로젝트 생성 후 개수 정확! ✅")
        else:
            print_error(f"  개수 불일치 (예상: {expected_count}, 실제: {batch_after_create_count})")
            batch_test_passed = False

        # 3개 프로젝트 삭제
        for i, project_id in enumerate(batch_project_ids):
            delete_project(token, project_id)
            created_project_ids.remove(project_id)
            print_info(f"  프로젝트 {i+1} 삭제: ID={project_id}")

        # 삭제 후 개수 확인
        batch_after_delete_projects, batch_after_delete_count = get_projects(token)

        print_info(f"  삭제 후 개수: {batch_after_delete_count} (예상: {initial_count})")

        if batch_after_delete_count == initial_count:
            print_success("  3개 프로젝트 삭제 후 개수 정확! ✅")
        else:
            print_error(f"  개수 불일치 (예상: {initial_count}, 실제: {batch_after_delete_count})")
            batch_test_passed = False

        test_results.append(("여러 프로젝트 생성/삭제 반복", batch_test_passed))
        print()

    except Exception as e:
        print_error(f"테스트 실행 중 오류: {e}")
        import traceback
        traceback.print_exc()

        # 생성된 프로젝트 정리
        if created_project_ids:
            print_warning(f"생성된 프로젝트 정리 중... ({len(created_project_ids)}개)")
            for project_id in created_project_ids:
                try:
                    delete_project(token, project_id)
                    print_info(f"  프로젝트 {project_id} 삭제 완료")
                except:
                    pass

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
        print_success("✅ 프로젝트 생성/삭제가 목록 조회에 즉시 반영됩니다!")
        return True
    else:
        print_error(f"일부 테스트 실패 ({passed_tests}/{total_tests})")
        return False


if __name__ == "__main__":
    success = run_test()
    sys.exit(0 if success else 1)

