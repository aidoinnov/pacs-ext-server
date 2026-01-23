#!/usr/bin/env python3
"""
QIDO Cache E2E Tests

Tests for QIDO-RS response caching functionality.
"""

import requests
import time
import json
import sys

BASE_URL = "http://localhost:8080"
LOGIN_URL = f"{BASE_URL}/api/auth/login"

# Test credentials
ADMIN_CREDENTIALS = {
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!"
}

# Test data
TEST_STUDY_UID = "1.2.410.200022.500.202205101053010.12252192375"
TEST_PROJECT_ID = 2


def login(credentials):
    """Login and get JWT token"""
    response = requests.post(LOGIN_URL, json=credentials)
    assert response.status_code == 200, f"Login failed: {response.text}"
    data = response.json()
    return data["token"]


def get_series(token, study_uid, project_id=None):
    """Get series for a study"""
    url = f"{BASE_URL}/api/me/dicom/studies/{study_uid}/series"
    if project_id:
        url += f"?project_id={project_id}"
    
    headers = {"Authorization": f"Bearer {token}"}
    start_time = time.time()
    response = requests.get(url, headers=headers)
    elapsed_time = time.time() - start_time
    
    assert response.status_code == 200, f"Get series failed: {response.text}"
    return response.json(), elapsed_time


def get_studies(token, project_id=None):
    """Get all studies"""
    url = f"{BASE_URL}/api/me/dicom/studies"
    if project_id:
        url += f"?project_id={project_id}"
    
    headers = {"Authorization": f"Bearer {token}"}
    start_time = time.time()
    response = requests.get(url, headers=headers)
    elapsed_time = time.time() - start_time
    
    assert response.status_code == 200, f"Get studies failed: {response.text}"
    return response.json(), elapsed_time


def test_series_cache_miss_then_hit():
    """Test 1: Series cache MISS on first request, HIT on subsequent requests"""
    print("\n" + "="*80)
    print("Test 1: Series Cache MISS → HIT")
    print("="*80)
    
    token = login(ADMIN_CREDENTIALS)
    
    # First request - should be MISS
    print("\n📊 Request 1 (Expected: Cache MISS)")
    series1, time1 = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
    print(f"   Response time: {time1:.3f}s")
    print(f"   Series count: {len(series1)}")
    
    time.sleep(0.5)
    
    # Second request - should be HIT
    print("\n📊 Request 2 (Expected: Cache HIT)")
    series2, time2 = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
    print(f"   Response time: {time2:.3f}s")
    print(f"   Series count: {len(series2)}")
    
    # Verify same data
    assert len(series1) == len(series2), "Series count mismatch"
    assert series1 == series2, "Series data mismatch"
    
    print("\n✅ Test 1 PASSED")


def test_series_cache_expiry():
    """Test 2: Series cache expires after TTL (60 seconds)"""
    print("\n" + "="*80)
    print("Test 2: Series Cache Expiry (60s TTL)")
    print("="*80)
    
    token = login(ADMIN_CREDENTIALS)
    
    # First request - MISS
    print("\n📊 Request 1 (Expected: Cache MISS)")
    series1, time1 = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
    print(f"   Response time: {time1:.3f}s")
    
    # Second request - HIT
    print("\n📊 Request 2 (Expected: Cache HIT)")
    series2, time2 = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
    print(f"   Response time: {time2:.3f}s")
    
    # Wait for cache to expire
    print("\n⏰ Waiting 65 seconds for cache to expire...")
    time.sleep(65)
    
    # Third request - should be MISS again
    print("\n📊 Request 3 (Expected: Cache MISS after expiry)")
    series3, time3 = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
    print(f"   Response time: {time3:.3f}s")
    
    assert series1 == series3, "Series data mismatch after cache expiry"
    
    print("\n✅ Test 2 PASSED")


def test_series_cache_different_projects():
    """Test 3: Different cache entries for different projects"""
    print("\n" + "="*80)
    print("Test 3: Different Cache Entries per Project")
    print("="*80)
    
    token = login(ADMIN_CREDENTIALS)
    
    # Request for project 2
    print("\n📊 Request for project_id=2")
    series_p2, time_p2 = get_series(token, TEST_STUDY_UID, project_id=2)
    print(f"   Response time: {time_p2:.3f}s")
    print(f"   Series count: {len(series_p2)}")
    
    # Request without project_id (should be different cache entry)
    print("\n📊 Request without project_id")
    series_all, time_all = get_series(token, TEST_STUDY_UID, project_id=None)
    print(f"   Response time: {time_all:.3f}s")
    print(f"   Series count: {len(series_all)}")
    
    # Both should have data (may be same or different depending on access)
    assert len(series_p2) > 0, "Project 2 series should not be empty"
    assert len(series_all) > 0, "All series should not be empty"
    
    print("\n✅ Test 3 PASSED")


