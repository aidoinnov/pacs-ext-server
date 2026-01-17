"""
성능 테스트 - 대량 데이터 조회
"""
import pytest
import logging
import time
from faker import Faker
from utils.api_client import APIClient
from utils.performance_metrics import MetricsCollector
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
def test_project_with_annotations(admin_client):
    """대량의 어노테이션이 있는 테스트 프로젝트"""
    # 프로젝트 생성
    project_response = admin_client.post("/api/projects", json={
        "name": f"Bulk Test Project {fake.uuid4()[:8]}",
        "description": "대량 데이터 테스트용",
        "status": "active"
    })
    project = project_response.json()
    project_id = project["id"]
    
    logger.info(f"Creating test annotations for project {project_id}...")
    
    # 100개의 어노테이션 생성
    annotation_count = 100
    created_annotations = []
    
    for i in range(annotation_count):
        try:
            response = admin_client.post("/api/annotations", json={
                "project_id": project_id,
                "study_instance_uid": TEST_STUDY_UID,
                "series_instance_uid": f"1.2.3.4.5.{i}",
                "annotation_data": SAMPLE_ANNOTATION_DATA,
                "tool_name": f"tool-{i % 5}",
                "label": f"label-{i % 10}",
                "description": f"Test annotation {i}"
            })
            if response.status_code in [200, 201]:
                created_annotations.append(response.json())
        except Exception as e:
            logger.warning(f"Failed to create annotation {i}: {e}")
    
    logger.info(f"Created {len(created_annotations)} test annotations")
    
    yield {
        "project": project,
        "annotations": created_annotations
    }
    
    # 정리
    try:
        for ann in created_annotations:
            try:
                admin_client.delete(f"/api/annotations/{ann['id']}")
            except:
                pass
        admin_client.delete(f"/api/projects/{project_id}")
    except:
        pass


class TestBulkDataQuery:
    """대량 데이터 조회 성능 테스트"""
    
    def test_01_query_all_annotations_by_project(self, admin_client, test_project_with_annotations):
        """프로젝트의 모든 어노테이션 조회 성능"""
        logger.info("Testing bulk annotation query by project...")
        
        project_id = test_project_with_annotations["project"]["id"]
        metrics = MetricsCollector()
        
        # 10회 반복 측정
        for i in range(10):
            start_time = time.time()
            try:
                response = admin_client.get("/api/annotations", params={
                    "project_id": project_id
                })
                elapsed = time.time() - start_time
                
                if response.status_code == 200:
                    data = response.json()
                    annotations = data if isinstance(data, list) else data.get("annotations", [])
                    logger.info(f"  Query {i+1}: {len(annotations)} annotations in {elapsed*1000:.2f}ms")
                    metrics.record_request("bulk_query_by_project", elapsed, 200)
                else:
                    metrics.record_request("bulk_query_by_project", elapsed, response.status_code, response.text)
            except Exception as e:
                elapsed = time.time() - start_time
                metrics.record_request("bulk_query_by_project", elapsed, 0, str(e))
        
        metrics.print_summary()
        
        stats = metrics.get_or_create("bulk_query_by_project").get_stats()
        assert stats["error_rate"] == 0, "Should have no errors"
        assert stats["avg_time"] < 2.0, f"Average query time too high: {stats['avg_time']*1000:.2f}ms"
        
        logger.info(f"✓ Bulk query by project completed")
    
    def test_02_query_annotations_by_study(self, admin_client, test_project_with_annotations):
        """Study UID로 어노테이션 조회 성능"""
        logger.info("Testing bulk annotation query by study...")
        
        metrics = MetricsCollector()
        
        # 10회 반복 측정
        for i in range(10):
            start_time = time.time()
            try:
                response = admin_client.get("/api/annotations", params={
                    "study_instance_uid": TEST_STUDY_UID
                })
                elapsed = time.time() - start_time
                
                if response.status_code == 200:
                    data = response.json()
                    annotations = data if isinstance(data, list) else data.get("annotations", [])
                    logger.info(f"  Query {i+1}: {len(annotations)} annotations in {elapsed*1000:.2f}ms")
                    metrics.record_request("bulk_query_by_study", elapsed, 200)
                else:
                    metrics.record_request("bulk_query_by_study", elapsed, response.status_code, response.text)
            except Exception as e:
                elapsed = time.time() - start_time
                metrics.record_request("bulk_query_by_study", elapsed, 0, str(e))
        
        metrics.print_summary()
        
        stats = metrics.get_or_create("bulk_query_by_study").get_stats()
        assert stats["error_rate"] == 0, "Should have no errors"
        
        logger.info(f"✓ Bulk query by study completed")
    
    def test_03_pagination_performance(self, admin_client, test_project_with_annotations):
        """페이지네이션 성능 테스트"""
        logger.info("Testing pagination performance...")
        
        project_id = test_project_with_annotations["project"]["id"]
        metrics = MetricsCollector()
        
        page_size = 20
        page = 1
        
        while True:
            start_time = time.time()
            try:
                response = admin_client.get("/api/annotations", params={
                    "project_id": project_id,
                    "page": page,
                    "page_size": page_size
                })
                elapsed = time.time() - start_time
                
                if response.status_code == 200:
                    data = response.json()
                    annotations = data if isinstance(data, list) else data.get("annotations", [])
                    logger.info(f"  Page {page}: {len(annotations)} annotations in {elapsed*1000:.2f}ms")
                    metrics.record_request("pagination_query", elapsed, 200)
                    
                    if len(annotations) < page_size:
                        break
                    page += 1
                else:
                    break
            except Exception as e:
                elapsed = time.time() - start_time
                metrics.record_request("pagination_query", elapsed, 0, str(e))
                break
        
        metrics.print_summary()
        logger.info(f"✓ Pagination test completed ({page} pages)")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

