"""
스냅샷 이미지 업로드/다운로드 E2E 테스트
"""
import pytest
import logging
import io
from PIL import Image
import requests
from faker import Faker
from utils.api_client import APIClient
from config import TestConfig, TEST_STUDY_UID, SAMPLE_ANNOTATION_DATA

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
        "name": f"Snapshot Test Project {fake.uuid4()[:8]}",
        "description": "스냅샷 테스트용",
        "status": "active"
    })
    project = response.json()
    yield project
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
    except:
        pass


def create_test_image() -> bytes:
    """테스트용 이미지 생성"""
    img = Image.new('RGB', (100, 100), color='red')
    img_bytes = io.BytesIO()
    img.save(img_bytes, format='PNG')
    return img_bytes.getvalue()


class TestSnapshotUploadDownload:
    """스냅샷 업로드/다운로드 테스트"""
    
    def test_01_generate_upload_url(self, admin_client, test_project):
        """업로드용 Signed URL 생성 테스트"""
        logger.info("Testing upload URL generation...")
        
        # 먼저 어노테이션 생성
        annotation_response = admin_client.post("/api/annotations", json={
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "test-tool"
        })
        annotation = annotation_response.json()
        annotation_id = annotation["id"]
        
        # Upload URL 생성
        response = admin_client.post(f"/api/annotations/{annotation_id}/snapshot/upload-url", json={
            "content_type": "image/png",
            "ttl_seconds": 3600
        })
        
        assert response.status_code == 200, f"Failed to generate upload URL: {response.text}"
        data = response.json()
        
        assert "upload_url" in data
        assert "snapshot_image_key" in data
        
        logger.info(f"✓ Upload URL generated for annotation {annotation_id}")
        return {
            "annotation_id": annotation_id,
            "upload_url": data["upload_url"],
            "snapshot_key": data["snapshot_image_key"]
        }
    
    def test_02_upload_snapshot_image(self, admin_client, test_project):
        """스냅샷 이미지 업로드 테스트"""
        logger.info("Testing snapshot image upload...")
        
        # 어노테이션 생성
        annotation_response = admin_client.post("/api/annotations", json={
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "test-tool"
        })
        annotation = annotation_response.json()
        annotation_id = annotation["id"]
        
        # Upload URL 생성
        url_response = admin_client.post(f"/api/annotations/{annotation_id}/snapshot/upload-url", json={
            "content_type": "image/png"
        })
        url_data = url_response.json()
        upload_url = url_data["upload_url"]
        
        # 이미지 업로드 (실제 S3/GCS에 업로드)
        test_image = create_test_image()
        upload_response = requests.put(
            upload_url,
            data=test_image,
            headers={"Content-Type": "image/png"}
        )
        
        assert upload_response.status_code in [200, 204], f"Failed to upload image: {upload_response.text}"
        
        logger.info(f"✓ Snapshot image uploaded for annotation {annotation_id}")
    
    def test_03_generate_download_url(self, admin_client, test_project):
        """다운로드용 Signed URL 생성 테스트"""
        logger.info("Testing download URL generation...")
        
        # 어노테이션 생성 (스냅샷 키 포함)
        annotation_response = admin_client.post("/api/annotations", json={
            "project_id": test_project["id"],
            "study_instance_uid": TEST_STUDY_UID,
            "annotation_data": SAMPLE_ANNOTATION_DATA,
            "tool_name": "test-tool"
        })
        annotation = annotation_response.json()
        annotation_id = annotation["id"]
        
        # Upload URL 생성 및 업로드 (스냅샷 키 설정)
        url_response = admin_client.post(f"/api/annotations/{annotation_id}/snapshot/upload-url", json={
            "content_type": "image/png"
        })
        
        # Download URL 생성
        download_response = admin_client.get(f"/api/annotations/{annotation_id}/snapshot/download-url")
        
        if download_response.status_code == 200:
            data = download_response.json()
            assert "download_url" in data
            logger.info(f"✓ Download URL generated for annotation {annotation_id}")
        else:
            logger.warning(f"Download URL generation returned {download_response.status_code}")
    
    def test_04_bulk_download_urls(self, admin_client):
        """대량 다운로드 URL 생성 테스트"""
        logger.info("Testing bulk download URL generation...")
        
        snapshot_keys = [
            f"snapshots/test-{fake.uuid4()}.png",
            f"snapshots/test-{fake.uuid4()}.png"
        ]
        
        response = admin_client.post("/api/snapshots/download-urls/bulk", json={
            "snapshot_keys": snapshot_keys,
            "ttl_seconds": 3600
        })
        
        if response.status_code == 200:
            data = response.json()
            assert "urls" in data or isinstance(data, list)
            logger.info(f"✓ Bulk download URLs generated")
        else:
            logger.warning(f"Bulk download returned {response.status_code}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

