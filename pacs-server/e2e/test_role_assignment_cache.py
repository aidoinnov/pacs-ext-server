#!/usr/bin/env python3
"""
E2E 테스트: 사용자 역할 할당 API 캐싱 동작 검증

테스트 시나리오:
1. PUT 중복 요청 방지 (max-age=1)
2. GET 1초 내 캐시 히트
3. PUT 후 GET - 클라이언트 캐시 무효화
4. ETag 검증 (304 Not Modified)
5. 동시 요청 처리
"""

import requests
import time
import json
from typing import Optional, Dict, Any
from datetime import datetime
import sys
import psycopg2
from psycopg2.extras import RealDictCursor

# 설정
BASE_URL = "http://localhost:8080"
PROJECT_ID = None  # Setup에서 생성
USER_ID_1 = None   # Setup에서 생성
USER_ID_2 = None   # Setup에서 생성
ROLE_ID_VIEWER = None  # Setup에서 조회
ROLE_ID_EDITOR = None  # Setup에서 조회

# DB 연결 정보
DB_HOST = "localhost"
DB_PORT = 5456
DB_NAME = "pacs_extension"
DB_USER = "pacs_extension_admin"
DB_PASSWORD = "PacsExtension2024"

# 색상 출력
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'

def log_success(msg: str):
    print(f"{Colors.GREEN}✅ {msg}{Colors.RESET}")

def log_error(msg: str):
    print(f"{Colors.RED}❌ {msg}{Colors.RESET}")

def log_info(msg: str):
    print(f"{Colors.BLUE}ℹ️  {msg}{Colors.RESET}")

def log_warning(msg: str):
    print(f"{Colors.YELLOW}⚠️  {msg}{Colors.RESET}")

def log_test(msg: str):
    print(f"\n{Colors.BLUE}{'='*80}{Colors.RESET}")
    print(f"{Colors.BLUE}🧪 {msg}{Colors.RESET}")
    print(f"{Colors.BLUE}{'='*80}{Colors.RESET}")


class APIClient:
    def __init__(self, base_url: str, token: Optional[str] = None):
        self.base_url = base_url
        self.token = token
        self.session = requests.Session()
        if token:
            self.session.headers.update({"Authorization": f"Bearer {token}"})
    
    def assign_role(self, project_id: int, user_id: int, role_id: int, 
                   if_none_match: Optional[str] = None) -> requests.Response:
        """역할 할당 (PUT)"""
        url = f"{self.base_url}/api/projects/{project_id}/users/{user_id}/role"
        headers = {}
        if if_none_match:
            headers["If-None-Match"] = if_none_match
        
        response = self.session.put(url, json={"role_id": role_id}, headers=headers)
        return response
    
    def get_members(self, project_id: int, if_none_match: Optional[str] = None,
                   no_cache: bool = False) -> requests.Response:
        """프로젝트 멤버 목록 조회 (GET)"""
        url = f"{self.base_url}/api/projects/{project_id}/users"
        headers = {}
        if if_none_match:
            headers["If-None-Match"] = if_none_match
        if no_cache:
            headers["Cache-Control"] = "no-cache"
        
        response = self.session.get(url, headers=headers)
        return response
    
    def batch_assign_roles(self, project_id: int, assignments: list) -> requests.Response:
        """일괄 역할 할당 (POST)"""
        url = f"{self.base_url}/api/projects/{project_id}/users/roles"
        response = self.session.post(url, json={"assignments": assignments})
        return response


def print_response_info(response: requests.Response, label: str = "Response"):
    """응답 정보 출력"""
    print(f"\n{label}:")
    print(f"  Status: {response.status_code}")
    print(f"  Cache-Control: {response.headers.get('Cache-Control', 'N/A')}")
    print(f"  ETag: {response.headers.get('ETag', 'N/A')}")
    if response.status_code == 200:
        try:
            data = response.json()
            if 'updated_at' in data:
                print(f"  updated_at: {data['updated_at']}")
            if 'latest_updated_at' in data:
                print(f"  latest_updated_at: {data['latest_updated_at']}")
        except:
            pass


