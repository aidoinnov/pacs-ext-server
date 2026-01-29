# Study List View API - HTTP 캐싱 클라이언트 가이드

## 개요

Study List View API는 **HTTP 캐싱 (ETag)**을 활용하여 성능을 최적화합니다.
이 문서는 클라이언트 개발자가 캐싱 메커니즘을 올바르게 활용하는 방법을 설명합니다.

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **캐싱 정책**: `Cache-Control: private, max-age=60` + ETag

---

## 🎯 캐싱 전략 요약

### 핵심 개념

1. **`max-age=60`**: 60초 동안 브라우저 캐시 사용
2. **ETag**: 데이터 변경 감지 (304 Not Modified 응답)
3. **`updated_at` 기반**: DB 타임스탬프로 ETag 생성

### 장점

- ✅ **중복 요청 방지**: 60초 내 동일 요청 차단
- ✅ **서버 부하 절감**: 304 응답 시 DB 조회 생략
- ✅ **네트워크 절약**: 304 응답 시 body 없음 (47.9% 응답 시간 단축)
- ✅ **클라이언트 제어**: `Cache-Control: no-cache`로 강제 새로고침 가능

---

## 📡 캐싱이 적용된 API 엔드포인트

### 1. Study List View 목록 조회

#### 요청

```http
GET /api/study-list-views?project_id={project_id}
Authorization: Bearer {token}
```

#### 응답 (200 OK)

```http
HTTP/1.1 200 OK
Content-Type: application/json
Cache-Control: private, max-age=60
ETag: W/"1737734400"

[
  {
    "id": 1,
    "project_id": 634,
    "name": "My Study View",
    "description": "Custom study list view",
    "filter_criteria": {...},
    "created_at": "2026-01-24T10:00:00Z",
    "updated_at": "2026-01-24T10:00:00Z"
  }
]
```

#### 응답 (304 Not Modified)

```http
HTTP/1.1 304 Not Modified
Cache-Control: private, max-age=60
ETag: W/"1737734400"
```

---

## 💻 클라이언트 사용 예시

### **시나리오 1: 일반적인 Study List View 조회**

브라우저가 자동으로 캐싱 처리.

```javascript
// 첫 번째 요청 - 200 OK, ETag 저장
const response1 = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
const views1 = await response1.json();
console.log('ETag:', response1.headers.get('ETag')); // W/"1737734400"

// 60초 내 두 번째 요청 - 브라우저 캐시 사용 (0ms)
const response2 = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
// 네트워크 요청 없음 - 브라우저 캐시에서 즉시 반환

// 60초 후 세 번째 요청 - 304 Not Modified (데이터 변경 없으면)
const response3 = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
// 서버에 If-None-Match 헤더 자동 전송
// 데이터 변경 없으면 304 응답 (body 없음, 빠름)
```

---

### **시나리오 2: View 생성/수정 후 즉시 조회**

Study List View를 생성/수정한 후 즉시 최신 데이터를 조회해야 하는 경우.

#### ❌ **잘못된 방법** (캐시된 데이터 반환 가능)

```javascript
// View 생성
await fetch('/api/study-list-views', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    project_id: 634,
    name: 'New View',
    description: 'My new view'
  })
});

// 즉시 조회 - 브라우저 캐시 사용 가능 (이전 데이터)
const response = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
// ❌ 새로 생성한 View가 목록에 없을 수 있음!
```

#### ✅ **올바른 방법: `Cache-Control: no-cache` 사용** (권장)

```javascript
// View 생성
await fetch('/api/study-list-views', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    project_id: 634,
    name: 'New View',
    description: 'My new view'
  })
});

// 캐시 무시하고 최신 데이터 조회
const response = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Cache-Control': 'no-cache'  // ← 브라우저 캐시 무시
  }
});
const views = await response.json(); // ✅ 최신 데이터 보장
```

---

### **시나리오 3: ETag를 활용한 조건부 요청**

수동으로 ETag를 관리하여 네트워크 절약.

```javascript
let cachedETag = null;
let cachedViews = null;

async function getStudyListViews(projectId, forceRefresh = false) {
  const headers = {
    'Authorization': `Bearer ${token}`
  };

  // 강제 새로고침이 아니고 ETag가 있으면 If-None-Match 추가
  if (!forceRefresh && cachedETag) {
    headers['If-None-Match'] = cachedETag;
  }

  const response = await fetch(`/api/study-list-views?project_id=${projectId}`, { headers });

  if (response.status === 304) {
    console.log('데이터 변경 없음 - 캐시 사용');
    return cachedViews; // 캐시된 데이터 반환
  }

  if (response.ok) {
    cachedETag = response.headers.get('ETag');
    cachedViews = await response.json();
    console.log('새 데이터 수신 - ETag:', cachedETag);
    return cachedViews;
  }

  throw new Error('Failed to fetch study list views');
}

// 사용 예시
const data1 = await getStudyListViews(634); // 200 OK, ETag 저장
const data2 = await getStudyListViews(634); // 304 Not Modified (데이터 변경 없으면)
const data3 = await getStudyListViews(634, true); // 200 OK (강제 새로고침)
```

---

## 🔍 응답 헤더 이해하기

### **Cache-Control: private, max-age=60**

