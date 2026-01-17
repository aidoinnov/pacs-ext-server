"""
프로젝트 및 권한 관리 E2E 테스트
"""
import pytest
import logging
from faker import Faker
from utils.api_client import APIClient
from config import TestConfig

logger = logging.getLogger(__name__)
fake = Faker()


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 클라이언트"""
    client = APIClient(config.base_url, config.timeout)
    client.login(config.admin_email, config.admin_password)
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_project(admin_client):
    """테스트용 프로젝트 생성"""
    project_name = f"E2E Test Project {fake.uuid4()[:8]}"
    
    response = admin_client.post("/api/projects", json={
        "name": project_name,
        "description": "E2E 테스트용 프로젝트",
        "status": "active"
    })
    
    assert response.status_code in [200, 201], f"Failed to create project: {response.text}"
    project = response.json()
    logger.info(f"Created test project: {project['name']} (ID: {project['id']})")
    
    yield project
    
    # 테스트 후 프로젝트 삭제
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
        logger.info(f"Deleted test project: {project['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete test project: {e}")


class TestProjectManagement:
    """프로젝트 관리 테스트"""
    
    def test_01_create_project(self, admin_client):
        """프로젝트 생성 테스트"""
        logger.info("Testing project creation...")
        
        project_name = f"Test Project {fake.uuid4()[:8]}"
        response = admin_client.post("/api/projects", json={
            "name": project_name,
            "description": "테스트 프로젝트",
            "status": "active"
        })
        
        assert response.status_code in [200, 201], f"Failed to create project: {response.text}"
        data = response.json()
        
        assert "id" in data
        assert data["name"] == project_name
        assert data["status"] == "active"
        
        # 생성된 프로젝트 삭제
        project_id = data["id"]
        admin_client.delete(f"/api/projects/{project_id}")
        
        logger.info(f"✓ Project created successfully: {project_name}")
    
    def test_02_list_projects(self, admin_client, test_project):
        """프로젝트 목록 조회 테스트"""
        logger.info("Testing project list...")
        
        response = admin_client.get("/api/projects")
        
        assert response.status_code == 200, f"Failed to list projects: {response.text}"
        data = response.json()
        
        assert isinstance(data, list) or "projects" in data
        projects = data if isinstance(data, list) else data["projects"]
        
        # 테스트 프로젝트가 목록에 있는지 확인
        project_ids = [p["id"] for p in projects]
        assert test_project["id"] in project_ids, "Test project not found in list"
        
        logger.info(f"✓ Found {len(projects)} projects")
    
    def test_03_get_project_detail(self, admin_client, test_project):
        """프로젝트 상세 조회 테스트"""
        logger.info("Testing project detail...")
        
        response = admin_client.get(f"/api/projects/{test_project['id']}")
        
        assert response.status_code == 200, f"Failed to get project: {response.text}"
        data = response.json()
        
        assert data["id"] == test_project["id"]
        assert data["name"] == test_project["name"]
        
        logger.info(f"✓ Got project detail: {data['name']}")
    
    def test_04_update_project(self, admin_client, test_project):
        """프로젝트 수정 테스트"""
        logger.info("Testing project update...")
        
        new_description = f"Updated description {fake.uuid4()[:8]}"
        response = admin_client.put(f"/api/projects/{test_project['id']}", json={
            "description": new_description
        })
        
        assert response.status_code == 200, f"Failed to update project: {response.text}"
        data = response.json()
        
        assert data["description"] == new_description
        
        logger.info(f"✓ Project updated successfully")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

