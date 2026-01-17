"""
성능 테스트 - 동시 요청 처리
"""
import pytest
import logging
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from utils.api_client import APIClient
from utils.performance_metrics import MetricsCollector
from config import TestConfig, PerformanceConfig, TEST_STUDY_UID

logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def perf_config():
    """성능 테스트 설정"""
    return PerformanceConfig.from_env()


@pytest.fixture(scope="module")
def admin_token(config):
    """관리자 토큰"""
    client = APIClient(config.base_url, config.timeout)
    data = client.login(config.admin_email, config.admin_password)
    token = data.get("access_token") or data.get("token")
    client.close()
    return token


def make_request(base_url: str, token: str, endpoint: str, method: str = "GET") -> tuple:
    """단일 요청 실행"""
    import requests
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    start_time = time.time()
    try:
        if method == "GET":
            response = requests.get(f"{base_url}{endpoint}", headers=headers, timeout=30)
        else:
            response = requests.post(f"{base_url}{endpoint}", headers=headers, timeout=30)
        
        elapsed = time.time() - start_time
        return elapsed, response.status_code, None
    except Exception as e:
        elapsed = time.time() - start_time
        return elapsed, 0, str(e)


class TestConcurrentRequests:
    """동시 요청 처리 성능 테스트"""
    
    def test_01_concurrent_login(self, config, perf_config):
        """동시 로그인 요청 테스트"""
        logger.info(f"Testing {perf_config.concurrent_users} concurrent login requests...")
        
        metrics = MetricsCollector()
        
        def login_request():
            client = APIClient(config.base_url, config.timeout)
            start_time = time.time()
            try:
                client.login(config.admin_email, config.admin_password)
                elapsed = time.time() - start_time
                metrics.record_request("concurrent_login", elapsed, 200)
            except Exception as e:
                elapsed = time.time() - start_time
                metrics.record_request("concurrent_login", elapsed, 0, str(e))
            finally:
                client.close()
        
        # 동시 실행
        with ThreadPoolExecutor(max_workers=perf_config.concurrent_users) as executor:
            futures = [executor.submit(login_request) for _ in range(perf_config.concurrent_users)]
            for future in as_completed(futures):
                future.result()
        
        metrics.print_summary()
        
        stats = metrics.get_or_create("concurrent_login").get_stats()
        assert stats["error_rate"] < 10, f"Error rate too high: {stats['error_rate']}%"
        
        logger.info(f"✓ Concurrent login test completed")
    
    def test_02_concurrent_annotation_queries(self, config, perf_config, admin_token):
        """동시 어노테이션 조회 요청 테스트"""
        logger.info(f"Testing {perf_config.concurrent_users} concurrent annotation queries...")
        
        metrics = MetricsCollector()
        
        def query_annotations():
            elapsed, status_code, error = make_request(
                config.base_url,
                admin_token,
                f"/api/annotations?study_instance_uid={TEST_STUDY_UID}",
                "GET"
            )
            metrics.record_request("concurrent_annotation_query", elapsed, status_code, error)
        
        # 동시 실행
        with ThreadPoolExecutor(max_workers=perf_config.concurrent_users) as executor:
            futures = [executor.submit(query_annotations) for _ in range(perf_config.requests_per_user)]
            for future in as_completed(futures):
                future.result()
        
        metrics.print_summary()
        
        stats = metrics.get_or_create("concurrent_annotation_query").get_stats()
        assert stats["error_rate"] < 10, f"Error rate too high: {stats['error_rate']}%"
        assert stats["p95_time"] < 1.0, f"P95 response time too high: {stats['p95_time']*1000:.2f}ms"
        
        logger.info(f"✓ Concurrent annotation query test completed")
    
    def test_03_concurrent_project_queries(self, config, perf_config, admin_token):
        """동시 프로젝트 조회 요청 테스트"""
        logger.info(f"Testing {perf_config.concurrent_users} concurrent project queries...")
        
        metrics = MetricsCollector()
        
        def query_projects():
            elapsed, status_code, error = make_request(
                config.base_url,
                admin_token,
                "/api/projects",
                "GET"
            )
            metrics.record_request("concurrent_project_query", elapsed, status_code, error)
        
        # 동시 실행
        with ThreadPoolExecutor(max_workers=perf_config.concurrent_users) as executor:
            futures = [executor.submit(query_projects) for _ in range(perf_config.requests_per_user)]
            for future in as_completed(futures):
                future.result()
        
        metrics.print_summary()
        
        stats = metrics.get_or_create("concurrent_project_query").get_stats()
        assert stats["error_rate"] < 10, f"Error rate too high: {stats['error_rate']}%"
        
        logger.info(f"✓ Concurrent project query test completed")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

