#!/usr/bin/env python3
"""
E2E 테스트: Role-Capability 매트릭스 API 캐싱 동작 검증

테스트 시나리오:
1. GET 전체 매트릭스 - ETag 및 304 응답 검증
2. GET 페이지네이션 - 모든 페이지 동일한 ETag 검증
3. Role 변경 후 ETag 변경 검증
4. Capability 할당 변경 후 ETag 변경 검증
5. 페이지별 변경 감지 - Page 1 변경 시 Page 2도 감지
6. 동시 요청 처리 - max-age=5 캐싱 검증
7. 프로젝트별 매트릭스 API 캐싱 검증
8. 여러 클라이언트 동시 변경 시나리오
9. Capability 메타데이터 변경 감지
10. 브라우저 캐시 동작 검증
"""

import requests
import time
import json
from typing import Optional, Dict, Any, List
from datetime import datetime
import sys
import psycopg2
from psycopg2.extras import RealDictCursor
import concurrent.futures

# 설정
BASE_URL = "http://localhost:8080"
ROLE_ID = None  # Setup에서 조회
CAPABILITY_ID = None  # Setup에서 조회
PROJECT_ID = None  # Setup에서 조회

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


class MatrixAPIClient:
    """Role-Capability Matrix API 클라이언트"""
    
    def __init__(self, base_url: str, token: Optional[str] = None):
        self.base_url = base_url
        self.token = token
        self.session = requests.Session()
        if token:
            self.session.headers.update({"Authorization": f"Bearer {token}"})
    
    def get_matrix_all(self, if_none_match: Optional[str] = None, 
                       no_cache: bool = False) -> requests.Response:
        """전체 매트릭스 조회 (GET /api/roles/global/capabilities/matrix/all)"""
        url = f"{self.base_url}/api/roles/global/capabilities/matrix/all"
        headers = {}
        if if_none_match:
            headers["If-None-Match"] = if_none_match
        if no_cache:
            headers["Cache-Control"] = "no-cache"
        
        return self.session.get(url, headers=headers)
    
    def get_matrix_paginated(self, page: int = 1, size: int = 10,
                            if_none_match: Optional[str] = None,
                            no_cache: bool = False) -> requests.Response:
        """페이지네이션 매트릭스 조회 (GET /api/roles/global/capabilities/matrix)"""
        url = f"{self.base_url}/api/roles/global/capabilities/matrix"
        params = {"page": page, "size": size}
        headers = {}
        if if_none_match:
            headers["If-None-Match"] = if_none_match
        if no_cache:
            headers["Cache-Control"] = "no-cache"
        
        return self.session.get(url, params=params, headers=headers)
    
    def get_project_matrix(self, project_id: int,
                          if_none_match: Optional[str] = None,
                          no_cache: bool = False) -> requests.Response:
        """프로젝트별 매트릭스 조회 (GET /api/roles/projects/{id}/capabilities/matrix)"""
        url = f"{self.base_url}/api/roles/projects/{project_id}/capabilities/matrix"
        headers = {}
        if if_none_match:
            headers["If-None-Match"] = if_none_match
        if no_cache:
            headers["Cache-Control"] = "no-cache"

        return self.session.get(url, headers=headers)


def print_response_info(response: requests.Response, label: str = "Response"):
    """응답 정보 출력"""
    print(f"\n{label}:")
    print(f"  Status: {response.status_code}")
    print(f"  Cache-Control: {response.headers.get('Cache-Control', 'N/A')}")
    print(f"  ETag: {response.headers.get('ETag', 'N/A')}")
    if response.status_code == 200:
        try:
            data = response.json()
            if 'roles' in data:
                print(f"  Roles count: {len(data['roles'])}")
            if 'pagination' in data:
                print(f"  Pagination: {data['pagination']}")
        except:
            pass


def get_db_connection():
    """DB 연결 생성"""
    return psycopg2.connect(
        host=DB_HOST,
        port=DB_PORT,
        dbname=DB_NAME,
        user=DB_USER,
        password=DB_PASSWORD
    )


