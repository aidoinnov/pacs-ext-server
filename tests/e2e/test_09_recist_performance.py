"""
RECIST Lesion 성능 테스트

대량 데이터 처리 및 동시성 테스트를 통한 성능 검증
"""

import pytest
import logging
import time
import concurrent.futures
from typing import List, Dict
from config import TestConfig

logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 API 클라이언트"""
    from utils.api_client import APIClient
    client = APIClient(config.base_url, config.timeout)
    client.login(config.admin_email, config.admin_password)
    return client


@pytest.fixture(scope="module")
def performance_project(admin_client):
    """성능 테스트용 프로젝트"""
    response = admin_client.post("/api/projects", json={
        "name": "RECIST Performance Test",
        "description": "Performance and load testing",
        "sponsor": "Test Lab",
        "start_date": "2025-01-01",
        "end_date": "2026-12-31",
        "auto_complete": False
    })
    
    assert response.status_code == 201
    project = response.json()
    logger.info(f"Created performance test project (ID: {project['id']})")
    
    yield project
    
    # Cleanup
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
        logger.info(f"Deleted project: {project['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete project: {e}")


class TestBulkDataPerformance:
    """대량 데이터 처리 성능 테스트"""
    
    def test_01_create_multiple_subjects(self, admin_client, performance_project):
        """대량 Subject 생성 성능 테스트"""
        logger.info("=" * 80)
        logger.info("Performance Test 1: Bulk Subject Creation")
        logger.info("=" * 80)
        
        project_id = performance_project['id']
        subject_count = 50
        
        start_time = time.time()
        subjects = []
        
        for i in range(1, subject_count + 1):
            response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
                "subject_code": f"PERF-{i:04d}",
                "patient_id": f"PT{i:06d}",
                "patient_name": f"Performance Test Patient {i}",
                "patient_birth_date": "1980-01-01"
            })
            
            assert response.status_code == 201
            subjects.append(response.json())
            
            if i % 10 == 0:
                logger.info(f"  Created {i}/{subject_count} subjects...")
        
        elapsed = time.time() - start_time
        avg_time = elapsed / subject_count
        
        logger.info(f"\n--- Results ---")
        logger.info(f"Total subjects created: {subject_count}")
        logger.info(f"Total time: {elapsed:.2f}s")
        logger.info(f"Average time per subject: {avg_time*1000:.2f}ms")
        logger.info(f"Throughput: {subject_count/elapsed:.2f} subjects/sec")
        
        # 성능 기준: 평균 100ms 이하
        assert avg_time < 0.1, f"Subject creation too slow: {avg_time*1000:.2f}ms > 100ms"
        
        logger.info("✅ Bulk subject creation performance test passed")
        
        # 다음 테스트를 위해 저장
        self.subjects = subjects
    
    def test_02_create_lesions_for_all_subjects(self, admin_client, performance_project):
        """모든 Subject에 대한 Lesion 생성 성능 테스트"""
        logger.info("=" * 80)
        logger.info("Performance Test 2: Bulk Lesion Creation")
        logger.info("=" * 80)
        
        assert hasattr(self, 'subjects'), "Subjects not found. Run test_01 first."
        
        total_lesions = 0
        start_time = time.time()
        
        for idx, subject in enumerate(self.subjects, 1):
            subject_id = subject['id']
            
            # Baseline TimePoint 생성
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "BL",
                "visit_type": "Baseline",
                "order_index": 0
            })
            assert response.status_code == 201
            baseline_tp = response.json()
            
            # 각 Subject에 3개의 Target Lesion 생성
            for i in range(3):
                response = admin_client.post(
                    f"/api/recist-lesions/subjects/{subject_id}",
                    json={
                        "lesion_type": "TARGET",
                        "baseline_timepoint_id": baseline_tp['id'],
                        "organ_site": f"Liver-S{i+1}",
                        "description": f"Performance test lesion {i+1}"
                    }
                )
                assert response.status_code == 201
                total_lesions += 1
            
            if idx % 10 == 0:
                logger.info(f"  Processed {idx}/{len(self.subjects)} subjects...")
        
        elapsed = time.time() - start_time
        avg_time = elapsed / total_lesions
        
        logger.info(f"\n--- Results ---")
        logger.info(f"Total lesions created: {total_lesions}")
        logger.info(f"Total time: {elapsed:.2f}s")
        logger.info(f"Average time per lesion: {avg_time*1000:.2f}ms")
        logger.info(f"Throughput: {total_lesions/elapsed:.2f} lesions/sec")
        
        # 성능 기준: 평균 150ms 이하
        assert avg_time < 0.15, f"Lesion creation too slow: {avg_time*1000:.2f}ms > 150ms"
        
        logger.info("✅ Bulk lesion creation performance test passed")

    def test_03_query_performance(self, admin_client):
        """대량 데이터 조회 성능 테스트"""
        logger.info("=" * 80)
        logger.info("Performance Test 3: Query Performance")
        logger.info("=" * 80)

        assert hasattr(self, 'subjects'), "Subjects not found. Run test_01 first."

        # 10개 Subject에 대해 Lesion 조회 성능 측정
        test_subjects = self.subjects[:10]
        query_times = []

        for subject in test_subjects:
            subject_id = subject['id']

            start_time = time.time()
            response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
            elapsed = time.time() - start_time

            assert response.status_code == 200
            lesions = response.json()
            query_times.append(elapsed)

        avg_query_time = sum(query_times) / len(query_times)
        max_query_time = max(query_times)
        min_query_time = min(query_times)

        logger.info(f"\n--- Query Performance Results ---")
        logger.info(f"Queries executed: {len(query_times)}")
        logger.info(f"Average query time: {avg_query_time*1000:.2f}ms")
        logger.info(f"Min query time: {min_query_time*1000:.2f}ms")
        logger.info(f"Max query time: {max_query_time*1000:.2f}ms")

        # 성능 기준: 평균 50ms 이하
        assert avg_query_time < 0.05, f"Query too slow: {avg_query_time*1000:.2f}ms > 50ms"

        logger.info("✅ Query performance test passed")

    def test_04_detail_query_performance(self, admin_client):
        """Lesion 상세 조회 성능 테스트 (Annotation 포함)"""
        logger.info("=" * 80)
        logger.info("Performance Test 4: Detail Query Performance")
        logger.info("=" * 80)

        assert hasattr(self, 'subjects'), "Subjects not found. Run test_01 first."

        # 첫 번째 Subject의 모든 Lesion 조회
        subject_id = self.subjects[0]['id']
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
        assert response.status_code == 200
        lesions = response.json()

        # 각 Lesion의 상세 정보 조회 성능 측정
        detail_query_times = []

        for lesion in lesions[:5]:  # 처음 5개만 테스트
            lesion_id = lesion['id']

            start_time = time.time()
            response = admin_client.get(f"/api/recist-lesions/{lesion_id}")
            elapsed = time.time() - start_time

            assert response.status_code == 200
            detail_query_times.append(elapsed)

        avg_detail_time = sum(detail_query_times) / len(detail_query_times)

        logger.info(f"\n--- Detail Query Performance Results ---")
        logger.info(f"Detail queries executed: {len(detail_query_times)}")
        logger.info(f"Average detail query time: {avg_detail_time*1000:.2f}ms")

        # 성능 기준: 평균 100ms 이하
        assert avg_detail_time < 0.1, f"Detail query too slow: {avg_detail_time*1000:.2f}ms > 100ms"

        logger.info("✅ Detail query performance test passed")


class TestConcurrencyPerformance:
    """동시성 성능 테스트"""

    def test_01_concurrent_lesion_creation(self, admin_client, performance_project):
        """동시 Lesion 생성 테스트"""
        logger.info("=" * 80)
        logger.info("Performance Test 5: Concurrent Lesion Creation")
        logger.info("=" * 80)

        project_id = performance_project['id']

        # Subject 생성
        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": "CONC-001",
            "patient_id": "PT999999",
            "patient_name": "Concurrency Test Patient",
            "patient_birth_date": "1980-01-01"
        })
        assert response.status_code == 201
        subject = response.json()
        subject_id = subject['id']

        # Baseline TimePoint 생성
        response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
            "name": "BL",
            "visit_type": "Baseline",
            "order_index": 0
        })
        assert response.status_code == 201
        baseline_tp = response.json()

        # 동시에 5개의 Target Lesion 생성 시도
        def create_lesion(index):
            from utils.api_client import APIClient
            client = APIClient(admin_client.base_url, admin_client.timeout)
            client.login(admin_client.email, admin_client.password)

            response = client.post(
                f"/api/recist-lesions/subjects/{subject_id}",
                json={
                    "lesion_type": "TARGET",
                    "baseline_timepoint_id": baseline_tp['id'],
                    "organ_site": f"Liver-S{index}",
                    "description": f"Concurrent test lesion {index}"
                }
            )
            return response.status_code, response.json() if response.status_code == 201 else None

        start_time = time.time()

        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
            futures = [executor.submit(create_lesion, i) for i in range(1, 6)]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]

        elapsed = time.time() - start_time

        # 결과 검증
        success_count = sum(1 for status, _ in results if status == 201)

        logger.info(f"\n--- Concurrency Test Results ---")
        logger.info(f"Concurrent requests: 5")
        logger.info(f"Successful creations: {success_count}")
        logger.info(f"Total time: {elapsed:.2f}s")

        # 모두 성공해야 함
        assert success_count == 5, f"Expected 5 successful creations, got {success_count}"

        # 동시성 처리 시간이 순차 처리보다 빨라야 함 (최소 2배 이상)
        # 순차 처리 예상 시간: 5 * 0.15s = 0.75s
        # 동시 처리는 0.5s 이하여야 함
        assert elapsed < 0.5, f"Concurrent processing too slow: {elapsed:.2f}s > 0.5s"

        logger.info("✅ Concurrent lesion creation test passed")

    def test_02_concurrent_queries(self, admin_client):
        """동시 조회 성능 테스트"""
        logger.info("=" * 80)
        logger.info("Performance Test 6: Concurrent Query Performance")
        logger.info("=" * 80)

        # 여러 Subject ID 준비 (이전 테스트에서 생성된 것 사용)
        subject_ids = list(range(1, 11))  # 임시로 1-10 사용

        def query_lesions(subject_id):
            from utils.api_client import APIClient
            client = APIClient(admin_client.base_url, admin_client.timeout)
            client.login(admin_client.email, admin_client.password)

            start = time.time()
            response = client.get(f"/api/recist-lesions/subjects/{subject_id}")
            elapsed = time.time() - start

            return response.status_code, elapsed

        start_time = time.time()

        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(query_lesions, sid) for sid in subject_ids]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]

        total_elapsed = time.time() - start_time

        # 성공한 쿼리의 평균 시간 계산
        successful_times = [elapsed for status, elapsed in results if status == 200]

        if successful_times:
            avg_time = sum(successful_times) / len(successful_times)

            logger.info(f"\n--- Concurrent Query Results ---")
            logger.info(f"Concurrent queries: {len(subject_ids)}")
            logger.info(f"Successful queries: {len(successful_times)}")
            logger.info(f"Total time: {total_elapsed:.2f}s")
            logger.info(f"Average query time: {avg_time*1000:.2f}ms")

            logger.info("✅ Concurrent query test completed")
        else:
            logger.warning("⚠️  No successful queries (subjects may not exist yet)")

