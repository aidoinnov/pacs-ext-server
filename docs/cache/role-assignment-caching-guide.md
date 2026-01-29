# 프로젝트 사용자 역할 할당 API - HTTP 캐싱 가이드

## 개요

프로젝트 사용자 역할 할당 API는 **HTTP 캐싱**을 활용하여 성능을 최적화합니다.
이 문서는 클라이언트 개발자가 캐싱 메커니즘을 올바르게 활용하는 방법을 설명합니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **캐싱 정책**: `Cache-Control: private, max-age=1` + ETag

---

## 🎯 캐싱 전략 요약

### **핵심 개념**

1. **`max-age=1`**: 1초 동안 브라우저 캐시 사용 (중복 요청 방지)
2. **ETag**: 데이터 변경 감지 (304 Not Modified 응답)
3. **`updated_at` 기반**: DB 타임스탬프로 ETag 생성 (서버 부하 절감)

### **장점**

- ✅ **중복 요청 방지**: 1초 내 동일 요청 차단
- ✅ **서버 부하 절감**: 304 응답 시 DB 조회 생략
- ✅ **네트워크 절약**: 304 응답 시 body 없음
- ✅ **클라이언트 제어**: `Cache-Control: no-cache`로 강제 새로고침 가능

---

## 📡 API 엔드포인트

### 1. 사용자 역할 할당 (PUT)

#### 요청

```http
PUT /api/projects/{project_id}/users/{user_id}/role
Content-Type: application/json

{
  "role_id": 2
}
```

#### 응답

**성공 (200 OK)**:
```http
HTTP/1.1 200 OK
Cache-Control: private, max-age=1
ETag: W/"1768834933"

{
  "message": "Role assigned successfully",
  "user_id": 5,
  "project_id": 1,
  "role_id": 2,
  "updated_at": "2026-01-19T15:02:13.530911Z"
}
```

**캐시 히트 (304 Not Modified)**:
```http
HTTP/1.1 304 Not Modified
Cache-Control: private, max-age=1
ETag: W/"1768834933"
```

---

### 2. 프로젝트 멤버 목록 조회 (GET)

#### 요청

```http
GET /api/projects/{project_id}/users?page=1&page_size=20
```

#### 응답

**성공 (200 OK)**:
```http
HTTP/1.1 200 OK
Cache-Control: private, max-age=1
ETag: W/"1768834935"

{
  "members": [
    {
      "user_id": 5,
      "username": "john_doe",
      "email": "john@example.com",
      "role_id": 2,
      "role_name": "Editor",
      "updated_at": "2026-01-19T15:02:15.491201Z"
    }
  ],
  "total_count": 10,
  "page": 1,
  "page_size": 20,
  "total_pages": 1,
  "latest_updated_at": "2026-01-19T15:02:15.491201Z"
}
```

---

## 🚀 클라이언트 구현 가이드

### **시나리오 1: 일반적인 조회**

브라우저가 자동으로 캐싱을 처리합니다.

```javascript
// 1차 요청 - 200 OK
const response1 = await fetch('/api/projects/1/users');
const data1 = await response1.json();

// 0.5초 후 2차 요청 - 브라우저가 캐시 사용
await sleep(500);
const response2 = await fetch('/api/projects/1/users');
const data2 = await response2.json(); // 캐시된 데이터

// 1.5초 후 3차 요청 - 캐시 만료, 서버에 If-None-Match 전송
await sleep(1500);
const response3 = await fetch('/api/projects/1/users');
// 데이터 변경 없으면 304, 변경 있으면 200
```

---

### **시나리오 2: 역할 할당 후 최신 데이터 조회**

역할 할당 후 즉시 최신 데이터가 필요한 경우.

#### ❌ **잘못된 방법**

```javascript
// 역할 할당
await fetch('/api/projects/1/users/5/role', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ role_id: 2 })
});

// 즉시 조회 - 브라우저 캐시 사용 가능
const response = await fetch('/api/projects/1/users');


#### ✅ **올바른 방법 1: `Cache-Control: no-cache` 사용** (권장)

```javascript
// 역할 할당
await fetch('/api/projects/1/users/5/role', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ role_id: 2 })
});

// 캐시 무시하고 최신 데이터 조회
const response = await fetch('/api/projects/1/users', {
  headers: {
    'Cache-Control': 'no-cache'  // ← 브라우저 캐시 무시
  }
});
const data = await response.json(); // ✅ 최신 데이터 보장
```

#### ✅ **올바른 방법 2: `cache: 'reload'` 사용**

```javascript
// 역할 할당
await fetch('/api/projects/1/users/5/role', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ role_id: 2 })
});

// 브라우저 캐시 완전 무시
const response = await fetch('/api/projects/1/users', {
  cache: 'reload'  // ← 브라우저 캐시 완전 무시
});
const data = await response.json(); // ✅ 최신 데이터 보장
```

---

### **시나리오 3: 중복 클릭 방지**

사용자가 "역할 할당" 버튼을 빠르게 여러 번 클릭하는 경우.

```javascript
async function assignRole(projectId, userId, roleId) {
  try {
    const response = await fetch(`/api/projects/${projectId}/users/${userId}/role`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ role_id: roleId })
    });

    if (response.status === 304) {
      console.log('이미 동일한 역할이 할당되어 있습니다.');
      return;
    }

    if (response.ok) {
      const data = await response.json();
      console.log('역할 할당 성공:', data);
    }
  } catch (error) {
    console.error('역할 할당 실패:', error);
  }
}