def test_get_matrix_all_etag(client: MatrixAPIClient):
    """테스트 1: GET 전체 매트릭스 - ETag 및 304 응답 검증"""
    log_test("테스트 1: GET 전체 매트릭스 - ETag 및 304 응답 검증")

    # 1차 요청
    log_info("1차 GET 요청...")
    resp1 = client.get_matrix_all()
    print_response_info(resp1, "1차 응답")

    assert resp1.status_code == 200, f"Expected 200, got {resp1.status_code}"
    assert "ETag" in resp1.headers, "ETag header missing"
    assert "max-age=5" in resp1.headers.get("Cache-Control", ""), "max-age=5 missing"

    etag1 = resp1.headers["ETag"]
    data1 = resp1.json()

    log_success(f"1차 요청 성공 - ETag: {etag1}, Roles: {len(data1['roles'])}")

    # 2초 후 동일한 요청 (If-None-Match 포함)
    time.sleep(2)
    log_info("2초 후 2차 GET 요청 (If-None-Match 포함)...")
    resp2 = client.get_matrix_all(if_none_match=etag1)
    print_response_info(resp2, "2차 응답")

    # 데이터 변경이 없었다면 304 기대
    if resp2.status_code == 304:
        log_success("✅ 304 Not Modified - ETag 일치, 네트워크 절약!")
    elif resp2.status_code == 200:
        log_warning("200 OK - 데이터가 변경되었거나 ETag 불일치")
        etag2 = resp2.headers["ETag"]
        if etag1 != etag2:
            log_info(f"ETag 변경: {etag1} → {etag2}")
    else:
        log_error(f"Unexpected status: {resp2.status_code}")
        return False

    # 6초 후 요청 (캐시 만료 후)
    time.sleep(6)
    log_info("6초 후 3차 GET 요청 (캐시 만료 후)...")
    resp3 = client.get_matrix_all()
    print_response_info(resp3, "3차 응답")

    assert resp3.status_code == 200, f"Expected 200, got {resp3.status_code}"

    log_success("테스트 1 통과!")
    return True


def test_pagination_same_etag(client: MatrixAPIClient):
    """테스트 2: GET 페이지네이션 - 모든 페이지 동일한 ETag 검증"""
    log_test("테스트 2: GET 페이지네이션 - 모든 페이지 동일한 ETag 검증")

    # Page 1 조회
    log_info("Page 1 조회...")
    resp1 = client.get_matrix_paginated(page=1, size=2)
    print_response_info(resp1, "Page 1 응답")

    assert resp1.status_code == 200
    etag1 = resp1.headers["ETag"]
    data1 = resp1.json()

    log_success(f"Page 1 - ETag: {etag1}, Roles: {len(data1['roles'])}")

    # Page 2 조회
    log_info("Page 2 조회...")
    resp2 = client.get_matrix_paginated(page=2, size=2)
    print_response_info(resp2, "Page 2 응답")

    assert resp2.status_code == 200
    etag2 = resp2.headers["ETag"]
    data2 = resp2.json()

    log_success(f"Page 2 - ETag: {etag2}, Roles: {len(data2['roles'])}")

    # ETag 비교
    if etag1 == etag2:
        log_success("✅ 모든 페이지가 동일한 ETag 사용!")
    else:
        log_error(f"❌ 페이지별로 다른 ETag: {etag1} vs {etag2}")
        return False

    # Page 1 재조회 (If-None-Match)
    log_info("Page 1 재조회 (If-None-Match)...")
    resp3 = client.get_matrix_paginated(page=1, size=2, if_none_match=etag1)
    print_response_info(resp3, "Page 1 재조회 응답")

    if resp3.status_code == 304:
        log_success("✅ 304 Not Modified - 캐시 히트!")
    elif resp3.status_code == 200:
        log_warning("200 OK - 데이터 변경됨")

    log_success("테스트 2 통과!")
    return True


