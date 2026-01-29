# HTTP Caching E2E Tests

HTTP 캐싱 기능에 대한 E2E 테스트 문서입니다.

## 📋 테스트 개요

### 구현된 캐싱 API

1. **User Role Assignment APIs** (6개 테스트)
   - `GET /api/projects/{project_id}/users/{user_id}/role`
   - ETag: `updated_at` 타임스탬프
   - Cache-Control: `private, max-age=1`

2. **Role-Capability Matrix APIs** (10개 테스트)
   - `GET /api/roles/{role_id}/capabilities`
   - ETag: `MAX(updated_at)` from 3 tables
   - Cache-Control: `private, max-age=60`

3. **Capability APIs** (24개 테스트)
   - `GET /api/capabilities` - 모든 Capability 목록
   - `GET /api/capabilities/{id}` - Capability 상세
   - `GET /api/capabilities/category/{category}` - 카테고리별 Capability
   - ETag: `MAX(updated_at)` 타임스탬프
   - Cache-Control: `private, max-age=60`

---

## 🧪 테스트 파일

### `test_capability_cache_e2e.py`

**테스트 시나리오 (24개)**:

1. **기본 캐싱 동작** (11개)
   - ✅ 모든 Capability 목록 캐싱
   - ✅ Capability 상세 캐싱
   - ✅ 카테고리별 Capability 캐싱
   - ✅ ETag 헤더 존재 확인
   - ✅ Cache-Control 헤더 확인
   - ✅ 304 Not Modified 응답 확인

2. **캐시 무효화** (3개)
   - ✅ Role-Capability 할당 후 캐시 상태 확인
   - ✅ 데이터 변경 없을 때 캐시 유지

3. **동시성 테스트** (2개)
   - ✅ 10개 동시 요청 시 모두 304 반환
   - ✅ 모든 ETag 일관성 확인

4. **에러 처리** (5개)
   - ✅ 잘못된 ETag 처리 (5가지 케이스)
   - ✅ 서버가 200으로 정상 응답

5. **ETag 형식 검증** (3개)
   - ✅ Weak ETag 형식 확인 (`W/"숫자"`)
   - ✅ 타임스탬프 범위 검증
   - ✅ 정규식 패턴 매칭

---

## 🚀 테스트 실행 방법

### 1. 개별 테스트 실행

```bash
# Capability 캐싱 테스트만 실행
cd pacs-server/e2e
python3 test_capability_cache_e2e.py
```

### 2. 모든 캐시 테스트 실행

```bash
# 모든 HTTP 캐싱 테스트 실행
cd pacs-server/e2e
./run_cache_tests.sh
```

**실행 결과**:
```
==========================================
🚀 캐시 관련 E2E 테스트 실행
==========================================

📦 Annotation 캐시 테스트
✅ PASSED: HEAD 요청 및 캐시 검증

⚡ HTTP Caching 테스트
✅ PASSED: Capability API Cache

📊 테스트 결과 요약
총 테스트: 2
통과: 2
실패: 0

🎉 모든 캐시 테스트 통과!
```

### 3. 전체 E2E 테스트 실행

```bash
# 모든 E2E 테스트 실행 (캐시 테스트 포함)
cd pacs-server/e2e
./run_all_tests.sh
```

### 4. 전용 캐시 테스트 스위트 실행

```bash
# HTTP 캐싱 전용 테스트 스위트
cd pacs-server/e2e
./run_all_cache_tests.sh
```

---

## 📊 테스트 결과 예시

```
============================================================
📋 Capability API 캐싱 E2E 테스트
============================================================

🔐 로그인 중...
✅ 로그인 성공

============================================================
🧪 All Capabilities Caching
============================================================
✅ First request - Cache miss (Status: 200)
✅ ETag header present (ETag: W/"1768903114709")
✅ Cache-Control header present (Cache-Control: private, max-age=60)
✅ Second request - Cache hit (304 Not Modified)
✅ Third request - Still cached (304)

============================================================
🧪 Concurrent Requests Cache Consistency
============================================================
✅ All concurrent requests returned 304
   Results: [(0, 304), (1, 304), ..., (9, 304)]
✅ All ETags consistent (Unique ETags: 1)

============================================================
📊 테스트 결과 요약
============================================================
✅ 통과: 24
❌ 실패: 0
📝 총계: 24

🎉 모든 테스트 통과!
```

---

## 🔧 사전 요구사항

### 1. 서버 실행

```bash
cd pacs-server
./target/debug/pacs_server
```

서버가 `http://localhost:8080`에서 실행 중이어야 합니다.

### 2. Python 의존성

```bash
pip3 install requests
```

### 3. 테스트 계정

- Username: `iaid-pacs-admin`
- Password: `Qlalfqjsgh1!`

---

## 📝 테스트 작성 가이드

새로운 캐싱 API 테스트를 추가하려면:

1. **테스트 파일 생성**: `test_<api_name>_cache_e2e.py`
2. **테스트 시나리오 작성**:
   - 첫 번째 요청 (캐시 미스)
   - ETag 헤더 확인
   - Cache-Control 헤더 확인
   - 두 번째 요청 (캐시 히트, 304)
   - 동시 요청 테스트
   - 잘못된 ETag 처리
3. **테스트 스크립트에 추가**:
   - `run_cache_tests.sh`에 테스트 추가
   - `run_all_tests.sh`에 테스트 추가

---

## 🐛 트러블슈팅

### 서버가 실행 중이 아닌 경우

```
❌ Server is not running at http://localhost:8080
💡 Please start the server first:
   cd ../pacs-server && ./target/debug/pacs_server
```

### 로그인 실패

```
❌ Login failed: 401 - {"error":"Login failed: ..."}
```

- Keycloak 서버가 실행 중인지 확인
- 테스트 계정이 존재하는지 확인

### Python 모듈 없음

```
❌ requests module not found
```

```bash
pip3 install requests
```

---

## 📚 관련 문서

- [TODO.md](../../TODO.md) - 완료된 캐싱 작업 목록
- [Migration 044](../migrations/044_add_user_project_updated_at.sql) - User Role Assignment
- [Migration 045](../migrations/045_add_role_updated_at.sql) - Role updated_at
- [Migration 046](../migrations/046_add_role_capability_updated_at.sql) - Role-Capability updated_at
- [Migration 047](../migrations/047_add_capability_updated_at_trigger.sql) - Capability updated_at trigger

