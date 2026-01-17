"""
인증 및 사용자 관리 E2E 테스트
"""
import pytest
import logging
from utils.api_client import APIClient
from config import TestConfig

logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def client(config):
    """API 클라이언트"""
    client = APIClient(config.base_url, config.timeout)
    yield client
    client.close()


class TestAuthentication:
    """인증 테스트"""
    
    def test_01_login_success(self, client, config):
        """로그인 성공 테스트"""
        logger.info("Testing successful login...")

        response = client.post("/api/auth/login", json={
            "username": config.admin_email,
            "password": config.admin_password
        })

        assert response.status_code == 200, f"Login failed: {response.text}"
        data = response.json()

        assert "access_token" in data or "token" in data, "No access token in response"

        # 토큰 저장
        client.token = data.get("access_token") or data.get("token")
        logger.info(f"✓ Login successful for {config.admin_email}")
    
    def test_02_login_invalid_credentials(self, client):
        """잘못된 인증 정보로 로그인 실패 테스트"""
        logger.info("Testing login with invalid credentials...")

        response = client.post("/api/auth/login", json={
            "username": "invalid_user",
            "password": "wrongpassword"
        })

        assert response.status_code in [400, 401, 403], "Should fail with invalid credentials"
        logger.info("✓ Login correctly rejected invalid credentials")
    
    def test_03_get_current_user(self, client, config):
        """현재 사용자 정보 조회 테스트"""
        logger.info("Testing get current user...")

        # 먼저 로그인
        client.login(config.admin_email, config.admin_password)

        response = client.get("/api/users/me")

        assert response.status_code == 200, f"Get user failed: {response.text}"
        data = response.json()

        assert "id" in data or "user_id" in data
        assert "email" in data or "username" in data

        email = data.get("email") or data.get("username", "N/A")
        logger.info(f"✓ Got current user info: {email}")
    
    def test_04_unauthorized_access(self, client):
        """인증 없이 접근 시 실패 테스트"""
        logger.info("Testing unauthorized access...")
        
        # 토큰 제거
        original_token = client.token
        client.token = None
        
        response = client.get("/api/users/me")
        
        assert response.status_code == 401, "Should fail without authentication"
        
        # 토큰 복원
        client.token = original_token
        logger.info("✓ Unauthorized access correctly rejected")
    
    def test_05_invalid_token(self, client):
        """잘못된 토큰으로 접근 시 실패 테스트"""
        logger.info("Testing invalid token...")
        
        # 잘못된 토큰 설정
        original_token = client.token
        client.token = "invalid.token.here"
        
        response = client.get("/api/users/me")
        
        assert response.status_code == 401, "Should fail with invalid token"
        
        # 토큰 복원
        client.token = original_token
        logger.info("✓ Invalid token correctly rejected")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