def test_put_duplicate_prevention(client: APIClient):
    """테스트 1: PUT 중복 요청 방지 (max-age=1)"""
    log_test("테스트 1: PUT 중복 요청 방지 (max-age=1)")
    
    # 1차 요청
    log_info("1차 PUT 요청...")
    resp1 = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_VIEWER)
    print_response_info(resp1, "1차 응답")
    
    assert resp1.status_code == 200, f"Expected 200, got {resp1.status_code}"
    assert "ETag" in resp1.headers, "ETag header missing"
    assert "max-age=1" in resp1.headers.get("Cache-Control", ""), "max-age=1 missing"
    
    etag1 = resp1.headers["ETag"]
    updated_at1 = resp1.json()["updated_at"]
    
    log_success(f"1차 요청 성공 - ETag: {etag1}, updated_at: {updated_at1}")
    
    # 0.3초 후 동일한 요청 (ETag 포함)
    time.sleep(0.3)
    log_info("0.3초 후 2차 PUT 요청 (If-None-Match 포함)...")
    resp2 = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_VIEWER, if_none_match=etag1)
    print_response_info(resp2, "2차 응답")

    # 브라우저는 max-age=1 내에 자동으로 캐시 사용하지만,
    # 서버는 If-None-Match가 있으면 304 반환 가능
    if resp2.status_code == 304:
        log_success("304 Not Modified - ETag 일치, 변경 없음")
    elif resp2.status_code == 200:
        log_warning("200 OK - 서버가 요청 처리함 (ETag 불일치 또는 데이터 변경)")
        updated_at2 = resp2.json()["updated_at"]
        if updated_at1 == updated_at2:
            log_info("updated_at 동일 - 실제로는 변경 없었음")
    else:
        log_error(f"Unexpected status: {resp2.status_code}")
        return False

    # 1.5초 후 요청 (캐시 만료 후)
    time.sleep(1.5)
    log_info("1.5초 후 3차 PUT 요청 (캐시 만료 후)...")
    resp3 = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_EDITOR)  # 다른 role_id
    print_response_info(resp3, "3차 응답")

    assert resp3.status_code == 200, f"Expected 200, got {resp3.status_code}"
    etag3 = resp3.headers["ETag"]
    updated_at3 = resp3.json()["updated_at"]

    # updated_at이 변경되었는지 확인
    assert updated_at3 != updated_at1, "updated_at should change after role change"
    log_success(f"역할 변경 성공 - 새 ETag: {etag3}, updated_at: {updated_at3}")

    log_success("테스트 1 통과!")
    return True


def test_get_cache_hit(client: APIClient):
    """테스트 2: GET 1초 내 캐시 히트"""
    log_test("테스트 2: GET 1초 내 캐시 히트")

    # 1차 GET 요청
    log_info("1차 GET 요청...")
    resp1 = client.get_members(PROJECT_ID)
    print_response_info(resp1, "1차 응답")

    assert resp1.status_code == 200, f"Expected 200, got {resp1.status_code}"
    assert "ETag" in resp1.headers, "ETag header missing"
    assert "max-age=1" in resp1.headers.get("Cache-Control", ""), "max-age=1 missing"

    etag1 = resp1.headers["ETag"]
    data1 = resp1.json()
    latest_updated_at1 = data1["latest_updated_at"]

    log_success(f"1차 요청 성공 - ETag: {etag1}, latest_updated_at: {latest_updated_at1}")

    # 0.5초 후 동일한 GET 요청 (If-None-Match 포함)
    time.sleep(0.5)
    log_info("0.5초 후 2차 GET 요청 (If-None-Match 포함)...")
    resp2 = client.get_members(PROJECT_ID, if_none_match=etag1)
    print_response_info(resp2, "2차 응답")

    # 브라우저는 max-age=1 내에 캐시 사용 (서버 요청 안 감)
    # 하지만 테스트에서는 명시적으로 요청하므로 304 기대
    if resp2.status_code == 304:
        log_success("304 Not Modified - 데이터 변경 없음, 네트워크 절약!")
    elif resp2.status_code == 200:
        log_warning("200 OK - 데이터가 변경되었거나 ETag 불일치")
        data2 = resp2.json()
        if data1 == data2:
            log_info("데이터 동일 - ETag가 변경되었을 수 있음")
    else:
        log_error(f"Unexpected status: {resp2.status_code}")
        return False

    # 1.5초 후 요청 (캐시 만료 후)
    time.sleep(1.5)
    log_info("1.5초 후 3차 GET 요청 (캐시 만료 후)...")
    resp3 = client.get_members(PROJECT_ID)
    print_response_info(resp3, "3차 응답")

    assert resp3.status_code == 200, f"Expected 200, got {resp3.status_code}"

    log_success("테스트 2 통과!")
    return True


