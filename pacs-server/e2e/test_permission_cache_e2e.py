#!/usr/bin/env python3
"""
Permission Management API 캐싱 E2E 테스트

테스트 시나리오:
1. GET /api/permissions - ETag 생성 및 304 응답
2. 잘못된 ETag 처리 (4가지 형식)
3. 성능 측정 (200 vs 304)
4. 동시 요청 처리 (10개 병렬)
5. no-cache 헤더 처리
"""

import requests
import time
import sys
import concurrent.futures

# 설정
BASE_URL = "http://localhost:8080"
API_BASE = f"{BASE_URL}/api"

# 테스트 통계
tests_passed = 0
tests_failed = 0
test_details = []


def print_test_header(title):
    """테스트 헤더 출력"""
    print(f"\n{'=' * 60}")
    print(f"🧪 {title}")
    print('=' * 60)


def print_result(test_name, passed, message="", details=""):
    """테스트 결과 출력"""
    global tests_passed, tests_failed, test_details
    if passed:
        tests_passed += 1
        print(f"✅ {test_name}")
        if message:
            print(f"   {message}")
        test_details.append({"name": test_name, "status": "PASS", "message": message})
    else:
        tests_failed += 1
        print(f"❌ {test_name}")
        if message:
            print(f"   {message}")
        if details:
            print(f"   Details: {details}")
        test_details.append({"name": test_name, "status": "FAIL", "message": message, "details": details})


def login():
    """로그인하여 토큰 획득"""
    response = requests.post(
        f"{API_BASE}/auth/login",
        json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"}
    )
    if response.status_code == 200:
        return response.json()["token"]
    else:
        print(f"❌ Login failed: {response.status_code} - {response.text}")
        sys.exit(1)


def test_etag_generation_and_304(token):
    """테스트 1: ETag 생성 및 304 응답"""
    print_test_header("Test 1: ETag Generation and 304 Response")
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 첫 번째 요청 - 캐시 미스
    response1 = requests.get(f"{API_BASE}/permissions", headers=headers)
    print_result(
        "First request - Cache miss (200 OK)",
        response1.status_code == 200,
        f"Status: {response1.status_code}"
    )
    
    if response1.status_code != 200:
        print(f"   Response: {response1.text}")
        return
    
    # ETag 확인
    etag1 = response1.headers.get("ETag")
    print_result(
        "ETag header present",
        etag1 is not None and etag1.startswith('W/"'),
        f"ETag: {etag1}"
    )
    
    # Cache-Control 확인 (300초 = 5분)
    cache_control = response1.headers.get("Cache-Control")
    print_result(
        "Cache-Control header correct",
        cache_control is not None and "private" in cache_control and "max-age=300" in cache_control,
        f"Cache-Control: {cache_control}"
    )
    
    # 응답 데이터 확인
    permissions = response1.json()
    print_result(
        "Response contains permissions",
        isinstance(permissions, list) and len(permissions) > 0,
        f"Permissions count: {len(permissions)}"
    )
    
    # 2. 두 번째 요청 - If-None-Match 헤더 포함
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag1
    }
    response2 = requests.get(f"{API_BASE}/permissions", headers=headers_with_etag)
    print_result(
        "Second request - Cache hit (304 Not Modified)",
        response2.status_code == 304,
        f"Status: {response2.status_code}"
    )
    
    # 3. 세 번째 요청 - 여전히 캐시됨
    time.sleep(0.5)
    response3 = requests.get(f"{API_BASE}/permissions", headers=headers_with_etag)
    print_result(
        "Third request - Still cached (304)",
        response3.status_code == 304,
        f"Status: {response3.status_code}"
    )


def test_invalid_etag_handling(token):
    """테스트 2: 잘못된 ETag 처리"""
    print_test_header("Test 2: Invalid ETag Handling")

    headers = {"Authorization": f"Bearer {token}"}

    # 먼저 정상 요청으로 ETag 획득
    response = requests.get(f"{API_BASE}/permissions", headers=headers)
    if response.status_code != 200:
        print_result("Failed to get initial response", False, response.text)
        return

    # 1. 잘못된 형식의 ETag (W/ 없음)
    headers_invalid1 = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": '"1234567890"'
    }
    response1 = requests.get(f"{API_BASE}/permissions", headers=headers_invalid1)
    print_result(
        "Invalid ETag format (no W/) - Returns 200",
        response1.status_code == 200,
        f"Status: {response1.status_code}"
    )

    # 2. 빈 ETag
    headers_invalid2 = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": '""'
    }
    response2 = requests.get(f"{API_BASE}/permissions", headers=headers_invalid2)
    print_result(
        "Empty ETag - Returns 200",
        response2.status_code == 200,
        f"Status: {response2.status_code}"
    )

    # 3. 잘못된 값의 ETag
    headers_invalid3 = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": 'W/"wrong-value"'
    }
    response3 = requests.get(f"{API_BASE}/permissions", headers=headers_invalid3)
    print_result(
        "Wrong ETag value - Returns 200",
        response3.status_code == 200,
        f"Status: {response3.status_code}"
    )

    # 4. 따옴표 없는 ETag
    headers_invalid4 = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": "W/1234567890"
    }
    response4 = requests.get(f"{API_BASE}/permissions", headers=headers_invalid4)
    print_result(
        "ETag without quotes - Returns 200",
        response4.status_code == 200,
        f"Status: {response4.status_code}"
    )


