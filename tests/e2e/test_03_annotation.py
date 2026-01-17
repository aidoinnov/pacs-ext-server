"""
어노테이션 CRUD E2E 테스트
"""
import pytest
import logging
from faker import Faker
from utils.api_client import APIClient
from config import TestConfig, TEST_STUDY_UID, TEST_SERIES_UID, TEST_INSTANCE_UID, SAMPLE_ANNOTATION_DATA

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
    """테스트용 프로젝트"""
    response = admin_client.post("/api/projects", json={
        "name": f"Annotation Test Project {fake.uuid4()[:8]}",
        "description": "어노테이션 테스트용",
        "status": "active"
    })
    project = response.json()
    yield project
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
    except:
        pass


class TestAnnotationCRUD:
    """어노테이션 CRUD 테스트"""
    
    def test_01_create_annotation(self, admin_client, test_project):
        """어노테이션 생성 테스트"""
        logger.info("Testing annotation creation...")
        
        annotation_data = {
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "series_instance_uid": TEST_SERIES_UID,
            "sop_instance_uid": TEST_INSTANCE_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "rectangle",
            "label": "test-label",
            "description": "E2E 테스트 어노테이션"
        }
        
        response = admin_client.post("/api/annotations", json=annotation_data)
        
        assert response.status_code in [200, 201], f"Failed to create annotation: {response.text}"
        data = response.json()
        
        assert "id" in data
        assert data["study_instance_uid"] == TEST_STUDY_UID
        assert data["tool_name"] == "rectangle"
        
        logger.info(f"✓ Annotation created: ID={data['id']}")
        return data
    
    def test_02_get_annotations_by_study(self, admin_client):
        """Study UID로 어노테이션 조회 테스트"""
        logger.info("Testing get annotations by study...")
        
        response = admin_client.get("/api/annotations", params={
            "study_instance_uid": TEST_STUDY_UID
        })
        
        assert response.status_code == 200, f"Failed to get annotations: {response.text}"
        data = response.json()
        
        assert "annotations" in data or isinstance(data, list)
        annotations = data if isinstance(data, list) else data["annotations"]
        
        logger.info(f"✓ Found {len(annotations)} annotations for study")
    
    def test_03_get_annotations_by_project(self, admin_client, test_project):
        """프로젝트로 어노테이션 조회 테스트"""
        logger.info("Testing get annotations by project...")
        
        response = admin_client.get("/api/annotations", params={
            "project_id": test_project["id"]
        })
        
        assert response.status_code == 200, f"Failed to get annotations: {response.text}"
        data = response.json()
        
        assert "annotations" in data or isinstance(data, list)
        
        logger.info(f"✓ Got annotations for project {test_project['id']}")
    
    def test_04_update_annotation(self, admin_client, test_project):
        """어노테이션 수정 테스트"""
        logger.info("Testing annotation update...")
        
        # 먼저 어노테이션 생성
        create_response = admin_client.post("/api/annotations", json={
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "test-tool"
        })
        annotation = create_response.json()
        annotation_id = annotation["id"]
        
        # 수정
        new_label = f"updated-label-{fake.uuid4()[:8]}"
        update_response = admin_client.put(f"/api/annotations/{annotation_id}", json={
            "label": new_label,
            "description": "Updated description"
        })
        
        assert update_response.status_code == 200, f"Failed to update: {update_response.text}"
        updated = update_response.json()
        
        assert updated["label"] == new_label
        
        logger.info(f"✓ Annotation updated: ID={annotation_id}")
    
    def test_05_delete_annotation(self, admin_client, test_project):
        """어노테이션 삭제 테스트"""
        logger.info("Testing annotation deletion...")
        
        # 먼저 어노테이션 생성
        create_response = admin_client.post("/api/annotations", json={
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "test-tool"
        })
        annotation = create_response.json()
        annotation_id = annotation["id"]
        
        # 삭제
        delete_response = admin_client.delete(f"/api/annotations/{annotation_id}")
        
        assert delete_response.status_code in [200, 204], f"Failed to delete: {delete_response.text}"
        
        # 삭제 확인
        get_response = admin_client.get(f"/api/annotations/{annotation_id}")
        assert get_response.status_code == 404, "Annotation should be deleted"
        
        logger.info(f"✓ Annotation deleted: ID={annotation_id}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