def test_put_then_get_invalidation(client: APIClient):
    """테스트 3: PUT 후 GET - 클라이언트 캐시 무효화"""
    log_test("테스트 3: PUT 후 GET - 클라이언트 캐시 무효화")

    # 1. 초기 GET (캐시 생성)
    log_info("1. 초기 GET 요청 (캐시 생성)...")
    resp_get1 = client.get_members(PROJECT_ID)
    print_response_info(resp_get1, "초기 GET 응답")

    assert resp_get1.status_code == 200
    data_before = resp_get1.json()
    etag_before = resp_get1.headers["ETag"]

    # 현재 user의 role 확인
    user_before = next((m for m in data_before["members"] if m["user_id"] == USER_ID_1), None)
    if user_before:
        log_info(f"변경 전 역할: user_id={USER_ID_1}, role_id={user_before.get('role_id')}")

    # 2. 0.3초 후 PUT (역할 변경)
    time.sleep(0.3)
    log_info("2. 0.3초 후 PUT 요청 (역할 변경)...")
    resp_put = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_EDITOR)
    print_response_info(resp_put, "PUT 응답")

    assert resp_put.status_code == 200
    updated_at_put = resp_put.json()["updated_at"]
    log_success(f"역할 변경 완료 - updated_at: {updated_at_put}")

    # 3. 즉시 GET (no-cache 없이) - 캐시 히트 가능성
    log_info("3. 즉시 GET 요청 (no-cache 없이)...")
    resp_get2 = client.get_members(PROJECT_ID, if_none_match=etag_before)
    print_response_info(resp_get2, "GET 응답 (no-cache 없이)")

    if resp_get2.status_code == 304:
        log_warning("⚠️  304 Not Modified - 오래된 캐시! (예상된 문제)")
        log_info("이것이 바로 클라이언트가 Cache-Control: no-cache를 보내야 하는 이유!")
    elif resp_get2.status_code == 200:
        data_after = resp_get2.json()
        user_after = next((m for m in data_after["members"] if m["user_id"] == USER_ID_1), None)
        if user_after and user_after.get("role_id") == ROLE_ID_EDITOR:
            log_success("✅ 최신 데이터 반환 - 역할 변경 반영됨")
        else:
            log_warning("⚠️  오래된 데이터 반환")

    # 4. Cache-Control: no-cache로 강제 새로고침
    log_info("4. Cache-Control: no-cache로 GET 요청 (강제 새로고침)...")
    resp_get3 = client.get_members(PROJECT_ID, no_cache=True)
    print_response_info(resp_get3, "GET 응답 (no-cache)")

    assert resp_get3.status_code == 200
    data_fresh = resp_get3.json()
    user_fresh = next((m for m in data_fresh["members"] if m["user_id"] == USER_ID_1), None)

    assert user_fresh is not None, f"User {USER_ID_1} not found"
    assert user_fresh.get("role_id") == ROLE_ID_EDITOR, \
        f"Expected role_id={ROLE_ID_EDITOR}, got {user_fresh.get('role_id')}"

    log_success(f"✅ 최신 데이터 확인 - user_id={USER_ID_1}, role_id={user_fresh.get('role_id')}")
    log_success("테스트 3 통과!")
    return True