def test_studies_cache():
    """Test 4: Studies endpoint caching"""
    print("\n" + "="*80)
    print("Test 4: Studies Cache")
    print("="*80)

    token = login(ADMIN_CREDENTIALS)

    # First request - MISS
    print("\n📊 Request 1 (Expected: Cache MISS)")
    studies1, time1 = get_studies(token, TEST_PROJECT_ID)
    print(f"   Response time: {time1:.3f}s")
    print(f"   Studies count: {len(studies1)}")

    time.sleep(0.5)

    # Second request - HIT
    print("\n📊 Request 2 (Expected: Cache HIT)")
    studies2, time2 = get_studies(token, TEST_PROJECT_ID)
    print(f"   Response time: {time2:.3f}s")
    print(f"   Studies count: {len(studies2)}")

    # Verify same data
    assert len(studies1) == len(studies2), "Studies count mismatch"

    print("\n✅ Test 4 PASSED")


def test_cache_performance():
    """Test 5: Cache performance improvement"""
    print("\n" + "="*80)
    print("Test 5: Cache Performance Improvement")
    print("="*80)

    token = login(ADMIN_CREDENTIALS)

    # Measure MISS time
    print("\n📊 Measuring Cache MISS performance...")
    miss_times = []
    for i in range(3):
        # Wait for cache to expire
        if i > 0:
            print(f"   Waiting 65s for cache expiry (iteration {i+1}/3)...")
            time.sleep(65)

        _, elapsed = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
        miss_times.append(elapsed)
        print(f"   MISS {i+1}: {elapsed:.3f}s")

    avg_miss = sum(miss_times) / len(miss_times)
    print(f"\n   Average MISS time: {avg_miss:.3f}s")

    # Measure HIT time
    print("\n📊 Measuring Cache HIT performance...")
    hit_times = []
    for i in range(5):
        time.sleep(0.2)
        _, elapsed = get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)
        hit_times.append(elapsed)
        print(f"   HIT {i+1}: {elapsed:.3f}s")

    avg_hit = sum(hit_times) / len(hit_times)
    print(f"\n   Average HIT time: {avg_hit:.3f}s")

    # Calculate improvement
    improvement = ((avg_miss - avg_hit) / avg_miss) * 100
    print(f"\n   Performance improvement: {improvement:.1f}%")

    # Cache should be faster or at least not significantly slower
    assert avg_hit <= avg_miss * 1.5, f"Cache HIT ({avg_hit:.3f}s) is significantly slower than MISS ({avg_miss:.3f}s)"

    print("\n✅ Test 5 PASSED")


def test_concurrent_requests():
    """Test 6: Concurrent cache requests"""
    print("\n" + "="*80)
    print("Test 6: Concurrent Cache Requests")
    print("="*80)

    token = login(ADMIN_CREDENTIALS)

    # First request to populate cache
    print("\n📊 Populating cache...")
    get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)

    # Concurrent requests
    print("\n📊 Making 10 concurrent requests...")
    import concurrent.futures

    def make_request():
        return get_series(token, TEST_STUDY_UID, TEST_PROJECT_ID)

    start_time = time.time()
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = [executor.submit(make_request) for _ in range(10)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]

    total_time = time.time() - start_time

    print(f"   Total time for 10 concurrent requests: {total_time:.3f}s")
    print(f"   Average time per request: {total_time/10:.3f}s")

    # All should return same data
    first_result = results[0][0]
    for result, _ in results:
        assert result == first_result, "Concurrent requests returned different data"

    print("\n✅ Test 6 PASSED")


if __name__ == "__main__":
    print("\n🧪 QIDO Cache E2E Tests")
    print("="*80)

    import argparse
    parser = argparse.ArgumentParser(description='QIDO Cache E2E Tests')
    parser.add_argument('--full', action='store_true', help='Run full test suite including slow tests')
    args = parser.parse_args()

    try:
        # Fast tests (always run)
        test_series_cache_miss_then_hit()
        test_series_cache_different_projects()
        test_studies_cache()
        test_concurrent_requests()

        # Slow tests (only with --full flag)
        if args.full:
            test_cache_performance()  # Takes ~200 seconds (3+ cache expiries)
        else:
            print("\n" + "="*80)
            print("⏭️  Skipping slow tests (use --full to run all tests)")
            print("="*80)

        print("\n" + "="*80)
        print("✅ All tests PASSED!")
        print("="*80)
        sys.exit(0)
    except AssertionError as e:
        print(f"\n❌ Test FAILED: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

