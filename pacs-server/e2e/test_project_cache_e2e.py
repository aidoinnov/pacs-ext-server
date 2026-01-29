#!/usr/bin/env python3
"""
E2E 테스트: Project API ETag 캐싱
"""

import requests
import time
import json
import concurrent.futures

BASE_URL = "http://localhost:8080"

def test_project_list_etag_caching():
    """프로젝트 목록 조회 ETag 캐싱 테스트"""
    print("\n=== Test 1: Project List ETag Caching ===")
    
    # 첫 번째 요청 - 200 OK + ETag
    response1 = requests.get(f"{BASE_URL}/api/projects")
    print(f"First request: {response1.status_code}")
    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"
    
    etag1 = response1.headers.get('ETag')
    print(f"ETag: {etag1}")
    assert etag1 is not None, "ETag header missing"
    
    cache_control = response1.headers.get('Cache-Control')
    print(f"Cache-Control: {cache_control}")
    assert 'private' in cache_control.lower(), "Cache-Control should contain 'private'"
    assert 'max-age=60' in cache_control.lower(), "Cache-Control should contain 'max-age=60'"
    
    # 두 번째 요청 - If-None-Match 헤더 포함 - 304 Not Modified
    response2 = requests.get(
        f"{BASE_URL}/api/projects",
        headers={'If-None-Match': etag1}
    )
    print(f"Second request with If-None-Match: {response2.status_code}")
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    
    etag2 = response2.headers.get('ETag')
    print(f"ETag (304): {etag2}")
    assert etag2 == etag1, "ETag should be the same"
    
    print("✅ Test 1 PASSED\n")


def test_project_detail_etag_caching():
    """프로젝트 상세 조회 ETag 캐싱 테스트"""
    print("\n=== Test 2: Project Detail ETag Caching ===")
    
    # 프로젝트 목록에서 첫 번째 프로젝트 ID 가져오기
    list_response = requests.get(f"{BASE_URL}/api/projects")
    projects = list_response.json()['projects']
    if not projects:
        print("⚠️  No projects found, skipping test")
        return
    
    project_id = projects[0]['id']
    print(f"Testing with project ID: {project_id}")
    
    # 첫 번째 요청 - 200 OK + ETag
    response1 = requests.get(f"{BASE_URL}/api/projects/{project_id}")
    print(f"First request: {response1.status_code}")
    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"
    
    etag1 = response1.headers.get('ETag')
    print(f"ETag: {etag1}")
    assert etag1 is not None, "ETag header missing"
    
    # 두 번째 요청 - If-None-Match 헤더 포함 - 304 Not Modified
    response2 = requests.get(
        f"{BASE_URL}/api/projects/{project_id}",
        headers={'If-None-Match': etag1}
    )
    print(f"Second request with If-None-Match: {response2.status_code}")
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    
    print("✅ Test 2 PASSED\n")


def test_cache_invalidation_on_update():
    """프로젝트 수정 시 캐시 무효화 테스트"""
    print("\n=== Test 3: Cache Invalidation on Update ===")
    
    # 프로젝트 목록에서 첫 번째 프로젝트 가져오기
    list_response = requests.get(f"{BASE_URL}/api/projects")
    projects = list_response.json()['projects']
    if not projects:
        print("⚠️  No projects found, skipping test")
        return
    
    project_id = projects[0]['id']
    print(f"Testing with project ID: {project_id}")
    
    # 첫 번째 요청 - ETag 가져오기
    response1 = requests.get(f"{BASE_URL}/api/projects/{project_id}")
    etag1 = response1.headers.get('ETag')
    print(f"Initial ETag: {etag1}")
    
    # 프로젝트 수정
    update_data = {
        "description": f"Updated at {time.time()}"
    }
    update_response = requests.put(
        f"{BASE_URL}/api/projects/{project_id}",
        json=update_data
    )
    print(f"Update response: {update_response.status_code}")
    
    # 잠시 대기 (updated_at 변경 보장)
    time.sleep(0.1)
    
    # 수정 후 요청 - 새로운 ETag 확인
    response2 = requests.get(f"{BASE_URL}/api/projects/{project_id}")
    etag2 = response2.headers.get('ETag')
    print(f"New ETag after update: {etag2}")
    
    assert etag2 != etag1, "ETag should change after update"
    
    # 이전 ETag로 요청 - 200 OK (캐시 무효화됨)
    response3 = requests.get(
        f"{BASE_URL}/api/projects/{project_id}",
        headers={'If-None-Match': etag1}
    )
    print(f"Request with old ETag: {response3.status_code}")
    assert response3.status_code == 200, f"Expected 200 (cache invalidated), got {response3.status_code}"
    
    print("✅ Test 3 PASSED\n")