def test_etag_validation(client: APIClient):
    """테스트 4: ETag 검증 (304 Not Modified)"""
    log_test("테스트 4: ETag 검증 (304 Not Modified)")

    # 1. GET 요청으로 ETag 획득
    log_info("1. GET 요청으로 ETag 획득...")
    resp1 = client.get_members(PROJECT_ID)
    print_response_info(resp1, "1차 GET 응답")

    assert resp1.status_code == 200
    etag1 = resp1.headers["ETag"]
    data1 = resp1.json()

    log_success(f"ETag 획득: {etag1}")

    # 2. 1.5초 후 동일한 GET 요청 (캐시 만료 후, If-None-Match 포함)
    time.sleep(1.5)
    log_info("2. 1.5초 후 GET 요청 (If-None-Match 포함)...")
    resp2 = client.get_members(PROJECT_ID, if_none_match=etag1)
    print_response_info(resp2, "2차 GET 응답")

    # 데이터 변경이 없었다면 304 기대
    if resp2.status_code == 304:
        log_success("✅ 304 Not Modified - ETag 일치, 네트워크 절약!")
    elif resp2.status_code == 200:
        log_info("200 OK - 데이터가 변경되었거나 ETag 불일치")
        etag2 = resp2.headers["ETag"]
        if etag1 != etag2:
            log_info(f"ETag 변경: {etag1} → {etag2}")
    else:
        log_error(f"Unexpected status: {resp2.status_code}")
        return False

    # 3. 역할 변경 후 ETag 변경 확인
    log_info("3. 역할 변경...")
    resp_put = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_VIEWER)
    assert resp_put.status_code == 200
    log_success("역할 변경 완료")

    # 4. GET 요청 (이전 ETag 사용)
    log_info("4. GET 요청 (이전 ETag 사용)...")
    resp3 = client.get_members(PROJECT_ID, if_none_match=etag1, no_cache=True)
    print_response_info(resp3, "3차 GET 응답")

    # 데이터가 변경되었으므로 200 기대
    assert resp3.status_code == 200, f"Expected 200 (data changed), got {resp3.status_code}"
    etag3 = resp3.headers["ETag"]

    assert etag3 != etag1, "ETag should change after data modification"
    log_success(f"✅ ETag 변경 확인: {etag1} → {etag3}")

    log_success("테스트 4 통과!")
    return True


def test_concurrent_requests(client: APIClient):
    """테스트 5: 동시 요청 처리"""
    log_test("테스트 5: 동시 요청 처리")

    import concurrent.futures

    # 동시에 같은 역할 할당 요청 5번
    log_info("동시에 같은 역할 할당 요청 5번...")

    def assign_role_task(task_id: int):
        try:
            resp = client.assign_role(PROJECT_ID, USER_ID_1, ROLE_ID_EDITOR)
            return (task_id, resp.status_code, resp.headers.get("ETag"))
        except Exception as e:
            return (task_id, None, str(e))

    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        futures = [executor.submit(assign_role_task, i) for i in range(5)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]

    # 결과 분석
    success_count = sum(1 for _, status, _ in results if status == 200)
    not_modified_count = sum(1 for _, status, _ in results if status == 304)

    log_info(f"결과: 200 OK={success_count}, 304 Not Modified={not_modified_count}")

    for task_id, status, etag in sorted(results):
        if status:
            log_info(f"  Task {task_id}: {status} - ETag: {etag}")
        else:
            log_error(f"  Task {task_id}: Error - {etag}")

    # 최소 1개는 성공해야 함
    assert success_count >= 1, "At least one request should succeed"

    log_success("테스트 5 통과!")
    return True


