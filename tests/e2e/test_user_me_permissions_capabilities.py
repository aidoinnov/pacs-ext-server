#!/usr/bin/env python3
"""
GET /api/users/me/permissions, GET /api/users/me/capabilities E2E 시나리오 테스트

요구사항: docs/api/capability/add-job.md
- permission 코드: resource_type.action
- capability 코드: security_capability.name
- project_data.assign, PROJECT_MANAGEMENT: SUPER_ADMIN, PROJECT_ADMIN (프로젝트 관련 통합)
- ROLE_MANAGEMENT: SUPER_ADMIN, ADMIN (접근·역할 메뉴 대체)

필수: 서버 재시작 후 테스트 (신규 API 반영)
"""
import logging
import pytest
from faker import Faker

from utils.api_client import APIClient
from config import TestConfig

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)
fake = Faker()


@pytest.fixture(scope="module")
def config():
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """admin(reader1_user 또는 iaid-pacs-admin) 클라이언트"""
    c = APIClient(config.base_url, config.timeout)
    c.login(config.admin_email, config.admin_password)
    yield c
    c.close()


@pytest.fixture(scope="module")
def super_admin_client(config):
    """SUPER_ADMIN(iaid-pacs-admin) 클라이언트 - 존재 시 사용"""
    c = APIClient(config.base_url, config.timeout)
    try:
        r = c.session.post(
            f"{config.base_url}/api/auth/login",
            json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
            timeout=config.timeout,
        )
        if r.status_code != 200:
            c.close()
            pytest.skip("iaid-pacs-admin 계정 없음")
        data = r.json()
        c.token = data.get("access_token") or data.get("token")
        yield c
    finally:
        c.close()


class TestMePermissionsCapabilitiesBasic:
    """기본 API 응답 검증"""

    def test_get_me_permissions_200(self, admin_client):
        """GET /api/users/me/permissions → 200, permissions 배열"""
        r = admin_client.get("/api/users/me/permissions")
        assert r.status_code == 200, f"Expected 200, got {r.status_code}: {r.text}"
        data = r.json()
        assert "permissions" in data
        assert isinstance(data["permissions"], list)
        logger.info(f"permissions count: {len(data['permissions'])}")

    def test_get_me_capabilities_200(self, admin_client):
        """GET /api/users/me/capabilities → 200, capability_codes 배열"""
        r = admin_client.get("/api/users/me/capabilities")
        assert r.status_code == 200, f"Expected 200, got {r.status_code}: {r.text}"
        data = r.json()
        assert "capability_codes" in data
        assert isinstance(data["capability_codes"], list)
        logger.info(f"capability_codes count: {len(data['capability_codes'])}")

    def test_unauthorized_permissions_401(self, config):
        """인증 없이 /me/permissions → 401"""
        c = APIClient(config.base_url, config.timeout)
        r = c.get("/api/users/me/permissions")
        assert r.status_code == 401, f"Expected 401, got {r.status_code}"
        c.close()

    def test_unauthorized_capabilities_401(self, config):
        """인증 없이 /me/capabilities → 401"""
        c = APIClient(config.base_url, config.timeout)
        r = c.get("/api/users/me/capabilities")
        assert r.status_code == 401, f"Expected 401, got {r.status_code}"
        c.close()

    def test_invalid_token_401(self, config):
        """잘못된 토큰으로 호출 → 401"""
        c = APIClient(config.base_url, config.timeout)
        c.token = "Bearer invalid_token_xyz"
        r = c.get("/api/users/me/permissions")
        assert r.status_code in [401, 403], f"Expected 401/403, got {r.status_code}"
        c.close()


class TestPermissionCodeFormat:
    """permission 코드 형식 검증 (resource_type.action)"""

    def test_permission_format_resource_type_action(self, admin_client):
        """permission 코드는 resource_type.action 형식"""
        r = admin_client.get("/api/users/me/permissions")
        assert r.status_code == 200
        perms = r.json().get("permissions", [])
        for p in perms:
            assert isinstance(p, str), f"permission must be string: {p}"
            # resource_type.action 형식 (최소 한 개의 점)
            assert "." in p, f"permission should be resource_type.action: {p}"
            parts = p.split(".")
            assert len(parts) >= 2, f"permission should have resource_type.action: {p}"

    def test_capability_format_non_empty_strings(self, admin_client):
        """capability 코드는 비어있지 않은 문자열"""
        r = admin_client.get("/api/users/me/capabilities")
        assert r.status_code == 200
        caps = r.json().get("capability_codes", [])
        for c in caps:
            assert isinstance(c, str), f"capability must be string: {c}"
            assert len(c.strip()) > 0, f"capability must be non-empty: {c}"