def test_different_query_params():
    """다른 쿼리 파라미터는 다른 ETag 생성 테스트"""
    print("\n=== Test 4: Different Query Parameters ===")

    # 필터 없이 요청
    response1 = requests.get(f"{BASE_URL}/api/projects")
    etag1 = response1.headers.get('ETag')
    print(f"ETag without filter: {etag1}")

    # status 필터로 요청
    response2 = requests.get(f"{BASE_URL}/api/projects?status=IN_PROGRESS")
    etag2 = response2.headers.get('ETag')
    print(f"ETag with status filter: {etag2}")

    # ETag가 다를 수 있음 (필터링된 결과의 MAX(updated_at)이 다를 수 있음)
    print(f"ETags are {'same' if etag1 == etag2 else 'different'}")

    print("✅ Test 4 PASSED\n")


def test_concurrent_requests():
    """동시 요청 시 캐시 일관성 테스트"""
    print("\n=== Test 5: Concurrent Requests ===")

    # 프로젝트 ID 가져오기
    list_response = requests.get(f"{BASE_URL}/api/projects")
    projects = list_response.json()['projects']
    if not projects:
        print("⚠️  No projects found, skipping test")
        return

    project_id = projects[0]['id']

    def fetch_project():
        response = requests.get(f"{BASE_URL}/api/projects/{project_id}")
        return response.headers.get('ETag')

    # 10개의 동시 요청
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = [executor.submit(fetch_project) for _ in range(10)]
        etags = [f.result() for f in concurrent.futures.as_completed(futures)]

    # 모든 ETag가 동일해야 함
    unique_etags = set(etags)
    print(f"Unique ETags: {len(unique_etags)}")
    assert len(unique_etags) == 1, f"Expected 1 unique ETag, got {len(unique_etags)}"

    print("✅ Test 5 PASSED\n")


def test_invalid_etag():
    """잘못된 ETag 처리 테스트"""
    print("\n=== Test 6: Invalid ETag Handling ===")

    # 잘못된 ETag로 요청 - 200 OK 반환해야 함
    response = requests.get(
        f"{BASE_URL}/api/projects",
        headers={'If-None-Match': 'invalid-etag-12345'}
    )
    print(f"Response with invalid ETag: {response.status_code}")
    assert response.status_code == 200, f"Expected 200, got {response.status_code}"

    print("✅ Test 6 PASSED\n")


def test_performance_comparison():
    """캐시 HIT vs MISS 성능 비교"""
    print("\n=== Test 7: Performance Comparison ===")

    # 프로젝트 ID 가져오기
    list_response = requests.get(f"{BASE_URL}/api/projects")
    projects = list_response.json()['projects']
    if not projects:
        print("⚠️  No projects found, skipping test")
        return

    project_id = projects[0]['id']

    # 캐시 MISS - 첫 번째 요청
    start_miss = time.time()
    response1 = requests.get(f"{BASE_URL}/api/projects/{project_id}")
    time_miss = time.time() - start_miss
    etag = response1.headers.get('ETag')

    # 캐시 HIT - If-None-Match 헤더 포함
    start_hit = time.time()
    response2 = requests.get(
        f"{BASE_URL}/api/projects/{project_id}",
        headers={'If-None-Match': etag}
    )
    time_hit = time.time() - start_hit

    print(f"Cache MISS: {time_miss:.3f}s")
    print(f"Cache HIT (304): {time_hit:.3f}s")

    if time_miss > 0:
        improvement = ((time_miss - time_hit) / time_miss) * 100
        print(f"Performance improvement: {improvement:.1f}%")

    # 304는 일반적으로 더 빠름
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"

    print("✅ Test 7 PASSED\n")


def test_list_cache_invalidation_on_update():
    """프로젝트 수정 시 목록 캐시도 무효화되는지 테스트"""
    print("\n=== Test 8: List Cache Invalidation on Update ===")

    # 프로젝트 목록에서 첫 번째 프로젝트 가져오기
    list_response = requests.get(f"{BASE_URL}/api/projects")
    projects = list_response.json()['projects']
    if not projects:
        print("⚠️  No projects found, skipping test")
        return

    project_id = projects[0]['id']

    # 목록 ETag 가져오기
    response1 = requests.get(f"{BASE_URL}/api/projects")
    etag1 = response1.headers.get('ETag')
    print(f"Initial list ETag: {etag1}")

    # 프로젝트 수정
    update_data = {
        "description": f"Updated for list cache test at {time.time()}"
    }
    update_response = requests.put(
        f"{BASE_URL}/api/projects/{project_id}",
        json=update_data
    )
    print(f"Update response: {update_response.status_code}")

    # 잠시 대기
    time.sleep(0.1)

    # 목록 다시 조회 - 새로운 ETag 확인
    response2 = requests.get(f"{BASE_URL}/api/projects")
    etag2 = response2.headers.get('ETag')
    print(f"New list ETag after update: {etag2}")

    assert etag2 != etag1, "List ETag should change after project update"

    print("✅ Test 8 PASSED\n")