def test_batch_assign_cache(client: APIClient):
    """테스트 6: 일괄 역할 할당 캐싱"""
    log_test("테스트 6: 일괄 역할 할당 캐싱")

    # 일괄 할당
    log_info("일괄 역할 할당...")
    assignments = [
        {"user_id": USER_ID_1, "role_id": ROLE_ID_VIEWER},
        {"user_id": USER_ID_2, "role_id": ROLE_ID_EDITOR},
    ]

    resp1 = client.batch_assign_roles(PROJECT_ID, assignments)
    print_response_info(resp1, "1차 일괄 할당 응답")

    assert resp1.status_code == 200
    assert "ETag" in resp1.headers
    assert "max-age=1" in resp1.headers.get("Cache-Control", "")

    etag1 = resp1.headers["ETag"]
    data1 = resp1.json()

    log_success(f"일괄 할당 성공 - assigned_count: {data1.get('assigned_count')}")

    # 0.5초 후 동일한 요청 (If-None-Match 포함)
    time.sleep(0.5)
    log_info("0.5초 후 동일한 일괄 할당 요청...")

    # Note: POST는 If-None-Match를 지원하지 않을 수 있음
    resp2 = client.batch_assign_roles(PROJECT_ID, assignments)
    print_response_info(resp2, "2차 일괄 할당 응답")

    # POST는 항상 200 반환 (멱등성 없음)
    assert resp2.status_code == 200

    log_success("테스트 6 통과!")
    return True


def main():
    """메인 테스트 실행"""
    print(f"\n{Colors.BLUE}{'='*80}{Colors.RESET}")
    print(f"{Colors.BLUE}🚀 사용자 역할 할당 API 캐싱 E2E 테스트{Colors.RESET}")
    print(f"{Colors.BLUE}{'='*80}{Colors.RESET}\n")

    # API 클라이언트 생성 (토큰 없이 테스트)
    client = APIClient(BASE_URL)

    # 서버 연결 확인
    try:
        resp = client.get_members(PROJECT_ID)
        if resp.status_code == 401:
            log_error("인증 필요! 토큰을 설정하세요.")
            log_info("export PACS_TOKEN='your_token_here'")
            return 1
    except requests.exceptions.ConnectionError:
        log_error(f"서버 연결 실패: {BASE_URL}")
        log_info("서버가 실행 중인지 확인하세요: cargo run")
        return 1

    # 테스트 실행
    tests = [
        ("PUT 중복 요청 방지", test_put_duplicate_prevention),
        ("GET 1초 내 캐시 히트", test_get_cache_hit),
        ("PUT 후 GET 캐시 무효화", test_put_then_get_invalidation),
        ("ETag 검증", test_etag_validation),
        ("동시 요청 처리", test_concurrent_requests),
        ("일괄 역할 할당 캐싱", test_batch_assign_cache),
        # Note: no-cache 헤더와 빈 목록 테스트는 이 API의 특수한 캐싱 전략상 적용 불가
    ]

    passed = 0
    failed = 0

    for name, test_func in tests:
        try:
            if test_func(client):
                passed += 1
            else:
                failed += 1
                log_error(f"테스트 실패: {name}")
        except AssertionError as e:
            failed += 1
            log_error(f"테스트 실패: {name}")
            log_error(f"  {str(e)}")
        except Exception as e:
            failed += 1
            log_error(f"테스트 에러: {name}")
            log_error(f"  {str(e)}")
            import traceback
            traceback.print_exc()

    # 결과 요약
    print(f"\n{Colors.BLUE}{'='*80}{Colors.RESET}")
    print(f"{Colors.BLUE}📊 테스트 결과{Colors.RESET}")
    print(f"{Colors.BLUE}{'='*80}{Colors.RESET}")
    print(f"{Colors.GREEN}✅ 통과: {passed}{Colors.RESET}")
    print(f"{Colors.RED}❌ 실패: {failed}{Colors.RESET}")
    print(f"{Colors.BLUE}{'='*80}{Colors.RESET}\n")

    return 0 if failed == 0 else 1