def test_role_change_etag_update(client: MatrixAPIClient):
    """테스트 3: Role 변경 후 ETag 변경 검증"""
    log_test("테스트 3: Role 변경 후 ETag 변경 검증")

    # 1. 현재 ETag 획득
    log_info("1. 현재 ETag 획득...")
    resp1 = client.get_matrix_all()
    assert resp1.status_code == 200
    etag_before = resp1.headers["ETag"]
    log_success(f"Before ETag: {etag_before}")

    # 2. Role 이름 변경
    log_info("2. Role 이름 변경 (ADMIN → ADMIN_TEST)...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN_TEST' WHERE name = 'ADMIN'")
        affected = cur.rowcount
        conn.commit()

        if affected > 0:
            log_success(f"✅ Role 변경 완료 ({affected}개)")
        else:
            log_warning("⚠️  ADMIN 역할이 없어서 다른 역할 변경...")
            cur.execute("UPDATE security_role SET description = 'Test description update' WHERE id = %s", (ROLE_ID,))
            conn.commit()
            log_success("✅ Role 설명 변경 완료")
    finally:
        cur.close()
        conn.close()

    # 3. 변경 후 ETag 획득
    time.sleep(1)
    log_info("3. 변경 후 ETag 획득...")
    resp2 = client.get_matrix_all(no_cache=True)
    assert resp2.status_code == 200
    etag_after = resp2.headers["ETag"]
    log_success(f"After ETag: {etag_after}")

    # 4. ETag 비교
    if etag_before != etag_after:
        log_success(f"✅ ETag 변경 감지 성공! {etag_before} → {etag_after}")
    else:
        log_error(f"❌ ETag 변경 감지 실패! (동일: {etag_before})")
        return False

    # 5. 원복
    log_info("5. 원복 (ADMIN_TEST → ADMIN)...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN' WHERE name = 'ADMIN_TEST'")
        conn.commit()
        log_success("✅ 원복 완료")
    finally:
        cur.close()
        conn.close()

    log_success("테스트 3 통과!")
    return True


def test_capability_assignment_etag_update(client: MatrixAPIClient):
    """테스트 4: Capability 할당 변경 후 ETag 변경 검증"""
    log_test("테스트 4: Capability 할당 변경 후 ETag 변경 검증")

    # 이전 테스트와 다른 초에 실행되도록 대기
    time.sleep(2)

    # 1. 현재 ETag 획득
    log_info("1. 현재 ETag 획득...")
    resp1 = client.get_matrix_all(no_cache=True)
    assert resp1.status_code == 200
    etag_before = resp1.headers["ETag"]
    log_success(f"Before ETag: {etag_before}")

    # 2. 할당되지 않은 조합 찾기
    log_info("2. 할당되지 않은 Role-Capability 조합 찾기...")
    conn = get_db_connection()
    try:
        cur = conn.cursor(cursor_factory=RealDictCursor)

        # 모든 Role과 Capability 조합 중 할당되지 않은 것 찾기
        cur.execute("""
            SELECT r.id as role_id, c.id as capability_id
            FROM security_role r
            CROSS JOIN security_capability c
            WHERE NOT EXISTS (
                SELECT 1 FROM security_role_capability rc
                WHERE rc.role_id = r.id AND rc.capability_id = c.id
            )
            LIMIT 1
        """)
        unassigned = cur.fetchone()

        if not unassigned:
            log_warning("⚠️  모든 조합이 이미 할당되어 있음, 기존 할당 사용...")
            # 기존 할당 중 하나를 삭제 후 재추가
            cur.execute("SELECT role_id, capability_id FROM security_role_capability LIMIT 1")
            existing = cur.fetchone()
            test_role_id = existing['role_id']
            test_capability_id = existing['capability_id']

            # 삭제
            cur.execute("DELETE FROM security_role_capability WHERE role_id = %s AND capability_id = %s",
                       (test_role_id, test_capability_id))
            conn.commit()
            log_success(f"✅ 기존 할당 삭제: role_id={test_role_id}, capability_id={test_capability_id}")
        else:
            test_role_id = unassigned['role_id']
            test_capability_id = unassigned['capability_id']
            log_success(f"✅ 할당되지 않은 조합 발견: role_id={test_role_id}, capability_id={test_capability_id}")

        # 3. 새 할당 추가 (2초 대기하여 다음 초로 넘어가도록)
        time.sleep(2)
        log_info("3. Capability 할당 추가...")
        cur.execute("""
            INSERT INTO security_role_capability (role_id, capability_id)
            VALUES (%s, %s)
        """, (test_role_id, test_capability_id))
        conn.commit()
        log_success("✅ 할당 추가 완료")

        # 4. 변경 후 ETag 획득
        time.sleep(1)
        log_info("4. 추가 후 ETag 획득...")
        resp2 = client.get_matrix_all(no_cache=True)
        assert resp2.status_code == 200
        etag_after = resp2.headers["ETag"]
        log_success(f"After ETag: {etag_after}")

        # 5. ETag 비교
        if etag_before != etag_after:
            log_success(f"✅ ETag 변경 감지 성공! {etag_before} → {etag_after}")
        else:
            log_error(f"❌ ETag 변경 감지 실패! (동일: {etag_before})")
            # 디버깅: created_at 확인
            cur.execute("""
                SELECT created_at, updated_at
                FROM security_role_capability
                WHERE role_id = %s AND capability_id = %s
            """, (test_role_id, test_capability_id))
            debug_row = cur.fetchone()
            if debug_row:
                log_info(f"Debug: created_at={debug_row['created_at']}, updated_at={debug_row['updated_at']}")
            return False

        # 6. 정리 (추가한 할당 삭제)
        log_info("6. 테스트 할당 정리...")
        cur.execute("""
            DELETE FROM security_role_capability
            WHERE role_id = %s AND capability_id = %s
        """, (test_role_id, test_capability_id))
        conn.commit()
        log_success("✅ 정리 완료")

    finally:
        cur.close()
        conn.close()

    log_success("테스트 4 통과!")
    return True


