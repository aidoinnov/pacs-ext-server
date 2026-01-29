# 사용자 역할 할당 API 캐싱 E2E 테스트

## 📋 개요

사용자 역할 할당 API의 HTTP 캐싱 동작을 검증하는 E2E 테스트입니다.

### 캐싱 전략

| API | Cache-Control | ETag | 목적 |
|-----|---------------|------|------|
| `PUT /api/projects/{id}/users/{id}/role` | `private, max-age=1` | ✅ | 중복 요청 방지 |
| `POST /api/projects/{id}/users/roles` | `private, max-age=1` | ✅ | 중복 요청 방지 |
| `GET /api/projects/{id}/users` | `private, max-age=1` | ✅ | 1초 내 중복 조회 차단 |

### 클라이언트 캐시 무효화

PUT/POST 후 최신 데이터가 필요한 경우:

```javascript
// 역할 할당 후
await fetch('/api/projects/1/users/5/role', {
  method: 'PUT',
  body: JSON.stringify({role_id: 2})
});

// 캐시 무시하고 최신 데이터 조회
await fetch('/api/projects/1/users', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

---

## 🧪 테스트 시나리오

### 1. PUT 중복 요청 방지 (max-age=1)

**목적**: 1초 내 동일한 PUT 요청이 캐시되는지 확인

**시나리오**:
1. PUT 요청 (역할 할당)
2. 0.3초 후 동일한 PUT 요청 (If-None-Match 포함)
3. 1.5초 후 PUT 요청 (캐시 만료 후)

**기대 결과**:
- 1차: 200 OK, ETag 반환, `max-age=1`
- 2차: 304 Not Modified (ETag 일치) 또는 200 OK
- 3차: 200 OK, 새로운 ETag

---

### 2. GET 1초 내 캐시 히트

**목적**: GET 요청이 1초 내 캐시되는지 확인

**시나리오**:
1. GET 요청 (멤버 목록 조회)
2. 0.5초 후 동일한 GET 요청 (If-None-Match 포함)
3. 1.5초 후 GET 요청 (캐시 만료 후)

**기대 결과**:
- 1차: 200 OK, ETag 반환, `max-age=1`
- 2차: 304 Not Modified (데이터 변경 없음)
- 3차: 200 OK

---

### 3. PUT 후 GET - 클라이언트 캐시 무효화

**목적**: PUT 후 GET 시 오래된 캐시 문제 확인 및 해결

**시나리오**:
1. GET 요청 (캐시 생성)
2. 0.3초 후 PUT 요청 (역할 변경)
3. 즉시 GET 요청 (no-cache 없이)
4. `Cache-Control: no-cache`로 GET 요청

**기대 결과**:
- 3번: 304 Not Modified (오래된 캐시) ⚠️
- 4번: 200 OK (최신 데이터) ✅

**교훈**: PUT/POST 후 최신 데이터가 필요하면 `Cache-Control: no-cache` 필수!

---

### 4. ETag 검증 (304 Not Modified)

**목적**: ETag 기반 조건부 요청 동작 확인

**시나리오**:
1. GET 요청으로 ETag 획득
2. 1.5초 후 GET 요청 (If-None-Match 포함)
3. 역할 변경 (PUT)
4. GET 요청 (이전 ETag 사용)

**기대 결과**:
- 2번: 304 Not Modified (데이터 변경 없음)
- 4번: 200 OK (ETag 불일치, 데이터 변경됨)

---

### 5. 동시 요청 처리

**목적**: 동시에 같은 요청이 들어올 때 처리 확인

**시나리오**:
- 5개 스레드에서 동시에 같은 역할 할당 요청

**기대 결과**:
- 최소 1개는 200 OK
- 나머지는 200 또는 304 (캐시/ETag에 따라)

---

### 6. 일괄 역할 할당 캐싱

**목적**: POST 요청도 캐싱되는지 확인

**시나리오**:
1. POST 일괄 역할 할당
2. 0.5초 후 동일한 POST 요청

**기대 결과**:
- 1차: 200 OK, ETag 반환, `max-age=1`
- 2차: 200 OK (POST는 멱등성 없음, 항상 실행)

---

## 🚀 실행 방법

### 1. 서버 실행

```bash
cd pacs-server
cargo run
```

### 2. Migration 실행

```bash
psql -U postgres -d pacs_ext_db -f migrations/044_add_updated_at_to_user_project.sql
```

### 3. 테스트 실행

```bash
cd pacs-server/e2e
python3 test_role_assignment_cache.py
```

### 4. 인증이 필요한 경우

```bash
export PACS_TOKEN='your_jwt_token_here'
python3 test_role_assignment_cache.py
```

---

## 📊 예상 출력

```
================================================================================
🚀 사용자 역할 할당 API 캐싱 E2E 테스트
================================================================================

================================================================================
🧪 테스트 1: PUT 중복 요청 방지 (max-age=1)
================================================================================
ℹ️  1차 PUT 요청...

1차 응답:
  Status: 200
  Cache-Control: private, max-age=1
  ETag: "1737123456"
  updated_at: 2025-01-19T12:34:56Z

✅ 1차 요청 성공 - ETag: "1737123456", updated_at: 2025-01-19T12:34:56Z
ℹ️  0.3초 후 2차 PUT 요청 (If-None-Match 포함)...

2차 응답:
  Status: 304
  Cache-Control: private, max-age=1
  ETag: "1737123456"

✅ 304 Not Modified - ETag 일치, 변경 없음
✅ 테스트 1 통과!

...

================================================================================
📊 테스트 결과
================================================================================
✅ 통과: 6
❌ 실패: 0
================================================================================
```

---

## 🔍 디버깅

### 테스트 실패 시 확인 사항

1. **서버가 실행 중인가?**
   ```bash
   curl http://localhost:8080/health
   ```

2. **Migration이 적용되었는가?**
   ```sql
   \d security_user_project
   -- updated_at 컬럼이 있어야 함
   ```

3. **테스트 데이터가 존재하는가?**
   ```sql
   SELECT * FROM security_user WHERE id = 5;
   SELECT * FROM security_project WHERE id = 1;
   ```

4. **캐시 헤더가 올바른가?**
   ```bash
   curl -v http://localhost:8080/api/projects/1/users
   # Cache-Control: private, max-age=1
   # ETag: "..."
   ```

---

## 📝 참고 자료

- [HTTP Caching - MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [ETag - MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- [Cache-Control - MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)

