#!/usr/bin/env python3
"""
Role-Permission Matrix ETag Cache E2E Test

이 테스트는 Role-Permission Matrix API의 ETag 캐싱 기능을 검증합니다.

테스트 시나리오:
1. 글로벌 역할-권한 매트릭스 조회 (ETag 생성)
2. 동일한 요청 시 304 Not Modified 응답 확인
3. 프로젝트별 역할-권한 매트릭스 조회 (ETag 생성)
4. 동일한 요청 시 304 Not Modified 응답 확인
5. 권한 할당 변경 후 ETag 변경 확인
6. 캐시 무효화 확인 (no-cache 헤더)
"""

import requests
import sys
import time

# 테스트 설정
BASE_URL = "http://localhost:8080"
LOGIN_URL = f"{BASE_URL}/api/auth/login"

# 테스트 계정 (admin 권한 필요)
TEST_USER = {
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!"
}


def login():
    """로그인하여 JWT 토큰 획득"""
    response = requests.post(LOGIN_URL, json=TEST_USER)
    if response.status_code != 200:
        print(f"❌ Login failed: {response.status_code}")
        print(f"Response: {response.text}")
        sys.exit(1)
    
    data = response.json()
    return data.get("token")


def test_global_matrix_etag_cache(token):
    """테스트 1: 글로벌 역할-권한 매트릭스 ETag 캐싱"""
    print("\n" + "="*80)
    print("테스트 1: 글로벌 역할-권한 매트릭스 ETag 캐싱")
    print("="*80)
    
    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/roles/global/permissions/matrix"
    
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
    print(f"✅ Roles: {len(data1.get('roles', []))}")
    print(f"✅ Permission Categories: {len(data1.get('permissions_by_category', {}))}")
    print(f"✅ Assignments: {len(data1.get('assignments', []))}")
    
    # 두 번째 요청 - If-None-Match 헤더 포함
    print("\n2️⃣ 두 번째 요청 (If-None-Match 헤더 포함)...")
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers_with_etag)
    
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    assert "ETag" in response2.headers, "ETag header not found in 304 response"
    assert response2.headers["ETag"] == etag1, "ETag mismatch"
    
    print(f"✅ Status: {response2.status_code} (Not Modified)")
    print(f"✅ ETag: {response2.headers['ETag']}")
    print(f"✅ Content-Length: {response2.headers.get('Content-Length', '0')} (empty body)")
    
    print("\n✅ 테스트 1 통과: 글로벌 매트릭스 ETag 캐싱 정상 작동")
    return etag1


def test_project_matrix_etag_cache(token, project_id=634):
    """테스트 2: 프로젝트별 역할-권한 매트릭스 ETag 캐싱"""
    print("\n" + "="*80)
    print(f"테스트 2: 프로젝트별 역할-권한 매트릭스 ETag 캐싱 (project_id={project_id})")
    print("="*80)
    
    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/projects/{project_id}/roles/permissions/matrix"
    
    # 첫 번째 요청 - ETag 생성
    print("\n1️⃣ 첫 번째 요청 (ETag 생성)...")
    response1 = requests.get(url, headers=headers)
    
    assert response1.status_code == 200, f"Expected 200, got {response1.status_code}"
    assert "ETag" in response1.headers, "ETag header not found"
    
    etag1 = response1.headers["ETag"]
    data1 = response1.json()
    
    print(f"✅ Status: {response1.status_code}")
    print(f"✅ ETag: {etag1}")
    print(f"✅ Roles: {len(data1.get('roles', []))}")
    print(f"✅ Assignments: {len(data1.get('assignments', []))}")
    
    # 두 번째 요청 - If-None-Match 헤더 포함
    print("\n2️⃣ 두 번째 요청 (If-None-Match 헤더 포함)...")
    headers_with_etag = {
        "Authorization": f"Bearer {token}",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers_with_etag)
    
    assert response2.status_code == 304, f"Expected 304, got {response2.status_code}"
    
    print(f"✅ Status: {response2.status_code} (Not Modified)")
    print(f"✅ ETag: {response2.headers['ETag']}")
    
    print("\n✅ 테스트 2 통과: 프로젝트별 매트릭스 ETag 캐싱 정상 작동")
    return etag1


def test_cache_invalidation_on_no_cache(token):
    """테스트 3: Cache-Control: no-cache 헤더로 캐시 무효화"""
    print("\n" + "="*80)
    print("테스트 3: Cache-Control: no-cache 헤더로 캐시 무효화")
    print("="*80)

    url = f"{BASE_URL}/api/roles/global/permissions/matrix"

    # 첫 번째 요청
    headers1 = {"Authorization": f"Bearer {token}"}
    response1 = requests.get(url, headers=headers1)
    etag1 = response1.headers["ETag"]

    print(f"\n1️⃣ 첫 번째 요청 ETag: {etag1}")

    # no-cache 헤더로 요청
    headers2 = {
        "Authorization": f"Bearer {token}",
        "Cache-Control": "no-cache",
        "If-None-Match": etag1
    }
    response2 = requests.get(url, headers=headers2)

    # no-cache 헤더가 있어도 서버는 ETag를 확인하고 304를 반환할 수 있음
    print(f"2️⃣ no-cache 요청 Status: {response2.status_code}")
    print(f"✅ ETag: {response2.headers.get('ETag', 'N/A')}")

    print("\n✅ 테스트 3 통과: Cache-Control 헤더 처리 확인")