- **`private`**: 브라우저만 캐시 가능 (CDN/프록시 캐시 불가)
- **`max-age=60`**: 60초 동안 캐시 유효

### **ETag: W/"1737734400"**

- **`W/`**: Weak ETag (약한 ETag)
- **`"1737734400"`**: `MAX(updated_at)` 타임스탬프 (Unix timestamp)

### **If-None-Match 요청 헤더**

브라우저가 자동으로 추가하는 헤더:

```http
GET /api/study-list-views?project_id=634
If-None-Match: W/"1737734400"
```

서버는 현재 ETag와 비교:
- **일치** → 304 Not Modified (body 없음)
- **불일치** → 200 OK (새 데이터 + 새 ETag)

---

## ⚠️ 주의사항

### 1. **POST/PUT/DELETE 후 즉시 GET 시 캐시 무효화 필요**

```javascript
// ❌ 잘못된 방법
await createStudyListView();
const views = await fetch('/api/study-list-views?project_id=634'); // 캐시된 데이터 가능

// ✅ 올바른 방법
await createStudyListView();
const views = await fetch('/api/study-list-views?project_id=634', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

### 2. **60초 캐시 주의**

- 60초 내에는 브라우저 캐시 사용 (서버 요청 없음)
- 실시간 데이터가 필요하면 `Cache-Control: no-cache` 사용

### 3. **프로젝트별 캐시 격리**

- 각 프로젝트의 View 목록은 독립적으로 캐시됨
- 프로젝트 A의 캐시는 프로젝트 B에 영향 없음

### 4. **필터 파라미터별 캐시 구분**

- `?project_id=634`와 `?project_id=635`는 별도로 캐시됨
- 필터 조건이 다르면 다른 캐시 엔트리 생성

---

## 📊 성능 개선 효과

### **E2E 테스트 결과**

| 응답 유형 | 평균 응답 시간 | 개선율 |
|----------|--------------|--------|
| 200 OK (첫 요청) | 0.068초 | - |
| 304 Not Modified | 0.035초 | **47.9% ⬆️** |
| 브라우저 캐시 (60초 내) | 0.000초 | **100% ⬆️** |

### **네트워크 절약**

- **200 OK**: ~3KB (View 목록 데이터)
- **304 Not Modified**: ~200 bytes (헤더만)
- **브라우저 캐시**: 0 bytes (네트워크 요청 없음)

---

## 🧪 테스트 방법

### **브라우저 개발자 도구**

1. Network 탭 열기
2. "Disable cache" **끄기**
3. Study List View 목록 조회
4. 응답 헤더 확인:
   - `Cache-Control: private, max-age=60`
   - `ETag: W/"..."`
5. 60초 내 재요청 → 캐시 사용 (Network 탭에 "from disk cache" 표시)
6. 60초 후 재요청 → 서버 요청 (If-None-Match 헤더 확인)

### **curl 테스트**

```bash
# 1차 요청
curl -i -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/study-list-views?project_id=634"

# ETag 복사 (예: W/"1737734400")

# 2차 요청 (If-None-Match 포함)
curl -i -H "Authorization: Bearer $TOKEN" \
  -H "If-None-Match: W/\"1737734400\"" \
  "http://localhost:8080/api/study-list-views?project_id=634"

# 304 Not Modified 응답 확인
```

---

## 🔗 관련 문서

- [Subject Cache Guide](./subject-cache-client-guide.md) - Subject API 캐싱
- [Project Data Cache Guide](./project-data-cache-client-guide.md) - 프로젝트 데이터 캐싱
- [Capability Cache Guide](./capability-cache-client-guide.md) - Capability API 캐싱
- [Caching Guide](./caching-guide.md) - 통합 캐싱 가이드

---

## 🎯 요약

| 상황 | 권장 방법 |
|------|----------|
| 일반 조회 | 그냥 `fetch()` - 자동 캐싱 |
| View 생성/수정 후 조회 | `Cache-Control: no-cache` 헤더 추가 |
| 강제 새로고침 | `cache: 'reload'` 옵션 사용 |
| 실시간 데이터 필요 | `Cache-Control: no-cache` 헤더 추가 |

**핵심**: 대부분의 경우 **자동 캐싱**이 동작하며, **데이터 수정 후 즉시 조회**할 때만 캐시 무효화 필요! ✨
  })
});

// 캐시 무시하고 최신 데이터 조회
const response = await fetch('/api/study-list-views?project_id=634', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Cache-Control': 'no-cache'  // ← 브라우저 캐시 무시
  }
});
const views = await response.json(); // ✅ 최신 데이터 보장
```

---

### **시나리오 3: ETag를 활용한 조건부 요청**

수동으로 ETag를 관리하여 네트워크 절약.

```javascript
let cachedETag = null;
let cachedViews = null;

async function getStudyListViews(projectId, forceRefresh = false) {
  const headers = {
    'Authorization': `Bearer ${token}`
  };

  // 강제 새로고침이 아니고 ETag가 있으면 If-None-Match 추가
  if (!forceRefresh && cachedETag) {
    headers['If-None-Match'] = cachedETag;
  }

  const response = await fetch(`/api/study-list-views?project_id=${projectId}`, { headers });