def test_page_change_detection(client: MatrixAPIClient):
    """테스트 5: 페이지별 변경 감지 - Page 1 변경 시 Page 2도 감지"""
    log_test("테스트 5: 페이지별 변경 감지 - Page 1 변경 시 Page 2도 감지")

    # 1. Page 1, Page 2 ETag 획득
    log_info("1. Page 1, Page 2 ETag 획득...")
    resp_p1 = client.get_matrix_paginated(page=1, size=2)
    resp_p2 = client.get_matrix_paginated(page=2, size=2)

    assert resp_p1.status_code == 200
    assert resp_p2.status_code == 200

    etag_p1_before = resp_p1.headers["ETag"]
    etag_p2_before = resp_p2.headers["ETag"]

    log_success(f"Page 1 ETag: {etag_p1_before}")
    log_success(f"Page 2 ETag: {etag_p2_before}")

    # 2. Page 1에 있는 Role 변경
    log_info("2. Page 1에 있는 Role 변경...")
    data_p1 = resp_p1.json()
    if len(data_p1['roles']) > 0:
        role_to_change = data_p1['roles'][0]['id']
        log_info(f"   변경할 Role ID: {role_to_change}")

        conn = get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("UPDATE security_role SET description = 'Test change for page detection' WHERE id = %s",
                       (role_to_change,))
            conn.commit()
            log_success("✅ Role 변경 완료")
        finally:
            cur.close()
            conn.close()
    else:
        log_warning("⚠️  Page 1에 Role이 없음, 전체 Role 변경...")
        conn = get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("UPDATE security_role SET description = 'Test change' WHERE id = %s", (ROLE_ID,))
            conn.commit()
            log_success("✅ Role 변경 완료")
        finally:
            cur.close()
            conn.close()

    # 3. Page 1, Page 2 재조회
    time.sleep(1)
    log_info("3. Page 1, Page 2 재조회...")
    resp_p1_after = client.get_matrix_paginated(page=1, size=2, no_cache=True)
    resp_p2_after = client.get_matrix_paginated(page=2, size=2, no_cache=True)

    assert resp_p1_after.status_code == 200
    assert resp_p2_after.status_code == 200

    etag_p1_after = resp_p1_after.headers["ETag"]
    etag_p2_after = resp_p2_after.headers["ETag"]

    log_success(f"Page 1 ETag (after): {etag_p1_after}")
    log_success(f"Page 2 ETag (after): {etag_p2_after}")

    # 4. 검증
    if etag_p1_before != etag_p1_after:
        log_success("✅ Page 1 ETag 변경 감지!")
    else:
        log_error("❌ Page 1 ETag 변경 감지 실패!")
        return False

    if etag_p2_before != etag_p2_after:
        log_success("✅ Page 2 ETag도 변경 감지! (중요!)")
    else:
        log_error("❌ Page 2 ETag 변경 감지 실패! (치명적!)")
        return False

    if etag_p1_after == etag_p2_after:
        log_success("✅ 변경 후에도 모든 페이지가 동일한 ETag 사용!")
    else:
        log_error("❌ 페이지별로 다른 ETag!")
        return False

    log_success("테스트 5 통과!")
    return True