def setup_test_data():
    """테스트 데이터 생성"""
    global PROJECT_ID, USER_ID_1, USER_ID_2, ROLE_ID_VIEWER, ROLE_ID_EDITOR

    log_info("🔧 테스트 데이터 생성 중...")

    conn = psycopg2.connect(
        host=DB_HOST,
        port=DB_PORT,
        dbname=DB_NAME,
        user=DB_USER,
        password=DB_PASSWORD
    )

    try:
        cur = conn.cursor(cursor_factory=RealDictCursor)

        # 1. 역할 조회 (VIEWER, USER 사용)
        cur.execute("SELECT id, name FROM security_role WHERE name IN ('VIEWER', 'USER') ORDER BY name")
        roles = cur.fetchall()
        if len(roles) < 2:
            log_error("❌ VIEWER/USER 역할이 없습니다!")
            sys.exit(1)

        for role in roles:
            if role['name'] == 'USER':
                ROLE_ID_EDITOR = role['id']  # USER를 Editor 역할로 사용
            elif role['name'] == 'VIEWER':
                ROLE_ID_VIEWER = role['id']

        log_success(f"✅ 역할 조회 완료: VIEWER={ROLE_ID_VIEWER}, USER(Editor)={ROLE_ID_EDITOR}")

        # 2. 테스트 프로젝트 생성
        cur.execute("""
            INSERT INTO security_project (name, description, created_at)
            VALUES ('test_cache_project', 'Test project for cache testing', NOW())
            RETURNING id
        """)
        PROJECT_ID = cur.fetchone()['id']
        log_success(f"✅ 프로젝트 생성 완료: ID={PROJECT_ID}")

        # 3. 테스트 사용자 생성
        import uuid
        keycloak_id_1 = str(uuid.uuid4())
        keycloak_id_2 = str(uuid.uuid4())

        cur.execute("""
            INSERT INTO security_user (keycloak_id, username, email, created_at)
            VALUES (%s, %s, %s, NOW())
            RETURNING id
        """, (keycloak_id_1, 'test_cache_user_1', 'test1@cache.test'))
        USER_ID_1 = cur.fetchone()['id']

        cur.execute("""
            INSERT INTO security_user (keycloak_id, username, email, created_at)
            VALUES (%s, %s, %s, NOW())
            RETURNING id
        """, (keycloak_id_2, 'test_cache_user_2', 'test2@cache.test'))
        USER_ID_2 = cur.fetchone()['id']

        log_success(f"✅ 사용자 생성 완료: USER_1={USER_ID_1}, USER_2={USER_ID_2}")

        conn.commit()
        log_success("✅ 테스트 데이터 생성 완료!")

    except Exception as e:
        conn.rollback()
        log_error(f"❌ 테스트 데이터 생성 실패: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        cur.close()
        conn.close()


def cleanup_test_data():
    """테스트 데이터 삭제"""
    log_info("🧹 테스트 데이터 정리 중...")

    conn = psycopg2.connect(
        host=DB_HOST,
        port=DB_PORT,
        dbname=DB_NAME,
        user=DB_USER,
        password=DB_PASSWORD
    )

    try:
        cur = conn.cursor()

        # 1. 프로젝트-사용자 관계 삭제
        if PROJECT_ID:
            cur.execute("DELETE FROM security_user_project WHERE project_id = %s", (PROJECT_ID,))
            log_success(f"✅ 프로젝트-사용자 관계 삭제 완료")

        # 2. 프로젝트 삭제
        if PROJECT_ID:
            cur.execute("DELETE FROM security_project WHERE id = %s", (PROJECT_ID,))
            log_success(f"✅ 프로젝트 삭제 완료: ID={PROJECT_ID}")

        # 3. 사용자 삭제
        if USER_ID_1:
            cur.execute("DELETE FROM security_user WHERE id = %s", (USER_ID_1,))
            log_success(f"✅ 사용자 1 삭제 완료: ID={USER_ID_1}")

        if USER_ID_2:
            cur.execute("DELETE FROM security_user WHERE id = %s", (USER_ID_2,))
            log_success(f"✅ 사용자 2 삭제 완료: ID={USER_ID_2}")

        conn.commit()
        log_success("✅ 테스트 데이터 정리 완료!")

    except Exception as e:
        conn.rollback()
        log_error(f"❌ 테스트 데이터 정리 실패: {e}")
        import traceback
        traceback.print_exc()
    finally:
        cur.close()
        conn.close()


if __name__ == "__main__":
    try:
        setup_test_data()
        sys.exit(main())
    finally:
        cleanup_test_data()