def test_performance_measurement(token):
    """테스트 3: 성능 측정 (200 vs 304)"""
    print_test_header("Test 3: Performance Measurement (200 vs 304)")

    headers = {"Authorization": f"Bearer {token}"}

    # 1. 첫 번째 요청 - 200 OK (캐시 미스)
    start_time = time.time()
    response1 = requests.get(f"{API_BASE}/permissions", headers=headers)
    time_200 = (time.time() - start_time) * 1000  # ms

    print_result(
        "200 OK response time measured",
        response1.status_code == 200,
        f"Time: {time_200:.2f}ms"
    )

    if response1.status_code != 200:
        return

    etag = response1.headers.get("ETag")

    # 2. 두 번째 요청 - 304 Not Modified (캐시 히트)
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag
    }
    start_time = time.time()
    response2 = requests.get(f"{API_BASE}/permissions", headers=headers_with_etag)
    time_304 = (time.time() - start_time) * 1000  # ms

    print_result(
        "304 Not Modified response time measured",
        response2.status_code == 304,
        f"Time: {time_304:.2f}ms"
    )

    # 3. 성능 개선 확인
    improvement = ((time_200 - time_304) / time_200) * 100
    print_result(
        "304 is faster than 200",
        time_304 < time_200,
        f"Improvement: {improvement:.1f}% (200: {time_200:.2f}ms, 304: {time_304:.2f}ms)"
    )


def test_concurrent_requests(token):
    """테스트 4: 동시 요청 처리 (10개 병렬)"""
    print_test_header("Test 4: Concurrent Requests (10 parallel)")

    headers = {"Authorization": f"Bearer {token}"}

    # 먼저 ETag 획득
    response = requests.get(f"{API_BASE}/permissions", headers=headers)
    if response.status_code != 200:
        print_result("Failed to get initial ETag", False, response.text)
        return

    etag = response.headers.get("ETag")

    def make_request(index):
        """단일 요청 실행"""
        headers_with_etag = {
            "Authorization": f"Bearer {token}",
            "If-None-Match": etag
        }
        response = requests.get(f"{API_BASE}/permissions", headers=headers_with_etag)
        return {
            "index": index,
            "status": response.status_code,
            "etag": response.headers.get("ETag")
        }

    # 10개 병렬 요청
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = [executor.submit(make_request, i) for i in range(10)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]

    # 모든 요청이 304를 반환했는지 확인
    all_304 = all(r["status"] == 304 for r in results)
    print_result(
        "All 10 concurrent requests returned 304",
        all_304,
        f"Results: {[r['status'] for r in results]}"
    )

    # ETag 일관성 확인
    etags = [r.get("etag") for r in results if r.get("etag")]
    consistent_etags = len(set(etags)) <= 1 if etags else True
    print_result(
        "ETag consistency across requests",
        consistent_etags,
        f"Unique ETags: {len(set(etags)) if etags else 0}"
    )


def test_no_cache_header(token):
    """테스트 5: no-cache 헤더 처리"""
    print_test_header("Test 5: no-cache Header Handling")

    headers = {"Authorization": f"Bearer {token}"}

    # 1. 정상 요청으로 ETag 획득
    response1 = requests.get(f"{API_BASE}/permissions", headers=headers)
    if response1.status_code != 200:
        print_result("Failed to get initial response", False, response1.text)
        return

    etag = response1.headers.get("ETag")

    # 2. Cache-Control: no-cache 헤더와 함께 요청
    headers_no_cache = {
        "Authorization": f"Bearer {token}",
        "Cache-Control": "no-cache",
        "If-None-Match": etag
    }
    response2 = requests.get(f"{API_BASE}/permissions", headers=headers_no_cache)

    # no-cache 헤더가 있어도 ETag가 일치하면 304를 반환해야 함
    # (서버는 클라이언트의 Cache-Control을 무시하고 ETag만 확인)
    print_result(
        "no-cache with matching ETag returns 304",
        response2.status_code == 304,
        f"Status: {response2.status_code}"
    )

    # 3. no-cache 헤더만 있고 If-None-Match 없음
    headers_no_cache_only = {
        "Authorization": f"Bearer {token}",
        "Cache-Control": "no-cache"
    }
    response3 = requests.get(f"{API_BASE}/permissions", headers=headers_no_cache_only)
    print_result(
        "no-cache without ETag returns 200",
        response3.status_code == 200,
        f"Status: {response3.status_code}"
    )


def print_summary():
    """테스트 결과 요약 출력"""
    print("\n" + "=" * 60)
    print("📊 TEST SUMMARY")
    print("=" * 60)

    total = tests_passed + tests_failed
    pass_rate = (tests_passed / total * 100) if total > 0 else 0

    print(f"Total Tests: {total}")
    print(f"✅ Passed: {tests_passed}")
    print(f"❌ Failed: {tests_failed}")
    print(f"Pass Rate: {pass_rate:.1f}%")

    if tests_failed > 0:
        print("\n❌ Failed Tests:")
        for detail in test_details:
            if detail["status"] == "FAIL":
                print(f"  - {detail['name']}")
                if detail.get("message"):
                    print(f"    {detail['message']}")

    print("=" * 60)

    return tests_failed == 0


def main():
    """메인 함수"""
    print("=" * 60)
    print("🚀 Permission Management API Caching E2E Tests")
    print("=" * 60)

    # 로그인
    print("\n🔐 Logging in...")
    token = login()
    print("✅ Login successful")

    # 테스트 실행
    try:
        test_etag_generation_and_304(token)
        test_invalid_etag_handling(token)
        test_performance_measurement(token)
        test_concurrent_requests(token)
        test_no_cache_header(token)
    except Exception as e:
        print(f"\n❌ Test execution failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

    # 결과 요약
    success = print_summary()

    # 종료 코드
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