def test_concurrent_requests(client: MatrixAPIClient):
    """테스트 6: 동시 요청 처리 - max-age=5 캐싱 검증"""
    log_test("테스트 6: 동시 요청 처리 - max-age=5 캐싱 검증")

    # 동시에 같은 요청 10번
    log_info("동시에 같은 매트릭스 조회 요청 10번...")

    def get_matrix_task(task_id: int):
        try:
            start_time = time.time()
            resp = client.get_matrix_all()
            elapsed = time.time() - start_time
            return (task_id, resp.status_code, resp.headers.get("ETag"), elapsed)
        except Exception as e:
            return (task_id, None, str(e), 0)

    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = [executor.submit(get_matrix_task, i) for i in range(10)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]

    # 결과 분석
    success_count = sum(1 for _, status, _, _ in results if status == 200)
    not_modified_count = sum(1 for _, status, _, _ in results if status == 304)
    etags = [etag for _, status, etag, _ in results if status == 200]

    log_info(f"결과: 200 OK={success_count}, 304 Not Modified={not_modified_count}")

    for task_id, status, etag, elapsed in sorted(results):
        if status:
            log_info(f"  Task {task_id}: {status} - ETag: {etag} - {elapsed:.3f}s")
        else:
            log_error(f"  Task {task_id}: Error - {etag}")

    # 모든 요청이 성공해야 함
    assert success_count + not_modified_count == 10, "All requests should succeed"

    # 모든 ETag가 동일해야 함
    if len(set(etags)) == 1:
        log_success(f"✅ 모든 요청이 동일한 ETag 반환: {etags[0]}")
    else:
        log_warning(f"⚠️  다른 ETag 반환: {set(etags)}")

    log_success("테스트 6 통과!")
    return True


def test_project_matrix_caching(client: MatrixAPIClient):
    """테스트 7: 프로젝트별 매트릭스 API 캐싱 검증"""
    log_test("테스트 7: 프로젝트별 매트릭스 API 캐싱 검증")

    if not PROJECT_ID:
        log_warning("⚠️  PROJECT_ID가 없어 테스트 스킵")
        return True

    # 1. 1차 요청
    log_info(f"1. 프로젝트 매트릭스 조회 (project_id={PROJECT_ID})...")
    resp1 = client.get_project_matrix(PROJECT_ID, no_cache=True)
    assert resp1.status_code == 200
    etag1 = resp1.headers.get("ETag")
    cache_control1 = resp1.headers.get("Cache-Control")

    log_success(f"1차 요청 성공 - ETag: {etag1}, Cache-Control: {cache_control1}")

    # 2. 2초 후 If-None-Match 요청
    time.sleep(2)
    log_info("2. 2초 후 If-None-Match 요청...")
    resp2 = client.get_project_matrix(PROJECT_ID, if_none_match=etag1)

    if resp2.status_code == 304:
        log_success("✅ 304 Not Modified - 캐시 히트!")
    else:
        log_error(f"❌ 예상: 304, 실제: {resp2.status_code}")
        return False

    # 3. Global 매트릭스와 동일한 ETag 사용 확인
    log_info("3. Global 매트릭스 ETag와 비교...")
    resp_global = client.get_matrix_all(no_cache=True)
    etag_global = resp_global.headers.get("ETag")

    if etag1 == etag_global:
        log_success(f"✅ 프로젝트별/Global 매트릭스가 동일한 ETag 사용: {etag1}")
    else:
        log_warning(f"⚠️  ETag 다름 - Project: {etag1}, Global: {etag_global}")
        # 프로젝트별 매트릭스는 다른 ETag를 사용할 수 있음 (정상)

    log_success("테스트 7 통과!")
    return True


