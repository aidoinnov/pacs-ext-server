# 📡 Annotation Version Check API (HEAD)

## 🎯 개요

특정 UID로 annotation 목록의 버전 정보만 조회하는 경량 API입니다.  
전체 데이터를 받지 않고 **헤더만** 확인하여 캐시 검증 및 변경 감지에 사용합니다.

---

## 🚀 API 엔드포인트

### 1. **Annotation 목록 버전 확인**

```http
HEAD /api/annotations?sop_instance_uid={uid}
HEAD /api/annotations?series_instance_uid={uid}
HEAD /api/annotations?study_instance_uid={uid}
```

#### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `sop_instance_uid` | string | ❌ | SOP Instance UID (최우선) |
| `series_instance_uid` | string | ❌ | Series Instance UID |
| `study_instance_uid` | string | ❌ | Study Instance UID |

**주의**: 최소한 하나의 UID는 필수입니다.

**우선순위**: `sop_instance_uid` > `series_instance_uid` > `study_instance_uid`

#### 응답 헤더

| 헤더 | 설명 | 예시 |
|------|------|------|
| `Last-Modified` | 가장 최근 annotation 수정 시간 (RFC 2822) | `Mon, 07 Nov 2025 12:34:56 GMT` |
| `X-List-Version` | 목록 버전 (ISO 8601) | `2025-11-07T12:34:56Z` |
| `X-Total-Count` | 총 annotation 개수 | `15` |
| `Cache-Control` | 캐시 정책 | `public, max-age=5` |

#### 응답 상태 코드

| 코드 | 설명 |
|------|------|
| `200 OK` | 버전 정보 조회 성공 (본문 없음) |
| `304 Not Modified` | 캐시 유효 (If-Modified-Since 사용 시) |
| `400 Bad Request` | UID 파라미터 누락 |
| `500 Internal Server Error` | 서버 오류 |

---

## 📖 사용 예시

### 1. **SOP Instance UID로 버전 확인**

```bash
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467.1"
```

**응답:**
```http
HTTP/1.1 200 OK
last-modified: Mon, 07 Nov 2025 12:34:56 GMT
x-list-version: 2025-11-07T12:34:56Z
x-total-count: 3
cache-control: public, max-age=5
content-length: 0
```

### 2. **Series Instance UID로 버전 확인**

```bash
curl -I "http://localhost:8080/api/annotations?series_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467"
```

### 3. **캐시 검증 (If-Modified-Since)**

```bash
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467.1" \
  -H "If-Modified-Since: Mon, 07 Nov 2025 12:34:56 GMT"
```

**응답 (변경 없음):**
```http
HTTP/1.1 304 Not Modified
last-modified: Mon, 07 Nov 2025 12:34:56 GMT
cache-control: public, max-age=5
```

**응답 (변경 있음):**
```http
HTTP/1.1 200 OK
last-modified: Mon, 07 Nov 2025 13:45:00 GMT
x-list-version: 2025-11-07T13:45:00Z
x-total-count: 5
cache-control: public, max-age=5
```

---

## 💡 활용 사례

### 1. **폴링 기반 변경 감지**

```javascript
// 5초마다 버전 확인
setInterval(async () => {
  const response = await fetch(
    '/api/annotations?sop_instance_uid=1.2.840...',
    { method: 'HEAD' }
  );
  
  const lastModified = response.headers.get('Last-Modified');
  const totalCount = response.headers.get('X-Total-Count');
  
  if (lastModified !== cachedVersion) {
    console.log('Annotations changed! Reload data.');
    // GET 요청으로 전체 데이터 다시 로드
  }
}, 5000);
```

### 2. **캐시 검증**

```javascript
// 로컬 캐시의 Last-Modified 값 사용
const cachedLastModified = localStorage.getItem('annotations_last_modified');

const response = await fetch(
  '/api/annotations?sop_instance_uid=1.2.840...',
  {
    method: 'HEAD',
    headers: {
      'If-Modified-Since': cachedLastModified
    }
  }
);

if (response.status === 304) {
  console.log('Cache is valid. Use local data.');
} else if (response.status === 200) {
  console.log('Cache is stale. Fetch new data.');
  const newLastModified = response.headers.get('Last-Modified');
  localStorage.setItem('annotations_last_modified', newLastModified);
}
```

### 3. **개수 확인**

```javascript
// 전체 데이터를 받지 않고 개수만 확인
const response = await fetch(
  '/api/annotations?series_instance_uid=1.2.840...',
  { method: 'HEAD' }
);

const totalCount = parseInt(response.headers.get('X-Total-Count'));
console.log(`Total annotations: ${totalCount}`);

if (totalCount === 0) {
  console.log('No annotations. Skip data loading.');
}
```

