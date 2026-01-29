# 프로젝트 목록 API ETag 캐싱 - 클라이언트 사용 가이드

## 📋 목차
- [개요](#개요)
- [ETag 캐싱이란?](#etag-캐싱이란)
- [API 엔드포인트](#api-엔드포인트)
- [사용 방법](#사용-방법)
- [구현 예시](#구현-예시)
- [주의사항](#주의사항)

---

## 개요

프로젝트 목록 API는 **ETag 기반 HTTP 캐싱**을 지원합니다. 이를 통해:
- ✅ **네트워크 대역폭 절감** (변경 없을 시 데이터 전송 안 함)
- ✅ **응답 속도 향상** (50-100ms 절감)
- ✅ **서버 부하 감소** (JSON 직렬화 생략)

---

## ETag 캐싱이란?

### ETag (Entity Tag)
- 리소스의 **버전 식별자**
- 리소스가 변경되면 ETag도 변경됨
- 형식: `W/"timestamp-count"` (Weak ETag)
  - `timestamp`: 가장 최근 수정 시각 (Unix timestamp)
  - `count`: 전체 프로젝트 개수

### 동작 방식

```
1️⃣ 첫 번째 요청
   Client → Server: GET /api/projects
   Server → Client: 200 OK
                     ETag: W/"1769574311-23"
                     [프로젝트 목록 데이터]

2️⃣ 두 번째 요청 (변경 없음)
   Client → Server: GET /api/projects
                    If-None-Match: W/"1769574311-23"
   Server → Client: 304 Not Modified
                     ETag: W/"1769574311-23"
                     [데이터 없음 - 캐시 사용]

3️⃣ 세 번째 요청 (변경 있음)
   Client → Server: GET /api/projects
                    If-None-Match: W/"1769574311-23"
   Server → Client: 200 OK
                     ETag: W/"1769574312-24"
                     [새로운 프로젝트 목록 데이터]
```

---

## API 엔드포인트

### 1. 전체 프로젝트 목록 조회

```
GET /api/projects
```

**쿼리 파라미터:**
- `status`: 상태별 필터링 (예: `IN_PROGRESS`, `COMPLETED`)
- `sponsor`: 스폰서별 필터링
- `page`: 페이지 번호 (기본값: 1)
- `page_size`: 페이지 크기 (기본값: 10)
- `sort_by`: 정렬 기준 (`created_at`, `name`, `start_date`)
- `sort_order`: 정렬 순서 (`asc`, `desc`)

**응답 헤더:**
- `ETag`: 리소스 버전 식별자
- `Cache-Control`: `private, max-age=60`

### 2. 활성 프로젝트 목록 조회

```
GET /api/projects/active
```

동일한 ETag 캐싱 지원

---

## 사용 방법

### Step 1: 첫 번째 요청 (ETag 저장)

```http
GET /api/projects HTTP/1.1
Host: localhost:8080
Authorization: Bearer {token}
```

**응답:**
```http
HTTP/1.1 200 OK
ETag: W/"1769574311-23"
Cache-Control: private, max-age=60
Content-Type: application/json

{
  "projects": [...],
  "pagination": {...}
}
```

👉 **클라이언트는 ETag 값을 저장해야 함**

### Step 2: 두 번째 요청 (ETag 포함)

```http
GET /api/projects HTTP/1.1
Host: localhost:8080
Authorization: Bearer {token}
If-None-Match: W/"1769574311-23"
```

**응답 (변경 없음):**
```http
HTTP/1.1 304 Not Modified
ETag: W/"1769574311-23"
Cache-Control: private, max-age=60
```

👉 **클라이언트는 기존 캐시 데이터 사용**

**응답 (변경 있음):**
```http
HTTP/1.1 200 OK
ETag: W/"1769574312-24"
Cache-Control: private, max-age=60
Content-Type: application/json

{
  "projects": [...],
  "pagination": {...}
}
```

👉 **클라이언트는 새 데이터로 캐시 업데이트**

---

## 구현 예시

### JavaScript (Fetch API)

```javascript
class ProjectListCache {
  constructor() {
    this.etag = null;
    this.cachedData = null;
  }

  async fetchProjects() {
    const headers = {
      'Authorization': `Bearer ${token}`,
    };

    // ETag가 있으면 If-None-Match 헤더 추가
    if (this.etag) {
      headers['If-None-Match'] = this.etag;
    }

    const response = await fetch('http://localhost:8080/api/projects', {
      headers: headers
    });

    // 304 Not Modified - 캐시 사용
    if (response.status === 304) {
      console.log('✅ 캐시 사용 (변경 없음)');
      return this.cachedData;
    }

    // 200 OK - 새 데이터
    if (response.status === 200) {
      this.etag = response.headers.get('ETag');
      this.cachedData = await response.json();
      console.log('✅ 새 데이터 수신, ETag:', this.etag);
      return this.cachedData;
    }

    throw new Error(`Unexpected status: ${response.status}`);
  }
}

// 사용 예시
const cache = new ProjectListCache();

// 첫 번째 호출 - 200 OK
const projects1 = await cache.fetchProjects();

// 두 번째 호출 - 304 Not Modified (변경 없으면)
const projects2 = await cache.fetchProjects();
```


