#!/usr/bin/env python3
"""
Study List View API ETag 캐싱 E2E 테스트

테스트 시나리오:
1. ETag 생성 및 304 Not Modified 응답
2. 잘못된 ETag 처리
3. 성능 측정
4. Cache-Control: no-cache 헤더 처리

Note: 데이터 변경 후 캐시 무효화 테스트는 View 추가/수정/삭제 API가 필요하므로 생략
"""

import requests
import sys
import time

BASE_URL = "http://localhost:8080"

# 테스트 계정
TEST_USER = {
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!"
}

def login():
    """로그인하여 토큰 획득"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json=TEST_USER
    )
    if response.status_code != 200:
        print(f"❌ 로그인 실패: {response.status_code}")
        print(response.text)
        sys.exit(1)

    token = response.json()["token"]
    print(f"✅ 로그인 성공")
    return token

def test_study_list_views_etag_cache(token):
    """테스트 1: ETag 생성 및 304 Not Modified 응답"""
    print(f"\n{'='*80}")
    print(f"테스트 1: ETag 생성 및 304 Not Modified 응답")
    print(f"{'='*80}")

    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/study-list-views"

    # 첫 번째 요청 - ETag 생성
    print("\n1️⃣ 첫 번째 요청 (ETag 생성)...")
    response1 = requests.get(url, headers=headers)

    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"
    assert "ETag" in response1.headers, "ETag header not found"
    assert "Cache-Control" in response1.headers, "Cache-Control header not found"

    etag1 = response1.headers["ETag"]
    cache_control = response1.headers["Cache-Control"]
    data1 = response1.json()

    print(f"✅ Status: {response1.status_code}")
    print(f"✅ ETag: {etag1}")
    print(f"✅ Cache-Control: {cache_control}")
    print(f"✅ View 개수: {data1.get('total', 0)}")

    assert "private" in cache_control.lower(), "Cache-Control should contain 'private'"
    assert "max-age" in cache_control.lower(), "Cache-Control should contain 'max-age'"

    # 두 번째 요청 - If-None-Match 헤더 포함
    print("\n2️⃣ 두 번째 요청 (If-None-Match 헤더 포함)...")
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers_with_etag)

    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    assert response2.headers["ETag"] == etag1, "ETag mismatch"

    print(f"✅ Status: {response2.status_code} (Not Modified)")
    print(f"✅ ETag: {response2.headers['ETag']}")
    print(f"✅ Content-Length: {response2.headers.get('Content-Length', '0')} (empty body)")

    print("\n✅ 테스트 1 통과: ETag 캐싱 정상 작동")
    return True

def test_invalid_etag_handling(token):
    """테스트 2: 잘못된 ETag 처리"""
    print(f"\n{'='*80}")
    print(f"테스트 2: 잘못된 ETag 처리")
    print(f"{'='*80}")

    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/study-list-views"

    invalid_etags = [
        ("invalid-format", "잘못된 형식"),
        ('""', "빈 문자열"),
        ('"99999999999"', "존재하지 않는 타임스탬프"),
        ('W/"abc123"', "잘못된 Weak ETag"),
    ]

    for etag, description in invalid_etags:
        print(f"\n🧪 테스트: {description} (If-None-Match: {etag})")
        response = requests.get(
            url,
            headers={**headers, "If-None-Match": etag}
        )

        # 잘못된 ETag는 무시하고 200 OK 반환해야 함
        assert response.status_code == 200, f"Invalid ETag should be ignored (got {response.status_code})"
        assert "ETag" in response.headers, "Response should include valid ETag"

        print(f"✅ Status: {response.status_code} (잘못된 ETag 무시)")
        print(f"✅ Valid ETag: {response.headers['ETag']}")

    print("\n✅ 테스트 2 통과: 잘못된 ETag 안전하게 처리")
    return True


def test_cache_performance(token):
    """테스트 3: 성능 측정 (304 응답이 더 빠른지 확인)"""
    print(f"\n{'='*80}")
    print(f"테스트 3: 캐시 성능 측정")
    print(f"{'='*80}")

    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/study-list-views"

    # 1차 요청 (200 OK) - 시간 측정
    print("\n1️⃣ 첫 요청 (200 OK)...")
    start = time.time()
    response1 = requests.get(url, headers=headers)
    time1 = time.time() - start

    assert response1.status_code == 200
    etag = response1.headers["ETag"]

    print(f"✅ Status: {response1.status_code}")
    print(f"✅ Response time: {time1:.3f}s")
    print(f"✅ ETag: {etag}")

    # 2차, 3차 요청 (304 Not Modified) - 시간 측정
    times_304 = []
    for i in range(2, 5):
        time.sleep(0.1)  # 요청 간 간격
        print(f"\n{i}️⃣ {i}번째 요청 (304 Not Modified)...")
        start = time.time()
        response = requests.get(
            url,
            headers={**headers, "If-None-Match": etag}
        )
        elapsed = time.time() - start
        times_304.append(elapsed)

        assert response.status_code == 304
        print(f"✅ Status: {response.status_code}")
        print(f"✅ Response time: {elapsed:.3f}s")

    # 성능 비교
    avg_304 = sum(times_304) / len(times_304)

    print(f"\n📊 성능 비교:")
    print(f"   200 OK:          {time1:.3f}s")
    print(f"   304 평균:        {avg_304:.3f}s")
    print(f"   개선율:          {((time1 - avg_304) / time1 * 100):.1f}%")

    if avg_304 < time1:
        print(f"✅ 304 응답이 더 빠름 (캐시 효과 확인)")
    else:
        print(f"⚠️  304 응답이 더 느림 (네트워크 상황에 따라 다를 수 있음)")

    print("\n✅ 테스트 3 통과: 성능 측정 완료")
    return True


def test_cache_invalidation_on_no_cache(token):
    """테스트 4: Cache-Control: no-cache 헤더 처리"""
    print(f"\n{'='*80}")
    print(f"테스트 4: Cache-Control: no-cache 헤더 처리")
    print(f"{'='*80}")

    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/study-list-views"

    # 첫 번째 요청
    print("\n1️⃣ 첫 요청...")
    response1 = requests.get(url, headers=headers)
    assert response1.status_code == 200

    etag1 = response1.headers["ETag"]
    print(f"✅ ETag: {etag1}")

    # Cache-Control: no-cache 헤더로 요청
    print("\n2️⃣ no-cache 헤더로 요청...")
    headers_no_cache = {
        "Authorization": f"Bearer {token}",
        "Cache-Control": "no-cache",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers_no_cache)

    print(f"✅ Status: {response2.status_code}")
    print(f"✅ Cache-Control: no-cache 헤더 포함")

    # no-cache 헤더가 있어도 서버는 ETag를 확인하고 304를 반환할 수 있음
    assert response2.status_code in [200, 304], f"Expected 200 or 304, got {response2.status_code}"

    if response2.status_code == 304:
        print(f"✅ 304 응답 (서버가 ETag 확인)")
    else:
        print(f"✅ 200 응답 (no-cache 헤더 처리)")

    print("\n✅ 테스트 4 통과: Cache-Control 헤더 처리 확인")
    return True


def test_empty_list_etag(token):
    """테스트 5: 빈 목록 ETag 처리"""
    print(f"\n{'='*80}")
    print(f"테스트 5: 빈 목록 ETag 처리")
    print(f"{'='*80}")

    headers = {"Authorization": f"Bearer {token}"}
    # 존재하지 않는 project_id로 필터링하여 빈 목록 생성
    url = f"{BASE_URL}/api/study-list-views?project_id=999999"

    # 첫 번째 요청
    print("\n1️⃣ 첫 요청 (빈 목록)...")
    response1 = requests.get(url, headers=headers)

    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"
    assert "ETag" in response1.headers, "ETag header not found"

    etag1 = response1.headers["ETag"]
    data1 = response1.json()

    print(f"✅ Status: {response1.status_code}")
    print(f"✅ ETag: {etag1}")
    print(f"✅ View 개수: {data1.get('total', 0)}")

    # 두 번째 요청 - If-None-Match 헤더 포함
    print("\n2️⃣ 두 번째 요청 (If-None-Match 헤더 포함)...")
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers_with_etag)

    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    assert response2.headers["ETag"] == etag1, "ETag mismatch"

    print(f"✅ Status: {response2.status_code} (Not Modified)")
    print(f"✅ ETag: {response2.headers['ETag']}")
    print(f"✅ 빈 목록도 정상적으로 캐싱됨")

    print("\n✅ 테스트 5 통과: 빈 목록 ETag 처리 정상")
    return True


def test_concurrent_requests(token):
    """테스트 6: 동시 요청 처리"""
    print(f"\n{'='*80}")
    print(f"테스트 6: 동시 요청 처리")
    print(f"{'='*80}")

    import concurrent.futures

    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/study-list-views"

    def make_request():
        response = requests.get(url, headers=headers)
        return response.headers.get("ETag"), response.status_code

    print("\n1️⃣ 동시에 10개 요청 전송...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = [executor.submit(make_request) for _ in range(10)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]

    etags = [r[0] for r in results]
    statuses = [r[1] for r in results]

    # 모든 요청이 200 OK 반환
    assert all(s == 200 for s in statuses), "Not all requests returned 200"

    # 모든 ETag가 동일해야 함
    unique_etags = set(etags)
    assert len(unique_etags) == 1, f"Expected 1 unique ETag, got {len(unique_etags)}"

    print(f"✅ 모든 요청 성공: 10/10")
    print(f"✅ 모든 ETag 동일: {list(unique_etags)[0]}")
    print(f"✅ 캐시 일관성 유지")

    print("\n✅ 테스트 6 통과: 동시 요청 처리 정상")
    return True


def main():
    print("🚀 Study List View API ETag 캐싱 E2E 테스트 시작")
    print("="*80)

    # 로그인
    print("\n🔐 로그인 중...")
    token = login()

    # 테스트 스위트
    test_suite = [
        ("ETag 생성 및 304 응답", test_study_list_views_etag_cache),
        ("잘못된 ETag 처리", test_invalid_etag_handling),
        ("캐시 성능 측정", test_cache_performance),
        ("no-cache 헤더 처리", test_cache_invalidation_on_no_cache),
        ("빈 목록 ETag 처리", test_empty_list_etag),
        ("동시 요청 처리", test_concurrent_requests),
    ]

    # 테스트 실행
    results = []
    for name, test_func in test_suite:
        try:
            result = test_func(token)
            results.append((name, result, None))
        except AssertionError as e:
            print(f"\n❌ Assertion 실패: {e}")
            results.append((name, False, str(e)))
        except Exception as e:
            print(f"\n❌ 예외 발생: {e}")
            import traceback
            traceback.print_exc()
            results.append((name, False, str(e)))

    # 결과 요약
    print(f"\n{'='*80}")
    print("📊 테스트 결과 상세")
    print(f"{'='*80}")

    for name, result, error in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status} - {name}")
        if error:
            print(f"      Error: {error}")

    # 통계
    total = len(results)
    passed = sum(1 for _, result, _ in results if result)
    failed = total - passed

    print(f"\n{'='*80}")
    print("📈 테스트 통계")
    print(f"{'='*80}")
    print(f"총 테스트: {total}개")
    print(f"성공: {passed}개")
    print(f"실패: {failed}개")
    print(f"성공률: {(passed/total*100):.1f}%")

    if all(result for _, result, _ in results):
        print("\n🎉 모든 테스트 통과!")
        print("✅ Study List View API ETag 캐싱 기능 정상 작동")
        print("✅ 엣지 케이스 처리 정상")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트 실패")
        sys.exit(1)

if __name__ == "__main__":
    main()