---

## 🔄 기존 API와의 비교

### HEAD vs GET

| 특징 | HEAD | GET |
|------|------|-----|
| **응답 본문** | ❌ 없음 | ✅ 전체 데이터 |
| **응답 크기** | ~500 bytes | 50KB - 500KB |
| **속도** | ⚡ 매우 빠름 (10-20ms) | 🐢 느림 (200-500ms) |
| **용도** | 버전 확인, 캐시 검증 | 데이터 조회 |
| **네트워크 비용** | 💰 매우 낮음 | 💰💰💰 높음 |

### 기존 HEAD API들

| API | 용도 |
|-----|------|
| `HEAD /api/annotations/{id}` | 특정 annotation의 버전 확인 |
| `HEAD /api/annotations/summary` | Series UID로 요약 목록 버전 확인 |
| `HEAD /api/annotations` | **[NEW]** UID로 annotation 목록 버전 확인 |

---

## ⚠️ 주의사항

### 1. **UID 파라미터 필수**

최소한 하나의 UID (`sop_instance_uid`, `series_instance_uid`, `study_instance_uid`)가 필요합니다.

```bash
# ❌ 잘못된 요청
curl -I "http://localhost:8080/api/annotations"

# ✅ 올바른 요청
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840..."
```

### 2. **우선순위**

여러 UID를 동시에 제공하면 우선순위에 따라 처리됩니다:

```bash
# sop_instance_uid가 우선 적용됨
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840...&series_instance_uid=1.2.840..."
```

### 3. **Last-Modified 없을 수 있음**

Annotation이 없으면 `Last-Modified` 헤더가 포함되지 않습니다:

```http
HTTP/1.1 200 OK
x-total-count: 0
cache-control: public, max-age=5
```

### 4. **캐시 시간**

`Cache-Control: public, max-age=5` (5초)로 설정되어 있습니다.  
빈번한 폴링 시 브라우저 캐시를 고려하세요.

---

## 🧪 테스트

### cURL 테스트

```bash
# 1. 기본 버전 확인
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467.1"

# 2. 캐시 검증
curl -I "http://localhost:8080/api/annotations?sop_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467.1" \
  -H "If-Modified-Since: Mon, 07 Nov 2025 12:00:00 GMT"

# 3. Series UID로 확인
curl -I "http://localhost:8080/api/annotations?series_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222.467"

# 4. Study UID로 확인
curl -I "http://localhost:8080/api/annotations?study_instance_uid=1.2.840.113619.2.55.3.604688.908.1675744222"
```

### JavaScript 테스트

```javascript
// HEAD 요청 테스트
async function checkAnnotationVersion(sopInstanceUid) {
  const response = await fetch(
    `/api/annotations?sop_instance_uid=${sopInstanceUid}`,
    { method: 'HEAD' }
  );
  
  console.log('Status:', response.status);
  console.log('Last-Modified:', response.headers.get('Last-Modified'));
  console.log('X-List-Version:', response.headers.get('X-List-Version'));
  console.log('X-Total-Count:', response.headers.get('X-Total-Count'));
}

checkAnnotationVersion('1.2.840.113619.2.55.3.604688.908.1675744222.467.1');
```

---

## 📊 성능 비교

### 네트워크 비용

| 시나리오 | HEAD | GET | 절감 |
|---------|------|-----|------|
| 버전 확인 (1회) | 500 bytes | 50 KB | **99%** |
| 폴링 (10회/분) | 5 KB/분 | 500 KB/분 | **99%** |
| 폴링 (1시간) | 300 KB | 30 MB | **99%** |

### 응답 시간

| 시나리오 | HEAD | GET |
|---------|------|-----|
| 로컬 네트워크 | 10-20ms | 200-300ms |
| 인터넷 | 50-100ms | 500-1000ms |

---

## 🎯 권장 사항

1. **폴링 주기**: 5-10초 간격 권장
2. **캐시 검증**: `If-Modified-Since` 헤더 사용
3. **개수 확인**: 데이터 로드 전 `X-Total-Count` 확인
4. **에러 처리**: 400/500 에러 시 재시도 로직 구현

---

## 📚 관련 문서

- [Annotation API 전체 명세](./FRONTEND-API-SPEC.md)
- [캐시 전략 가이드](./FRONTEND-INTEGRATION-GUIDE.md#캐시-검증)
- [HEAD 요청 활용법](./BACKEND-SUMMARY-API-IMPLEMENTATION.md#head-요청)