// 사용자가 0.3초 간격으로 2번 클릭
assignRole(1, 5, 2); // 1차 요청 - 200 OK
setTimeout(() => assignRole(1, 5, 2), 300); // 2차 요청 - 304 (캐시 히트)
```

**효과**: 1초 내 중복 요청은 서버 부하 없이 304 응답

---

### **시나리오 4: ETag를 활용한 조건부 요청**

수동으로 ETag를 관리하여 네트워크 절약.

```javascript
let cachedETag = null;
let cachedData = null;

async function getMembers(projectId, forceRefresh = false) {
  const headers = {};

  // 강제 새로고침이 아니고 ETag가 있으면 If-None-Match 추가
  if (!forceRefresh && cachedETag) {
    headers['If-None-Match'] = cachedETag;
  }

  const response = await fetch(`/api/projects/${projectId}/users`, { headers });

  if (response.status === 304) {
    console.log('데이터 변경 없음 - 캐시 사용');
    return cachedData; // 캐시된 데이터 반환
  }

  if (response.ok) {
    cachedETag = response.headers.get('ETag');
    cachedData = await response.json();
    console.log('새 데이터 수신 - ETag:', cachedETag);
    return cachedData;
  }

  throw new Error('Failed to fetch members');
}

// 사용 예시
const data1 = await getMembers(1); // 200 OK, ETag 저장
const data2 = await getMembers(1); // 304 Not Modified (데이터 변경 없으면)
const data3 = await getMembers(1, true); // 200 OK (강제 새로고침)
```

---

## 🔍 응답 헤더 이해하기

### **Cache-Control: private, max-age=1**

- **`private`**: 브라우저만 캐시 가능 (CDN/프록시 캐시 불가)
- **`max-age=1`**: 1초 동안 캐시 유효

### **ETag: W/"1768834933"**

- **`W/`**: Weak ETag (약한 ETag)
- **`"1768834933"`**: `updated_at` 타임스탬프 (Unix timestamp)

### **If-None-Match 요청 헤더**

브라우저가 자동으로 추가하는 헤더:

```http
GET /api/projects/1/users
If-None-Match: W/"1768834933"
```

서버는 현재 ETag와 비교:
- **일치** → 304 Not Modified (body 없음)
- **불일치** → 200 OK (새 데이터 + 새 ETag)

---

## ⚠️ 주의사항

### 1. **PUT 후 즉시 GET 시 캐시 무효화 필요**

```javascript
// ❌ 잘못된 예
await updateRole(userId, roleId);
const members = await getMembers(projectId); // 캐시된 이전 데이터 가능

// ✅ 올바른 예
await updateRole(userId, roleId);
const members = await fetch('/api/projects/1/users', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

### 2. **브라우저 개발자 도구에서 "Disable cache" 주의**

개발 중 "Disable cache" 옵션이 켜져 있으면 캐싱 동작을 확인할 수 없습니다.

### 3. **304 응답도 성공**

```javascript
const response = await fetch('/api/projects/1/users');

// ❌ 잘못된 체크
if (response.status === 200) {
  // 304일 때도 성공인데 처리 안 함
}

// ✅ 올바른 체크
if (response.ok) { // 200-299 모두 true
  // 200, 304 모두 처리
}
```

---

## 📊 성능 비교

### **캐싱 없을 때**

```
요청 1: DB 조회 (50ms) + 네트워크 (100ms) = 150ms
요청 2: DB 조회 (50ms) + 네트워크 (100ms) = 150ms
요청 3: DB 조회 (50ms) + 네트워크 (100ms) = 150ms
총: 450ms, DB 부하 3회
```

### **캐싱 있을 때 (1초 내 3번 요청)**

```
요청 1: DB 조회 (50ms) + 네트워크 (100ms) = 150ms
요청 2: 브라우저 캐시 (0ms) = 0ms
요청 3: 브라우저 캐시 (0ms) = 0ms
총: 150ms, DB 부하 1회
```

### **ETag 활용 (데이터 변경 없을 때)**

```
요청 1: DB 조회 (50ms) + 네트워크 (100ms) = 150ms
요청 2 (1.5초 후): ETag 체크 (5ms) + 304 응답 (50ms) = 55ms
총: 205ms, DB 부하 1회 (ETag 체크는 DB 조회 없음)
```

---

## 🧪 테스트 방법

### **브라우저 개발자 도구**

1. Network 탭 열기
2. "Disable cache" **끄기**
3. API 요청 실행
4. 응답 헤더 확인:
   - `Cache-Control: private, max-age=1`
   - `ETag: W/"..."`
5. 1초 내 재요청 → 캐시 사용 (Network 탭에 "from disk cache" 표시)
6. 1초 후 재요청 → 서버 요청 (If-None-Match 헤더 확인)

### **curl 테스트**

```bash
# 1차 요청
curl -i http://localhost:8080/api/projects/1/users

# ETag 복사 (예: W/"1768834933")

# 2차 요청 (If-None-Match 포함)
curl -i -H "If-None-Match: W/\"1768834933\"" \
  http://localhost:8080/api/projects/1/users

# 304 Not Modified 응답 확인
```

---

## 📚 참고 자료

- [MDN - HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [MDN - ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- [MDN - Cache-Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)

---

## 🎯 요약

| 상황 | 방법 | 코드 |
|------|------|------|
| 일반 조회 | 그냥 fetch | `fetch('/api/projects/1/users')` |
| PUT 후 조회 | `no-cache` 헤더 | `fetch(url, {headers: {'Cache-Control': 'no-cache'}})` |
| 강제 새로고침 | `cache: 'reload'` | `fetch(url, {cache: 'reload'})` |
| 중복 방지 | 자동 처리 | 1초 내 중복 요청은 자동으로 캐시 사용 |

**핵심**: PUT/POST 후 즉시 GET 할 때는 **`Cache-Control: no-cache`** 헤더 추가! ✨