class TestSettingsPermissionsWhenAdmin:
    """SUPER_ADMIN/PROJECT_ADMIN 시 settings 권한 검증"""

    def test_super_admin_has_role_management(self, super_admin_client):
        """SUPER_ADMIN은 ROLE_MANAGEMENT 포함 (접근·역할 메뉴 권한)"""
        r = super_admin_client.get("/api/users/me/capabilities")
        assert r.status_code == 200
        caps = r.json().get("capability_codes", [])
        assert "ROLE_MANAGEMENT" in caps, f"SUPER_ADMIN should have ROLE_MANAGEMENT: {caps}"

    def test_super_admin_has_project_management(self, super_admin_client):
        """SUPER_ADMIN은 project_data.assign, PROJECT_MANAGEMENT 포함 (프로젝트 통합)"""
        r = super_admin_client.get("/api/users/me/permissions")
        assert r.status_code == 200
        perms = r.json().get("permissions", [])
        assert "project_data.assign" in perms, f"SUPER_ADMIN should have project_data.assign: {perms}"

        r2 = super_admin_client.get("/api/users/me/capabilities")
        assert r2.status_code == 200
        caps = r2.json().get("capability_codes", [])
        assert "PROJECT_MANAGEMENT" in caps, f"SUPER_ADMIN should have PROJECT_MANAGEMENT: {caps}"


class TestProjectAdminScenario:
    """PROJECT_ADMIN 시나리오: 프로젝트 생성 후 멤버로 PROJECT_ADMIN 추가"""

    @pytest.fixture(scope="class")
    def project_admin_client(self, config):
        """프로젝트 생성 → 현재 사용자를 PROJECT_ADMIN으로 추가"""
        c = APIClient(config.base_url, config.timeout)
        c.login(config.admin_email, config.admin_password)

        # 프로젝트 생성
        project_name = f"E2E Perm Cap Test {fake.uuid4()[:8]}"
        r = c.post("/api/projects", json={
            "name": project_name,
            "description": "Permission/Capability E2E",
            "sponsor": "Test",
            "status": "active"
        })
        assert r.status_code in [200, 201], f"Project creation failed: {r.text}"
        project = r.json()

        # 현재 사용자를 PROJECT_ADMIN으로 추가
        me = c.get("/api/users/me").json()
        user_id = me.get("id")
        if user_id:
            mr = c.post(f"/api/projects/{project['id']}/members", json={
                "user_id": user_id,
                "role_id": 196  # PROJECT_ADMIN
            })
            if mr.status_code not in [200, 201]:
                logger.warning(f"Failed to add PROJECT_ADMIN: {mr.text}")

        yield c

        try:
            c.delete(f"/api/projects/{project['id']}")
        except Exception:
            pass
        c.close()

    def test_project_admin_has_project_management(self, project_admin_client):
        """PROJECT_ADMIN 역할 사용자는 project_data.assign, PROJECT_MANAGEMENT 포함"""
        r = project_admin_client.get("/api/users/me/permissions")
        assert r.status_code == 200
        perms = r.json().get("permissions", [])
        assert "project_data.assign" in perms, f"PROJECT_ADMIN should have project_data.assign: {perms}"

        r2 = project_admin_client.get("/api/users/me/capabilities")
        assert r2.status_code == 200
        caps = r2.json().get("capability_codes", [])
        assert "PROJECT_MANAGEMENT" in caps, f"PROJECT_ADMIN should have PROJECT_MANAGEMENT: {caps}"


class TestConsistency:
    """응답 일관성 검증"""

    def test_permissions_capabilities_consistent_per_user(self, admin_client):
        """동일 사용자가 연속 호출 시 동일 응답"""
        r1 = admin_client.get("/api/users/me/permissions")
        r2 = admin_client.get("/api/users/me/permissions")
        assert r1.status_code == 200 and r2.status_code == 200
        p1 = set(r1.json().get("permissions", []))
        p2 = set(r2.json().get("permissions", []))
        assert p1 == p2, f"permissions inconsistent: {p1 ^ p2}"

        r3 = admin_client.get("/api/users/me/capabilities")
        r4 = admin_client.get("/api/users/me/capabilities")
        assert r3.status_code == 200 and r4.status_code == 200
        c3 = set(r3.json().get("capability_codes", []))
        c4 = set(r4.json().get("capability_codes", []))
        assert c3 == c4, f"capabilities inconsistent: {c3 ^ c4}"

    def test_me_and_permissions_capabilities_same_user(self, admin_client):
        """GET /me와 permissions/capabilities는 동일 사용자 기준"""
        me = admin_client.get("/api/users/me")
        perms = admin_client.get("/api/users/me/permissions")
        caps = admin_client.get("/api/users/me/capabilities")
        assert me.status_code == 200 and perms.status_code == 200 and caps.status_code == 200
        # 응답 구조만 검증 (같은 토큰이므로 동일 사용자)
        assert "id" in me.json() or "username" in me.json()
        assert "permissions" in perms.json()
        assert "capability_codes" in caps.json()


class TestResponseSchema:
    """응답 스키마 검증"""

    def test_permissions_response_schema(self, admin_client):
        """permissions 응답: { permissions: string[] }"""
        r = admin_client.get("/api/users/me/permissions")
        assert r.status_code == 200
        data = r.json()
        assert set(data.keys()) == {"permissions"}
        for item in data["permissions"]:
            assert isinstance(item, str)

    def test_capabilities_response_schema(self, admin_client):
        """capabilities 응답: { capability_codes: string[] }"""
        r = admin_client.get("/api/users/me/capabilities")
        assert r.status_code == 200
        data = r.json()
        assert set(data.keys()) == {"capability_codes"}
        for item in data["capability_codes"]:
            assert isinstance(item, str)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