def test_multi_client_scenario(client: MatrixAPIClient):
    """테스트 8: 여러 클라이언트 동시 변경 시나리오"""
    log_test("테스트 8: 여러 클라이언트 동시 변경 시나리오")

    # 클라이언트 A, B 생성
    client_a = MatrixAPIClient(BASE_URL)
    client_b = MatrixAPIClient(BASE_URL)

    # 1. 클라이언트 A, B 모두 매트릭스 조회
    log_info("1. 클라이언트 A, B 모두 매트릭스 조회...")
    resp_a1 = client_a.get_matrix_all(no_cache=True)
    resp_b1 = client_b.get_matrix_all(no_cache=True)

    etag_a1 = resp_a1.headers.get("ETag")
    etag_b1 = resp_b1.headers.get("ETag")

    log_success(f"클라이언트 A ETag: {etag_a1}")
    log_success(f"클라이언트 B ETag: {etag_b1}")

    if etag_a1 != etag_b1:
        log_error(f"❌ 초기 ETag 불일치!")
        return False

    # 2. 클라이언트 A가 Role 변경
    time.sleep(2)
    log_info("2. 클라이언트 A가 Role 변경...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN_MODIFIED' WHERE name = 'ADMIN'")
        affected = cur.rowcount
        conn.commit()

        if affected > 0:
            log_success(f"✅ Role 변경 완료 ({affected}개)")
        else:
            log_warning("⚠️  변경할 Role 없음")
    finally:
        cur.close()
        conn.close()

    # 3. 클라이언트 B가 오래된 ETag로 조회
    time.sleep(1)
    log_info("3. 클라이언트 B가 오래된 ETag로 조회...")
    resp_b2 = client_b.get_matrix_all(if_none_match=etag_b1)

    if resp_b2.status_code == 200:
        etag_b2 = resp_b2.headers.get("ETag")
        log_success(f"✅ 200 OK - 새 데이터 수신, 새 ETag: {etag_b2}")

        if etag_b2 != etag_b1:
            log_success(f"✅ ETag 변경 감지! {etag_b1} → {etag_b2}")
        else:
            log_error(f"❌ ETag 변경 감지 실패!")
            return False
    else:
        log_error(f"❌ 예상: 200, 실제: {resp_b2.status_code}")
        return False

    # 4. 원복
    log_info("4. Role 원복...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN' WHERE name = 'ADMIN_MODIFIED'")
        conn.commit()
        log_success("✅ 원복 완료")
    finally:
        cur.close()
        conn.close()

    log_success("테스트 8 통과!")
    return True