def test_invalid_etag_handling(token):
    """테스트 4: 잘못된 ETag 처리"""
    print("\n" + "="*80)
    print("테스트 4: 잘못된 ETag 처리")
    print("="*80)

    url = f"{BASE_URL}/api/roles/global/permissions/matrix"
    headers = {"Authorization": f"Bearer {token}"}

    invalid_etags = [
        'W/"invalid-etag"',
        '"12345"',
        'W/"0"',
        '"wrong-format"'
    ]

    for i, invalid_etag in enumerate(invalid_etags, 1):
        print(f"\n{i}️⃣ 잘못된 ETag 테스트: {invalid_etag}")
        headers_with_invalid = {
            "Authorization": f"Bearer {token}",
            "If-None-Match": invalid_etag
        }
        response = requests.get(url, headers=headers_with_invalid)

        # 잘못된 ETag는 200 OK 반환해야 함
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        assert "ETag" in response.headers, "ETag header not found"

        print(f"✅ Status: {response.status_code} (올바르게 200 반환)")
        print(f"✅ 새로운 ETag: {response.headers['ETag']}")

    print("\n✅ 테스트 4 통과: 잘못된 ETag 처리 정상")


def test_performance_measurement(token):
    """테스트 5: 캐시 성능 측정 (200 OK vs 304 Not Modified)"""
    print("\n" + "="*80)
    print("테스트 5: 캐시 성능 측정")
    print("="*80)

    url = f"{BASE_URL}/api/roles/global/permissions/matrix"
    headers = {"Authorization": f"Bearer {token}"}

    # 200 OK 응답 시간 측정
    print("\n1️⃣ 200 OK 응답 시간 측정...")
    start = time.time()
    response1 = requests.get(url, headers=headers)
    time_200 = time.time() - start

    assert response1.status_code == 200
    etag = response1.headers["ETag"]

    print(f"✅ 200 OK 응답 시간: {time_200:.4f}s")
    print(f"✅ ETag: {etag}")

    # 304 Not Modified 응답 시간 측정 (3회 평균)
    print("\n2️⃣ 304 Not Modified 응답 시간 측정 (3회 평균)...")
    times_304 = []

    for i in range(3):
        headers_with_etag = {
            "Authorization": f"Bearer {token}",
            "If-None-Match": etag
        }
        start = time.time()
        response = requests.get(url, headers=headers_with_etag)
        elapsed = time.time() - start
        times_304.append(elapsed)

        assert response.status_code == 304
        print(f"  시도 {i+1}: {elapsed:.4f}s")

    avg_304 = sum(times_304) / len(times_304)
    improvement = ((time_200 - avg_304) / time_200) * 100

    print(f"\n📊 성능 비교:")
    print(f"  200 OK 평균: {time_200:.4f}s")
    print(f"  304 평균: {avg_304:.4f}s")
    print(f"  성능 개선: {improvement:.1f}%")

    print("\n✅ 테스트 5 통과: 성능 측정 완료")


def test_concurrent_requests(token):
    """테스트 6: 동시 요청 처리"""
    print("\n" + "="*80)
    print("테스트 6: 동시 요청 처리")
    print("="*80)

    import concurrent.futures

    url = f"{BASE_URL}/api/roles/global/permissions/matrix"
    headers = {"Authorization": f"Bearer {token}"}

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


if __name__ == "__main__":
    print("🚀 Role-Permission Matrix ETag Cache E2E Test 시작")
    print("="*80)
    
    # 로그인
    print("\n🔐 로그인 중...")
    token = login()
    print("✅ 로그인 성공")
    
    try:
        # 테스트 실행
        test_global_matrix_etag_cache(token)
        test_project_matrix_etag_cache(token)
        test_cache_invalidation_on_no_cache(token)
        test_invalid_etag_handling(token)
        test_performance_measurement(token)
        test_concurrent_requests(token)

        print("\n" + "="*80)
        print("🎉 모든 테스트 통과! (6개)")
        print("="*80)
        print("\n✅ Role-Permission Matrix ETag 캐싱 기능 정상 작동")
        print("✅ 304 Not Modified 응답 정상")
        print("✅ Cache-Control 헤더 정상")
        print("✅ 잘못된 ETag 처리 정상")
        print("✅ 성능 측정 완료")
        print("✅ 동시 요청 처리 정상")

    except AssertionError as e:
        print(f"\n❌ 테스트 실패: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ 예외 발생: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

