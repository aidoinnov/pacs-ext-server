#!/usr/bin/env python3
"""
인증 플로우 E2E 테스트 (회원가입 → 관리자 승인 → 로그인 → 계정 삭제)

이메일 인증 비활성화: POST /api/auth/verify-email API는 제거됨
"""
import logging
import random
import string
import time

import pytest
import requests

from config import TestConfig
from utils.api_client import APIClient

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 API 클라이언트 (로그인됨)"""
    c = APIClient(config.base_url, config.timeout)
    c.login(config.admin_email, config.admin_password)
    yield c
    c.close()


@pytest.fixture
def random_credentials():
    """테스트용 랜덤 계정 정보"""
    suffix = "".join(random.choices(string.ascii_lowercase + string.digits, k=8))
    return {
        "username": f"e2e_test_{suffix}",
        "email": f"e2e_{suffix}@example.com",
        "password": "TestPassword123!",
        "full_name": "E2E Test User",
    }


class TestAuthSignupFlow:
    """회원가입 → 승인 → 로그인 플로우 테스트"""

    def test_01_signup_success(self, config, random_credentials):
        """회원가입 성공"""
        logger.info("TEST: 회원가입")
        signup_data = {
            **random_credentials,
            "organization": "Test Org",
            "department": "Test Dept",
            "phone": "010-1234-5678",
        }

        response = requests.post(
            f"{config.base_url}/api/auth/signup",
            json=signup_data,
            timeout=config.timeout,
        )

        assert response.status_code == 201, f"회원가입 실패: {response.text}"
        data = response.json()

        assert "user_id" in data
        assert data["username"] == signup_data["username"]
        assert data["email"] == signup_data["email"]
        assert "account_status" in data

        # PENDING_APPROVAL 또는 PendingApproval (Debug format)
        status = data["account_status"]
        assert "PENDING" in status.upper() or "APPROVAL" in status.upper(), (
            f"예상: PENDING_APPROVAL, 실제: {status}"
        )

        logger.info(f"✓ 회원가입 성공 user_id={data['user_id']}")

    def test_02_signup_duplicate_rejected(self, config, random_credentials):
        """중복 회원가입 차단"""
        logger.info("TEST: 중복 회원가입 차단")
        signup_data = {
            **random_credentials,
            "full_name": "Duplicate Test",
        }

        # 1차 가입
        r1 = requests.post(
            f"{config.base_url}/api/auth/signup",
            json=signup_data,
            timeout=config.timeout,
        )
        assert r1.status_code == 201

        # 2차 중복 가입
        r2 = requests.post(
            f"{config.base_url}/api/auth/signup",
            json=signup_data,
            timeout=config.timeout,
        )
        assert r2.status_code in [400, 409], f"중복 가입이 차단되지 않음: {r2.status_code}"

        logger.info("✓ 중복 회원가입 차단 확인")

    def test_03_full_flow_signup_approve_login_delete(self, config, admin_client):
        """전체 플로우: 회원가입 → 관리자 승인 → 로그인 → 계정 삭제"""
        logger.info("TEST: Signup → Approve → Login → Delete")
        suffix = "".join(random.choices(string.ascii_lowercase + string.digits, k=8))
        username = f"flow_test_{suffix}"
        email = f"flow_{suffix}@example.com"
        password = "FlowPassword123!"

        # 1. 회원가입
        signup_resp = requests.post(
            f"{config.base_url}/api/auth/signup",
            json={
                "username": username,
                "email": email,
                "password": password,
                "full_name": "Flow Test User",
            },
            timeout=config.timeout,
        )
        assert signup_resp.status_code == 201, signup_resp.text
        user_id = signup_resp.json()["user_id"]
        logger.info(f"  1) 회원가입 OK user_id={user_id}")

        # 2. 관리자 승인
        approve_resp = admin_client.post(
            "/api/auth/admin/users/approve",
            json={"user_id": user_id},
        )
        assert approve_resp.status_code == 200, approve_resp.text
        logger.info("  2) 관리자 승인 OK")

        time.sleep(1)

        # 3. 로그인
        login_resp = requests.post(
            f"{config.base_url}/api/auth/login",
            json={"username": username, "password": password},
            timeout=config.timeout,
        )
        assert login_resp.status_code == 200, login_resp.text
        token = login_resp.json().get("token") or login_resp.json().get("access_token")
        assert token, "토큰 없음"
        logger.info("  3) 로그인 OK")

        # 4. 계정 삭제 (admin token 사용)
        delete_resp = admin_client.delete(f"/api/users/{user_id}")
        assert delete_resp.status_code in [200, 204], delete_resp.text
        logger.info("  4) 계정 삭제 OK")

        # 5. 삭제 후 로그인 실패
        login_again = requests.post(
            f"{config.base_url}/api/auth/login",
            json={"username": username, "password": password},
            timeout=config.timeout,
        )
        assert login_again.status_code == 401, "삭제된 계정으로 로그인 가능"
        logger.info("✓ 전체 플로우 통과")

    def test_04_weak_password_rejected(self, config, random_credentials):
        """약한 비밀번호 회원가입 차단"""
        logger.info("TEST: 약한 비밀번호 차단")
        weak = {
            **random_credentials,
            "username": f"weak_{random_credentials['username']}",
            "email": f"weak_{random_credentials['email']}",
            "password": "short",
        }

        resp = requests.post(
            f"{config.base_url}/api/auth/signup",
            json=weak,
            timeout=config.timeout,
        )
        assert resp.status_code == 400, f"약한 비밀번호가 허용됨: {resp.status_code}"
        logger.info("✓ 약한 비밀번호 차단 확인")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