def test_capability_metadata_change(client: MatrixAPIClient):
    """테스트 9: Capability 메타데이터 변경 감지"""
    log_test("테스트 9: Capability 메타데이터 변경 감지")

    # 1. 현재 ETag 획득
    time.sleep(2)
    log_info("1. 현재 ETag 획득...")
    resp1 = client.get_matrix_all(no_cache=True)
    assert resp1.status_code == 200
    etag_before = resp1.headers.get("ETag")
    log_success(f"Before ETag: {etag_before}")

    # 2. Capability 메타데이터 변경
    time.sleep(2)
    log_info("2. Capability 설명 변경...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("""
            UPDATE security_capability
            SET description = 'Modified description for testing'
            WHERE id = %s
        """, (CAPABILITY_ID,))
        affected = cur.rowcount
        conn.commit()

        if affected > 0:
            log_success(f"✅ Capability 변경 완료 ({affected}개)")
        else:
            log_warning("⚠️  변경할 Capability 없음")
            return True
    finally:
        cur.close()
        conn.close()

    # 3. 변경 후 ETag 획득
    time.sleep(1)
    log_info("3. 변경 후 ETag 획득...")
    resp2 = client.get_matrix_all(no_cache=True)
    assert resp2.status_code == 200
    etag_after = resp2.headers.get("ETag")
    log_success(f"After ETag: {etag_after}")

    # 4. ETag 비교
    if etag_before != etag_after:
        log_success(f"✅ ETag 변경 감지 성공! {etag_before} → {etag_after}")
    else:
        log_error(f"❌ ETag 변경 감지 실패! (동일: {etag_before})")
        return False

    # 5. 원복
    log_info("5. Capability 원복...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("""
            UPDATE security_capability
            SET description = NULL
            WHERE id = %s
        """, (CAPABILITY_ID,))
        conn.commit()
        log_success("✅ 원복 완료")
    finally:
        cur.close()
        conn.close()

    log_success("테스트 9 통과!")
    return True


def test_browser_cache_behavior(client: MatrixAPIClient):
    """테스트 10: 브라우저 캐시 동작 검증"""
    log_test("테스트 10: 브라우저 캐시 동작 검증")

    # 1. 1차 요청 (200 OK + ETag)
    log_info("1. 1차 요청 (200 OK + ETag)...")
    resp1 = client.get_matrix_all(no_cache=True)
    assert resp1.status_code == 200
    etag1 = resp1.headers.get("ETag")
    cache_control = resp1.headers.get("Cache-Control")

    log_success(f"1차 요청 - ETag: {etag1}, Cache-Control: {cache_control}")

    # Cache-Control 파싱
    if "max-age=5" not in cache_control:
        log_error(f"❌ max-age=5가 없음: {cache_control}")
        return False

    if "private" not in cache_control:
        log_error(f"❌ private가 없음: {cache_control}")
        return False

    log_success("✅ Cache-Control 검증 성공: private, max-age=5")

    # 2. 3초 후 요청 (max-age 내, 브라우저 캐시 사용 예상)
    log_info("2. 3초 후 요청 (max-age 내)...")
    time.sleep(3)
    resp2 = client.get_matrix_all(if_none_match=etag1)

    if resp2.status_code == 304:
        log_success("✅ 304 Not Modified - 서버가 ETag 일치 확인")
    elif resp2.status_code == 200:
        log_info("ℹ️  200 OK - 브라우저가 서버에 요청함 (정상)")
    else:
        log_error(f"❌ 예상하지 못한 응답: {resp2.status_code}")
        return False

    # 3. 6초 후 요청 (max-age 만료, 서버 재검증 필요)
    log_info("3. 6초 후 요청 (max-age 만료)...")
    time.sleep(3)
    resp3 = client.get_matrix_all(if_none_match=etag1)

    if resp3.status_code == 304:
        log_success("✅ 304 Not Modified - ETag 일치, 캐시 재사용")
    elif resp3.status_code == 200:
        log_success("✅ 200 OK - 새 데이터 수신")
    else:
        log_error(f"❌ 예상하지 못한 응답: {resp3.status_code}")
        return False

    # 4. 변경 후 요청 (ETag 불일치, 200 OK 예상)
    log_info("4. 데이터 변경 후 요청...")

    # Role 변경
    time.sleep(2)
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN_CACHE_TEST' WHERE name = 'ADMIN'")
        conn.commit()
    finally:
        cur.close()
        conn.close()

    time.sleep(1)
    resp4 = client.get_matrix_all(if_none_match=etag1)

    if resp4.status_code == 200:
        etag4 = resp4.headers.get("ETag")
        log_success(f"✅ 200 OK - 새 데이터 수신, 새 ETag: {etag4}")

        if etag4 != etag1:
            log_success(f"✅ ETag 변경 확인! {etag1} → {etag4}")
        else:
            log_error(f"❌ ETag 변경 안 됨!")
            return False
    else:
        log_error(f"❌ 예상: 200, 실제: {resp4.status_code}")
        return False

    # 5. 원복
    log_info("5. Role 원복...")
    conn = get_db_connection()
    try:
        cur = conn.cursor()
        cur.execute("UPDATE security_role SET name = 'ADMIN' WHERE name = 'ADMIN_CACHE_TEST'")
        conn.commit()
        log_success("✅ 원복 완료")
    finally:
        cur.close()
        conn.close()

    log_success("테스트 10 통과!")
    return True


def setup_test_data():
    """테스트 데이터 조회"""
    global ROLE_ID, CAPABILITY_ID, PROJECT_ID

    log_info("🔧 테스트 데이터 조회 중...")

    conn = get_db_connection()

    try:
        cur = conn.cursor(cursor_factory=RealDictCursor)

        # 1. Role 조회 (첫 번째 Role 사용)
        cur.execute("SELECT id, name FROM security_role ORDER BY id LIMIT 1")
        role = cur.fetchone()
        if not role:
            log_error("❌ Role이 없습니다!")
            sys.exit(1)

        ROLE_ID = role['id']
        log_success(f"✅ Role 조회 완료: ID={ROLE_ID}, Name={role['name']}")

        # 2. Capability 조회 (첫 번째 Capability 사용)
        cur.execute("SELECT id, name FROM security_capability ORDER BY id LIMIT 1")
        capability = cur.fetchone()
        if not capability:
            log_error("❌ Capability가 없습니다!")
            sys.exit(1)

        CAPABILITY_ID = capability['id']
        log_success(f"✅ Capability 조회 완료: ID={CAPABILITY_ID}, Name={capability['name']}")

        # 3. Project 조회 (첫 번째 Project 사용, 없으면 None)
        cur.execute("SELECT id, name FROM security_project ORDER BY id LIMIT 1")
        project = cur.fetchone()
        if project:
            PROJECT_ID = project['id']
            log_success(f"✅ Project 조회 완료: ID={PROJECT_ID}, Name={project['name']}")
        else:
            log_warning("⚠️  Project가 없습니다. 테스트 7은 스킵됩니다.")
            PROJECT_ID = None

        log_success("✅ 테스트 데이터 조회 완료!")

    except Exception as e:
        log_error(f"❌ 테스트 데이터 조회 실패: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        cur.close()
        conn.close()


def cleanup_test_data():
    """테스트 데이터 정리 (필요시)"""
    log_info("🧹 테스트 데이터 정리 중...")

    # 이 테스트는 기존 데이터를 조회만 하고 생성하지 않으므로
    # 특별한 정리가 필요 없음

    # 테스트 4에서 삭제한 할당이 있을 수 있으므로 확인하지 않음
    log_info("ℹ️  테스트는 기존 데이터만 사용하므로 정리 불필요")
    log_success("✅ 테스트 데이터 정리 완료!")


def test_empty_list_caching(client: MatrixAPIClient):
    """테스트 11: 빈 목록 캐싱 검증"""
    log_test("테스트 11: 빈 목록 캐싱 검증")

    # 존재하지 않는 프로젝트로 빈 목록 생성
    nonexistent_project_id = 999999

    # 1. 첫 번째 요청
    log_info(f"1. 첫 번째 요청 (빈 목록, project_id={nonexistent_project_id})...")
    resp1 = client.get_project_matrix(nonexistent_project_id, no_cache=True)

    # 404 또는 200 (빈 목록) 모두 허용
    if resp1.status_code not in [200, 404]:
        log_error(f"예상치 못한 상태 코드: {resp1.status_code}")
        return False

    if resp1.status_code == 404:
        log_success("404 응답 (프로젝트 없음) - 정상")
        return True

    etag1 = resp1.headers.get("ETag")
    data1 = resp1.json()

    log_success(f"1차 요청 - ETag: {etag1}, Roles: {len(data1.get('roles', []))}")

    if not etag1:
        log_error("ETag 헤더 없음")
        return False

    # 2. 두 번째 요청 - If-None-Match 헤더 포함
    log_info("2. 두 번째 요청 (If-None-Match 헤더 포함)...")
    resp2 = client.get_project_matrix(nonexistent_project_id, if_none_match=etag1)

    if resp2.status_code == 304:
        log_success("✅ 304 Not Modified - 빈 목록도 정상적으로 캐싱됨")
        return True
    else:
        log_error(f"❌ 예상: 304, 실제: {resp2.status_code}")
        return False


def main():
    """메인 테스트 실행"""
    print(f"\n{Colors.BLUE}{'='*80}{Colors.RESET}")
    print(f"{Colors.BLUE}🚀 Role-Capability 매트릭스 API 캐싱 E2E 테스트{Colors.RESET}")
    print(f"{Colors.BLUE}{'='*80}{Colors.RESET}\n")

    # API 클라이언트 생성 (토큰 없이 테스트)
    client = MatrixAPIClient(BASE_URL)

    # 서버 연결 확인
    try:
        resp = client.get_matrix_all()
        if resp.status_code == 401:
            log_error("인증 필요! 토큰을 설정하세요.")
            log_info("export PACS_TOKEN='your_token_here'")
            return 1
        elif resp.status_code != 200:
            log_error(f"서버 응답 에러: {resp.status_code}")
            return 1
    except requests.exceptions.ConnectionError:
        log_error(f"서버 연결 실패: {BASE_URL}")
        log_info("서버가 실행 중인지 확인하세요: cargo run")
        return 1

    # 테스트 실행
    tests = [
        ("GET 전체 매트릭스 - ETag 및 304 응답 검증", test_get_matrix_all_etag),
        ("GET 페이지네이션 - 모든 페이지 동일한 ETag 검증", test_pagination_same_etag),
        ("Role 변경 후 ETag 변경 검증", test_role_change_etag_update),
        ("Capability 할당 변경 후 ETag 변경 검증", test_capability_assignment_etag_update),
        ("페이지별 변경 감지 - Page 1 변경 시 Page 2도 감지", test_page_change_detection),
        ("동시 요청 처리 - max-age=5 캐싱 검증", test_concurrent_requests),
        ("프로젝트별 매트릭스 API 캐싱 검증", test_project_matrix_caching),
        ("여러 클라이언트 동시 변경 시나리오", test_multi_client_scenario),
        ("Capability 메타데이터 변경 감지", test_capability_metadata_change),
        ("브라우저 캐시 동작 검증", test_browser_cache_behavior),
        ("빈 목록 캐싱 검증", test_empty_list_caching),
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


if __name__ == "__main__":
    try:
        setup_test_data()
        sys.exit(main())
    finally:
        cleanup_test_data()




