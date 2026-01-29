#!/usr/bin/env python3
"""
Access Control E2E Test

RBAC 평가 로직 및 접근 제어 API를 테스트합니다:
1. Role-Capability Matrix 조회
2. Role-Permission Matrix 조회
3. User-Project Matrix 조회
4. Permission Check
5. Data Access Check
6. Access Logs
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter


class AccessControlE2ETest(BaseE2ETest):
    """Access Control E2E Test"""

    def __init__(self):
        super().__init__()
        self.admin_token = None
        self.test_user_id = None
        self.test_project_id = None

    def get_test_name(self) -> str:
        """테스트 이름 반환"""
        return "Access Control E2E Test"

    def setup(self):
        """테스트 환경 설정"""
        # 관리자 로그인
        TestPrinter.print_info("관리자 로그인 중...")
        login_response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/login",
            json={
                "username": "iaid-pacs-admin",
                "password": "Qlalfqjsgh1!"
            },
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if login_response.status_code == 200:
            self.admin_token = login_response.json().get("token")
            TestPrinter.print_success("관리자 로그인 성공")
        else:
            raise Exception(f"관리자 로그인 실패: {login_response.text}")
        
        # 테스트용 사용자 ID 및 프로젝트 ID 설정 (기존 데이터 사용)
        self.test_user_id = 1  # iaid-pacs-admin
        self.test_project_id = 1  # 기존 프로젝트

    def cleanup(self):
        """테스트 데이터 정리"""
        pass  # 기존 데이터를 사용하므로 cleanup 불필요

    def test_role_capability_matrix(self):
        """테스트 1: Role-Capability Matrix 조회"""
        TestPrinter.print_header("테스트 1: Role-Capability Matrix 조회")
        
        print("\n1️⃣  GET /api/roles/global/capabilities/matrix 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/roles/global/capabilities/matrix",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"Matrix 조회 실패: {response.text}"
        
        data = response.json()

        # 필수 필드 확인
        assert "roles" in data, "roles 필드 없음"
        assert "capabilities_by_category" in data, "capabilities_by_category 필드 없음"
        assert "assignments" in data, "assignments 필드 없음"

        # Capability 개수 계산
        total_capabilities = sum(len(caps) for caps in data['capabilities_by_category'].values())

        TestPrinter.print_success("Role-Capability Matrix 조회 성공")
        TestPrinter.print_info(f"Roles: {len(data['roles'])}개", indent=1)
        TestPrinter.print_info(f"Capabilities: {total_capabilities}개", indent=1)
        TestPrinter.print_info(f"Categories: {len(data['capabilities_by_category'])}개", indent=1)
        TestPrinter.print_info(f"Assignments: {len(data['assignments'])}개", indent=1)
        
        # 페이지네이션 테스트 (기본 엔드포인트가 페이지네이션 지원)
        print("\n2️⃣  페이지네이션 테스트 (page=1, size=5)...")
        response2 = requests.get(
            f"{TestConfig.BASE_URL}/api/roles/global/capabilities/matrix?page=1&size=5",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response2.status_code}")
        assert response2.status_code == 200, f"페이지네이션 실패: {response2.text}"

        data2 = response2.json()
        assert "roles" in data2, "roles 필드 없음"
        assert "pagination" in data2, "pagination 필드 없음"

        TestPrinter.print_success("페이지네이션 성공")
        TestPrinter.print_info(f"Current Page: {data2['pagination']['current_page']}", indent=1)
        TestPrinter.print_info(f"Page Size: {data2['pagination']['page_size']}", indent=1)
        TestPrinter.print_info(f"Total Items: {data2['pagination']['total_items']}", indent=1)
        TestPrinter.print_info(f"Total Pages: {data2['pagination']['total_pages']}", indent=1)
        TestPrinter.print_info(f"Roles in page: {len(data2['roles'])}개", indent=1)

    def test_role_permission_matrix(self):
        """테스트 2: Role-Permission Matrix 조회"""
        TestPrinter.print_header("테스트 2: Role-Permission Matrix 조회")

        print("\n1️⃣  GET /api/roles/global/permissions/matrix 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/roles/global/permissions/matrix",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"Matrix 조회 실패: {response.text}"

        data = response.json()

        # 필수 필드 확인
        assert "roles" in data, "roles 필드 없음"
        assert "permissions_by_category" in data, "permissions_by_category 필드 없음"
        assert "assignments" in data, "assignments 필드 없음"

        # Permission 개수 계산
        total_permissions = sum(len(perms) for perms in data['permissions_by_category'].values())

        TestPrinter.print_success("Role-Permission Matrix 조회 성공")
        TestPrinter.print_info(f"Roles: {len(data['roles'])}개", indent=1)
        TestPrinter.print_info(f"Permissions: {total_permissions}개", indent=1)
        TestPrinter.print_info(f"Categories: {len(data['permissions_by_category'])}개", indent=1)
        TestPrinter.print_info(f"Assignments: {len(data['assignments'])}개", indent=1)

    def test_user_project_matrix(self):
        """테스트 3: User-Project Matrix 조회"""
        TestPrinter.print_header("테스트 3: User-Project Matrix 조회")

        print("\n1️⃣  GET /api/user-project-matrix 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/user-project-matrix",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"Matrix 조회 실패: {response.text}"

        data = response.json()

        # 필수 필드 확인
        assert "projects" in data, "projects 필드 없음"
        assert "matrix" in data, "matrix 필드 없음"

        TestPrinter.print_success("User-Project Matrix 조회 성공")
        TestPrinter.print_info(f"Projects: {len(data['projects'])}개", indent=1)
        TestPrinter.print_info(f"Matrix entries: {len(data['matrix'])}개", indent=1)

    def test_permission_check(self):
        """테스트 4: Permission Check"""
        TestPrinter.print_header("테스트 4: Permission Check")

        print("\n1️⃣  POST /api/access-control/permissions/check 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}

        # 권한 체크 요청
        check_data = {
            "user_id": self.test_user_id,
            "project_id": self.test_project_id,
            "capability": "VIEW_DICOM"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/access-control/permissions/check",
            json=check_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("Permission Check 성공")
            TestPrinter.print_info(f"User ID: {check_data['user_id']}", indent=1)
            TestPrinter.print_info(f"Project ID: {check_data['project_id']}", indent=1)
            TestPrinter.print_info(f"Capability: {check_data['capability']}", indent=1)
            TestPrinter.print_info(f"Allowed: {data.get('allowed', 'N/A')}", indent=1)
        else:
            TestPrinter.print_warning(f"Permission Check 응답: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_user_permissions(self):
        """테스트 5: User Permissions 조회"""
        TestPrinter.print_header("테스트 5: User Permissions 조회")

        print(f"\n1️⃣  GET /api/access-control/permissions/user/{self.test_user_id}/project/{self.test_project_id} 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}

        response = requests.get(
            f"{TestConfig.BASE_URL}/api/access-control/permissions/user/{self.test_user_id}/project/{self.test_project_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("User Permissions 조회 성공")

            if isinstance(data, list):
                TestPrinter.print_info(f"Permissions: {len(data)}개", indent=1)
                if len(data) > 0:
                    TestPrinter.print_info(f"첫 번째 권한: {data[0]}", indent=1)
            else:
                TestPrinter.print_info(f"응답: {data}", indent=1)
        else:
            TestPrinter.print_warning(f"User Permissions 조회 응답: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_project_access(self):
        """테스트 6: Project Access Check"""
        TestPrinter.print_header("테스트 6: Project Access Check")

        print(f"\n1️⃣  GET /api/access-control/access/user/{self.test_user_id}/project/{self.test_project_id} 호출...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}

        response = requests.get(
            f"{TestConfig.BASE_URL}/api/access-control/access/user/{self.test_user_id}/project/{self.test_project_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("Project Access Check 성공")
            TestPrinter.print_info(f"Can Access: {data.get('can_access', 'N/A')}", indent=1)
            TestPrinter.print_info(f"Reason: {data.get('reason', 'N/A')}", indent=1)
        else:
            TestPrinter.print_warning(f"Project Access Check 응답: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_access_logs(self):
        """테스트 7: Access Logs 조회"""
        TestPrinter.print_header("테스트 7: Access Logs 조회")

        headers = {"Authorization": f"Bearer {self.admin_token}"}

        # User Access Logs
        print(f"\n1️⃣  GET /api/access-control/logs/user/{self.test_user_id} 호출...")
        response1 = requests.get(
            f"{TestConfig.BASE_URL}/api/access-control/logs/user/{self.test_user_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response1.status_code}")

        if response1.status_code == 200:
            data1 = response1.json()
            TestPrinter.print_success("User Access Logs 조회 성공")

            if isinstance(data1, list):
                TestPrinter.print_info(f"Logs: {len(data1)}개", indent=1)
            else:
                TestPrinter.print_info(f"응답: {data1}", indent=1)
        else:
            TestPrinter.print_warning(f"User Access Logs 응답: {response1.status_code}")

        # Project Access Logs
        print(f"\n2️⃣  GET /api/access-control/logs/project/{self.test_project_id} 호출...")
        response2 = requests.get(
            f"{TestConfig.BASE_URL}/api/access-control/logs/project/{self.test_project_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response2.status_code}")

        if response2.status_code == 200:
            data2 = response2.json()
            TestPrinter.print_success("Project Access Logs 조회 성공")

            if isinstance(data2, list):
                TestPrinter.print_info(f"Logs: {len(data2)}개", indent=1)
            else:
                TestPrinter.print_info(f"응답: {data2}", indent=1)
        else:
            TestPrinter.print_warning(f"Project Access Logs 응답: {response2.status_code}")

    def run_tests(self):
        """테스트 실행"""
        self.test_role_capability_matrix()
        self.test_role_permission_matrix()
        self.test_user_project_matrix()
        self.test_permission_check()
        self.test_user_permissions()
        self.test_project_access()
        self.test_access_logs()


if __name__ == "__main__":
    test = AccessControlE2ETest()
    test.run()
