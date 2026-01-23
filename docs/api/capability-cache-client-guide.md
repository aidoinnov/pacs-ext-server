# Capability API - HTTP 캐싱 클라이언트 가이드

## 개요

Capability API는 **HTTP 캐싱 (ETag)**을 활용하여 성능을 최적화합니다.
이 문서는 클라이언트 개발자가 캐싱 메커니즘을 올바르게 활용하는 방법을 설명합니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **캐싱 정책**: `Cache-Control: private, max-age=60` + ETag

---

## 🎯 캐싱 전략 요약

### **핵심 개념**

1. **`max-age=60`**: 60초 동안 브라우저 캐시 사용
2. **ETag**: 데이터 변경 감지 (304 Not Modified 응답)
3. **`updated_at` 기반**: DB 타임스탬프로 ETag 생성

### **장점**

- ✅ **중복 요청 방지**: 60초 내 동일 요청 차단
- ✅ **서버 부하 절감**: 304 응답 시 DB 조회 생략
- ✅ **네트워크 절약**: 304 응답 시 body 없음
- ✅ **클라이언트 제어**: `Cache-Control: no-cache`로 강제 새로고침 가능

---

## 📡 캐싱이 적용된 API 엔드포인트

### 1. 전체 Capability 목록 조회

#### 요청

```http
GET /api/capabilities
```

#### 응답

**성공 (200 OK)**:
```http
HTTP/1.1 200 OK
Cache-Control: private, max-age=60
ETag: W/"1768903114709"

[
  {
    "id": 476,
    "name": "DICOM 데이터 조회",
    "category": "DICOM 데이터 관리",
    "description": "DICOM Study/Series/Instance 조회",
    "updated_at": "2026-01-20T12:45:14.709Z"
  },
  ...
]
```

**캐시 히트 (304 Not Modified)**:
```http
HTTP/1.1 304 Not Modified
Cache-Control: private, max-age=60
ETag: W/"1768903114709"
```

---

### 2. Capability 상세 조회

#### 요청

```http
GET /api/capabilities/{id}
```

#### 응답

**성공 (200 OK)**:
```http
HTTP/1.1 200 OK
Cache-Control: private, max-age=60
ETag: W/"1766987302915"

{
  "id": 476,
  "name": "DICOM 데이터 조회",
  "category": "DICOM 데이터 관리",
  "description": "DICOM Study/Series/Instance 조회",
  "updated_at": "2026-01-19T08:15:02.915Z"
}
```

---

### 3. 카테고리별 Capability 조회

#### 요청

```http
GET /api/capabilities/category/{category}
```

#### 응답

**성공 (200 OK)**:
```http
HTTP/1.1 200 OK
Cache-Control: private, max-age=60
ETag: W/"1768224228502"

[
  {
    "id": 476,
    "name": "DICOM 데이터 조회",
    "category": "DICOM 데이터 관리",
    ...
  }
]
```

---

## 🚀 클라이언트 구현 가이드

### **시나리오 1: 일반적인 조회**

브라우저가 자동으로 캐싱을 처리합니다.

```javascript
// 1차 요청 - 200 OK
const response1 = await fetch('/api/capabilities');
const capabilities1 = await response1.json();

// 30초 후 2차 요청 - 브라우저가 캐시 사용
await sleep(30000);
const response2 = await fetch('/api/capabilities');
const capabilities2 = await response2.json(); // 캐시된 데이터

// 70초 후 3차 요청 - 캐시 만료, 서버에 If-None-Match 전송
await sleep(70000);
const response3 = await fetch('/api/capabilities');
// 데이터 변경 없으면 304, 변경 있으면 200
```

---

### **시나리오 2: 역할 할당 후 최신 Capability 조회**

역할에 Capability를 할당한 후 즉시 최신 데이터가 필요한 경우.

#### ✅ **올바른 방법: `Cache-Control: no-cache` 사용**

```javascript
// Capability 할당
await fetch('/api/roles/1/capabilities', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ capability_id: 476 })
});

// 캐시 무시하고 최신 데이터 조회
const response = await fetch('/api/capabilities', {
  headers: {
    'Cache-Control': 'no-cache'  // ← 브라우저 캐시 무시
  }
});
const capabilities = await response.json(); // ✅ 최신 데이터 보장
```

---

### **시나리오 3: ETag를 활용한 조건부 요청**

수동으로 ETag를 관리하여 네트워크 절약.

```javascript
let cachedETag = null;
let cachedCapabilities = null;

async function getCapabilities(forceRefresh = false) {
  const headers = {};

  // 강제 새로고침이 아니고 ETag가 있으면 If-None-Match 추가
  if (!forceRefresh && cachedETag) {
    headers['If-None-Match'] = cachedETag;
  }

  const response = await fetch('/api/capabilities', { headers });

  if (response.status === 304) {
    console.log('데이터 변경 없음 - 캐시 사용');
    return cachedCapabilities; // 캐시된 데이터 반환
  }

  if (response.ok) {
    cachedETag = response.headers.get('ETag');
    cachedCapabilities = await response.json();
    console.log('새 데이터 수신 - ETag:', cachedETag);
    return cachedCapabilities;
  }

  throw new Error('Failed to fetch capabilities');
}

// 사용 예시
const data1 = await getCapabilities(); // 200 OK, ETag 저장
const data2 = await getCapabilities(); // 304 Not Modified (데이터 변경 없으면)
const data3 = await getCapabilities(true); // 200 OK (강제 새로고침)
```

---

## 🔍 응답 헤더 이해하기

### **Cache-Control: private, max-age=60**

- **`private`**: 브라우저만 캐시 가능 (CDN/프록시 캐시 불가)
- **`max-age=60`**: 60초 동안 캐시 유효

### **ETag: W/"1768903114709"**

- **`W/`**: Weak ETag (약한 ETag)
- **`"1768903114709"`**: `MAX(updated_at)` 타임스탬프 (Unix timestamp)

### **If-None-Match 요청 헤더**

브라우저가 자동으로 추가하는 헤더:

```http
GET /api/capabilities
If-None-Match: W/"1768903114709"
```

서버는 현재 ETag와 비교:
- **일치** → 304 Not Modified (body 없음)
- **불일치** → 200 OK (새 데이터 + 새 ETag)

---

## ⚠️ 주의사항

### 1. **POST/PUT 후 즉시 GET 시 캐시 무효화 필요**

```javascript
// ❌ 잘못된 예
await assignCapability(roleId, capabilityId);
const capabilities = await getCapabilities(); // 캐시된 이전 데이터 가능

// ✅ 올바른 예
await assignCapability(roleId, capabilityId);
const capabilities = await fetch('/api/capabilities', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

### 2. **304 응답도 성공**

```javascript
const response = await fetch('/api/capabilities');

// ✅ 올바른 체크
if (response.ok) { // 200-299 모두 true
  // 200, 304 모두 처리
}
```

---

## 🎯 요약

| 상황 | 방법 | 코드 |
|------|------|------|
| 일반 조회 | 그냥 fetch | `fetch('/api/capabilities')` |
| POST/PUT 후 조회 | `no-cache` 헤더 | `fetch(url, {headers: {'Cache-Control': 'no-cache'}})` |
| 강제 새로고침 | `cache: 'reload'` | `fetch(url, {cache: 'reload'})` |

**핵심**: POST/PUT 후 즉시 GET 할 때는 **`Cache-Control: no-cache`** 헤더 추가! ✨

