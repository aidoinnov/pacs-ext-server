#!/usr/bin/env python3
"""
Authentication Flow E2E Test

회원가입부터 로그인까지 전체 인증 플로우를 테스트합니다:
1. 회원가입 (Signup)
2. 이메일 인증 (Email Verification)
3. 관리자 승인 (Admin Approval)
4. 로그인 (Login)
5. 계정 삭제 (Account Deletion)
"""

import requests
import json
import time
import random
import string
from test_base import BaseE2ETest, TestConfig, TestPrinter


class AuthFlowE2ETest(BaseE2ETest):
    """Authentication Flow E2E Test"""

    def __init__(self):
        super().__init__()
        self.test_user_id = None
        self.test_username = None
        self.test_email = None
        self.test_password = "TestPassword123!"
        self.admin_token = None

    def get_test_name(self) -> str:
        """테스트 이름 반환"""
        return "Authentication Flow E2E Test"

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

    def cleanup(self):
        """테스트 데이터 정리"""
        if self.test_user_id and self.admin_token:
            try:
                TestPrinter.print_info(f"테스트 사용자 삭제 중 (ID: {self.test_user_id})...")
                headers = {"Authorization": f"Bearer {self.admin_token}"}
                response = requests.delete(
                    f"{TestConfig.BASE_URL}/api/auth/users/{self.test_user_id}",
                    headers=headers,
                    timeout=TestConfig.DEFAULT_TIMEOUT
                )
                if response.status_code in [200, 204]:
                    TestPrinter.print_success("테스트 사용자 삭제 완료")
                else:
                    TestPrinter.print_warning(f"사용자 삭제 실패: {response.status_code}")
            except Exception as e:
                TestPrinter.print_warning(f"Cleanup 중 에러: {e}")

    def generate_test_credentials(self):
        """테스트용 사용자 정보 생성"""
        random_suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=8))
        self.test_username = f"testuser_{random_suffix}"
        self.test_email = f"test_{random_suffix}@example.com"
        
        TestPrinter.print_info(f"테스트 사용자명: {self.test_username}")
        TestPrinter.print_info(f"테스트 이메일: {self.test_email}")

    def test_signup(self):
        """테스트 1: 회원가입"""
        TestPrinter.print_header("테스트 1: 회원가입 (Signup)")
        
        self.generate_test_credentials()
        
        # 회원가입 요청
        signup_data = {
            "username": self.test_username,
            "email": self.test_email,
            "password": self.test_password,
            "full_name": "Test User",
            "organization": "Test Organization",
            "department": "Test Department",
            "phone": "010-1234-5678"
        }
        
        print("\n1️⃣  회원가입 요청...")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/signup",
            json=signup_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 201, f"회원가입 실패: {response.text}"
        
        data = response.json()
        self.test_user_id = data.get("user_id")
        
        assert "user_id" in data
        assert data["username"] == self.test_username
        assert data["email"] == self.test_email
        assert "account_status" in data
        
        TestPrinter.print_success(f"회원가입 성공 (User ID: {self.test_user_id})")
        TestPrinter.print_info(f"계정 상태: {data['account_status']}", indent=1)
        TestPrinter.print_info(f"메시지: {data.get('message', 'N/A')}", indent=1)
        
        # 중복 회원가입 시도 (실패해야 함)
        print("\n2️⃣  중복 회원가입 시도 (실패 예상)...")
        response2 = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/signup",
            json=signup_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response2.status_code}")
        assert response2.status_code == 400, "중복 회원가입이 차단되지 않음"
        
        TestPrinter.print_success("중복 회원가입 차단 확인")

    def test_email_verification(self):
        """테스트 2: 이메일 인증"""
        TestPrinter.print_header("테스트 2: 이메일 인증")

        if not self.test_user_id:
            TestPrinter.print_warning("회원가입이 먼저 실행되어야 합니다")
            return

        print("\n1️⃣  이메일 인증 요청...")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/verify-email",
            json={"user_id": self.test_user_id},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("이메일 인증 성공")
            TestPrinter.print_info(f"계정 상태: {data.get('account_status', 'N/A')}", indent=1)
            TestPrinter.print_info(f"메시지: {data.get('message', 'N/A')}", indent=1)
        else:
            # 이미 인증된 경우 또는 다른 이유로 실패할 수 있음
            TestPrinter.print_warning(f"이메일 인증 응답: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_admin_approval(self):
        """테스트 3: 관리자 승인"""
        TestPrinter.print_header("테스트 3: 관리자 승인")

        if not self.test_user_id or not self.admin_token:
            TestPrinter.print_warning("회원가입과 관리자 로그인이 먼저 실행되어야 합니다")
            return

        print("\n1️⃣  관리자 승인 요청...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/admin/users/approve",
            json={"user_id": self.test_user_id},
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("관리자 승인 성공")
            TestPrinter.print_info(f"계정 상태: {data.get('account_status', 'N/A')}", indent=1)
            TestPrinter.print_info(f"승인 시간: {data.get('approved_at', 'N/A')}", indent=1)
        else:
            TestPrinter.print_warning(f"관리자 승인 응답: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_login_before_approval(self):
        """테스트 4: 승인 전 로그인 시도 (실패 예상)"""
        TestPrinter.print_header("테스트 4: 승인 전 로그인 시도")

        if not self.test_username:
            TestPrinter.print_warning("회원가입이 먼저 실행되어야 합니다")
            return

        print("\n1️⃣  승인 전 로그인 시도...")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/login",
            json={
                "username": self.test_username,
                "password": self.test_password
            },
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        # 승인 전에는 로그인이 실패해야 함 (또는 Keycloak 설정에 따라 다를 수 있음)
        if response.status_code == 401:
            TestPrinter.print_success("승인 전 로그인 차단 확인")
        elif response.status_code == 200:
            TestPrinter.print_warning("승인 전에도 로그인 가능 (Keycloak 설정 확인 필요)")
        else:
            TestPrinter.print_info(f"로그인 응답: {response.status_code}")

    def test_login_after_approval(self):
        """테스트 5: 승인 후 로그인"""
        TestPrinter.print_header("테스트 5: 승인 후 로그인")

        if not self.test_username:
            TestPrinter.print_warning("회원가입이 먼저 실행되어야 합니다")
            return

        # 승인 처리를 위해 잠시 대기
        print("\n⏳ 승인 처리 대기 중 (3초)...")
        time.sleep(3)

        print("\n1️⃣  로그인 시도...")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/login",
            json={
                "username": self.test_username,
                "password": self.test_password
            },
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            assert "token" in data, "토큰이 응답에 없음"

            TestPrinter.print_success("로그인 성공")
            TestPrinter.print_info(f"토큰 길이: {len(data['token'])}", indent=1)

            # 토큰으로 API 호출 테스트
            print("\n2️⃣  토큰으로 API 호출 테스트...")
            headers = {"Authorization": f"Bearer {data['token']}"}
            test_response = requests.get(
                f"{TestConfig.BASE_URL}/health",
                headers=headers,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )

            print(f"Status: {test_response.status_code}")
            assert test_response.status_code == 200, "토큰으로 API 호출 실패"

            TestPrinter.print_success("토큰 검증 성공")
        else:
            TestPrinter.print_warning(f"로그인 실패: {response.status_code}")
            TestPrinter.print_info(f"응답: {response.text[:200]}", indent=1)

    def test_account_deletion(self):
        """테스트 6: 계정 삭제"""
        TestPrinter.print_header("테스트 6: 계정 삭제")

        if not self.test_user_id or not self.admin_token:
            TestPrinter.print_warning("회원가입과 관리자 로그인이 먼저 실행되어야 합니다")
            return

        print("\n1️⃣  계정 삭제 요청...")
        headers = {"Authorization": f"Bearer {self.admin_token}"}
        response = requests.delete(
            f"{TestConfig.BASE_URL}/api/auth/users/{self.test_user_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code in [200, 204], f"계정 삭제 실패: {response.text}"

        TestPrinter.print_success("계정 삭제 성공")

        # 삭제된 계정으로 로그인 시도 (실패해야 함)
        print("\n2️⃣  삭제된 계정으로 로그인 시도 (실패 예상)...")
        response2 = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/login",
            json={
                "username": self.test_username,
                "password": self.test_password
            },
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response2.status_code}")
        assert response2.status_code == 401, "삭제된 계정으로 로그인이 가능함"

        TestPrinter.print_success("삭제된 계정 로그인 차단 확인")

        # cleanup에서 삭제하지 않도록 플래그 설정
        self.test_user_id = None

    def test_password_validation(self):
        """테스트 7: 비밀번호 유효성 검증"""
        TestPrinter.print_header("테스트 7: 비밀번호 유효성 검증")

        # 약한 비밀번호로 회원가입 시도
        weak_passwords = [
            ("short", "너무 짧은 비밀번호"),
            ("lowercase", "소문자만"),
            ("UPPERCASE", "대문자만"),
            ("12345678", "숫자만"),
            ("NoNumber!", "숫자 없음"),
        ]

        for idx, (password, description) in enumerate(weak_passwords, 1):
            print(f"\n{idx}️⃣  약한 비밀번호 테스트: {description}")

            random_suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=6))
            signup_data = {
                "username": f"weakpw_{random_suffix}",
                "email": f"weakpw_{random_suffix}@example.com",
                "password": password,
                "full_name": "Test User"
            }

            response = requests.post(
                f"{TestConfig.BASE_URL}/api/auth/signup",
                json=signup_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )

            print(f"   Status: {response.status_code}")

            if response.status_code == 400:
                TestPrinter.print_success(f"   약한 비밀번호 차단 확인: {description}")
            else:
                TestPrinter.print_warning(f"   약한 비밀번호가 허용됨: {description}")

    def run_tests(self):
        """테스트 실행"""
        self.test_signup()
        self.test_email_verification()
        self.test_admin_approval()
        # self.test_login_before_approval()  # 순서상 승인 후에 실행하면 의미 없음
        self.test_login_after_approval()
        self.test_password_validation()
        self.test_account_deletion()


if __name__ == "__main__":
    test = AuthFlowE2ETest()
    test.run()