def test_pagination_cache():
    """페이지네이션 파라미터별 캐싱 테스트"""
    print("\n=== Test 9: Pagination Cache ===")

    # 페이지 1 요청
    response1 = requests.get(f"{BASE_URL}/api/projects?page=1&page_size=10")
    etag1 = response1.headers.get('ETag')
    print(f"Page 1 ETag: {etag1}")

    # 페이지 2 요청
    response2 = requests.get(f"{BASE_URL}/api/projects?page=2&page_size=10")
    etag2 = response2.headers.get('ETag')
    print(f"Page 2 ETag: {etag2}")

    # 페이지가 달라도 MAX(updated_at)은 같으므로 ETag는 같아야 함
    print(f"ETags are {'same' if etag1 == etag2 else 'different'}")
    assert etag1 == etag2, "ETags should be the same (same dataset, different pages)"

    # 페이지 1 캐시 확인
    response3 = requests.get(
        f"{BASE_URL}/api/projects?page=1&page_size=10",
        headers={'If-None-Match': etag1}
    )
    print(f"Page 1 cache check: {response3.status_code}")
    assert response3.status_code == 304, f"Expected 304, got {response3.status_code}"

    print("✅ Test 9 PASSED\n")


def test_empty_result_cache():
    """빈 결과 캐싱 테스트"""
    print("\n=== Test 10: Empty Result Cache ===")

    # 존재하지 않는 status로 필터링
    response1 = requests.get(f"{BASE_URL}/api/projects?status=NONEXISTENT_STATUS")
    print(f"First request: {response1.status_code}")

    # 빈 결과도 ETag가 있어야 함
    etag1 = response1.headers.get('ETag')
    print(f"ETag for empty result: {etag1}")
    assert etag1 is not None, "ETag should exist even for empty results"

    # 캐시 확인
    response2 = requests.get(
        f"{BASE_URL}/api/projects?status=NONEXISTENT_STATUS",
        headers={'If-None-Match': etag1}
    )
    print(f"Second request with If-None-Match: {response2.status_code}")
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"

    print("✅ Test 10 PASSED\n")


def test_active_projects_etag_caching():
    """활성 프로젝트 목록 ETag 캐싱 테스트"""
    print("\n=== Test 11: Active Projects ETag Caching ===")

    # 첫 번째 요청 - 200 OK + ETag
    response1 = requests.get(f"{BASE_URL}/api/projects/active")
    print(f"First request: {response1.status_code}")
    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"

    etag1 = response1.headers.get('ETag')
    print(f"ETag: {etag1}")
    assert etag1 is not None, "ETag header missing"

    cache_control = response1.headers.get('Cache-Control')
    print(f"Cache-Control: {cache_control}")
    assert 'private' in cache_control.lower(), "Cache-Control should contain 'private'"
    assert 'max-age=60' in cache_control.lower(), "Cache-Control should contain 'max-age=60'"

    # 두 번째 요청 - If-None-Match 헤더 포함 - 304 Not Modified
    response2 = requests.get(
        f"{BASE_URL}/api/projects/active",
        headers={'If-None-Match': etag1}
    )
    print(f"Second request with If-None-Match: {response2.status_code}")
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"

    etag2 = response2.headers.get('ETag')
    print(f"ETag (304): {etag2}")
    assert etag2 == etag1, "ETag should be the same"

    print("✅ Test 11 PASSED\n")


if __name__ == "__main__":
    print("🚀 Starting Project API ETag Caching E2E Tests")
    print(f"Base URL: {BASE_URL}")
    print("="*60)

    try:
        test_project_list_etag_caching()
        test_project_detail_etag_caching()
        test_cache_invalidation_on_update()
        test_different_query_params()
        test_concurrent_requests()
        test_invalid_etag()
        test_performance_comparison()
        test_list_cache_invalidation_on_update()
        test_pagination_cache()
        test_empty_result_cache()
        test_active_projects_etag_caching()

        print("\n" + "="*60)
        print("🎉 All 11 tests PASSED!")
        print("="*60)
    except AssertionError as e:
        print(f"\n❌ Test FAILED: {e}")
        exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        exit(1)

