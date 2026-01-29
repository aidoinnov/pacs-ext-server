# PACS API 캐싱 가이드

## 개요

PACS 서버는 다양한 캐싱 전략을 사용하여 성능을 최적화합니다.
이 문서는 모든 캐싱 메커니즘을 한눈에 파악하고, 각 API별 상세 가이드로 연결합니다.

---

## 📊 캐싱 전략 비교

| API | 캐싱 방식 | TTL | 주요 목적 | 성능 개선 | 상세 가이드 |
|-----|----------|-----|----------|----------|------------|
| **QIDO-RS** | Redis (서버 사이드) | 60초 | Dcm4chee 부하 절감 | 17-36% | [📖 QIDO 캐시 가이드](./qido-cache-client-guide.md) |
| **Membership** | Redis (서버 사이드) | 180초 | RBAC DB 쿼리 절감 | 3-60% | [📖 Membership 캐시 가이드](./membership-cache-guide.md) |
| **Capability** | HTTP (ETag) | 60초 | DB 조회 절감 | 63% | [📖 Capability 캐시 가이드](./capability-cache-client-guide.md) |
| **Role Assignment** | HTTP (ETag) | 1초 | 중복 요청 방지 | - | [📖 Role Assignment 캐시 가이드](./role-assignment-caching-guide.md) |
| **Subject** | HTTP (ETag) | 60초 | DB 조회 절감 | **69.7%** ⬆️ | [📖 Subject 캐시 가이드](./subject-cache-client-guide.md) |
| **Project Data** | HTTP (ETag) | 60초 | DB 조회 절감 | **71.3%** ⬆️ | [📖 Project Data 캐시 가이드](./project-data-cache-client-guide.md) |
| **Study List View** | HTTP (ETag) | 60초 | DB 조회 절감 | **47.9%** ⬆️ | [📖 Study List View 캐시 가이드](./study-list-view-cache-client-guide.md) |

---

## 🎯 캐싱 방식별 특징

### 1. **Redis 캐싱 (서버 사이드)**

**적용 API**: QIDO-RS (Studies, Series), Membership (RBAC)

**특징**:
- ✅ 서버에서 자동 처리 (클라이언트 투명)
- ✅ 모든 클라이언트가 캐시 공유
- ✅ 외부 API 호출 절감 (Dcm4chee)
- ✅ DB 쿼리 절감 (Membership)
- ⚠️ 캐시 무효화 불가 (TTL 만료 대기)

**사용 예시**:
```javascript
// 특별한 처리 불필요 - 서버가 자동으로 캐싱
const series = await fetch(`/api/me/dicom/studies/${studyUid}/series?project_id=2`);
```

**성능 개선**:
- QIDO Series: 0.22초 → 0.14초 (36% 개선)
- QIDO Studies: 0.29초 → 0.24초 (17% 개선)
- Membership: DB 쿼리 80% 절감, 응답 시간 3-60% 개선

---

### 2. **HTTP 캐싱 (ETag)**

**적용 API**: Capability, Role Assignment, Subject, Project Data, Study List View

**특징**:
- ✅ 브라우저 캐시 활용
- ✅ 304 Not Modified 응답 (네트워크 절약)
- ✅ 클라이언트 제어 가능 (`no-cache`)
- ✅ 데이터 변경 시 자동 감지

**사용 예시**:
```javascript
// 일반 조회 - 브라우저가 자동 캐싱
const capabilities = await fetch('/api/capabilities');

// 강제 새로고침 - 캐시 무시
const latest = await fetch('/api/capabilities', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

**성능 개선**:
- 60초 내 중복 요청: 0ms (브라우저 캐시)
- ETag 활용: 평균 47-71% 개선
  - Subject: 69.7% (0.051s → 0.016s)
  - Project Data: 71.3% (0.053s → 0.015s)
  - Study List View: 47.9% (0.068s → 0.035s)
  - Capability: 63% (0.150s → 0.055s)

---

## 🚀 클라이언트 개발자를 위한 빠른 가이드

### **시나리오 1: 일반적인 데이터 조회**

```javascript
// ✅ 모든 API - 그냥 fetch 사용
const data = await fetch('/api/...');
// 서버/브라우저가 자동으로 캐싱 처리
```

---

### **시나리오 2: 데이터 수정 후 즉시 조회**

```javascript
// 데이터 수정 (POST/PUT/DELETE)
await fetch('/api/roles/1/capabilities', {
  method: 'POST',
  body: JSON.stringify({ capability_id: 476 })
});

// ✅ 캐시 무시하고 최신 데이터 조회
const latest = await fetch('/api/capabilities', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

---

### **시나리오 3: 주기적인 폴링**

```javascript
// ❌ 잘못된 방법 - 매번 서버 요청
setInterval(async () => {
  const data = await fetch('/api/capabilities', {
    headers: { 'Cache-Control': 'no-cache' }
  });
}, 5000);

// ✅ 올바른 방법 - 캐시 활용
setInterval(async () => {
  const data = await fetch('/api/capabilities');
  // 데이터 변경 시에만 서버 요청 (ETag 활용)
}, 5000);
```

---

### **시나리오 4: QIDO-RS 데이터 조회**

```javascript
// ✅ 특별한 처리 불필요
const series = await fetch(
  `/api/me/dicom/studies/${studyUid}/series?project_id=${projectId}`
);
// 서버가 자동으로 Redis 캐싱 처리
```

---

## 📋 API별 상세 가이드

### 1. **QIDO-RS API 캐싱**

**캐싱 방식**: Redis (서버 사이드)  
**TTL**: 60초  
**적용 엔드포인트**:
- `GET /api/me/dicom/studies/{study_uid}/series`
- `GET /api/dicom/studies/{study_uid}/series`
- `GET /api/me/dicom/studies`

**상세 가이드**: [📖 QIDO 캐시 클라이언트 가이드](./qido-cache-client-guide.md)

**핵심 포인트**:
- ✅ 클라이언트는 특별한 처리 불필요
- ✅ 60초 이내 동일 요청은 자동으로 빠른 응답
- ⚠️ 60초 이내 데이터는 캐시됨 (최신 데이터 보장 안 됨)

---

### 2. **Capability API 캐싱**

**캐싱 방식**: HTTP (ETag)  
**TTL**: 60초  
**적용 엔드포인트**:
- `GET /api/capabilities`
- `GET /api/capabilities/{id}`
- `GET /api/capabilities/category/{category}`

**상세 가이드**: [📖 Capability 캐시 클라이언트 가이드](./capability-cache-client-guide.md)

**핵심 포인트**:
- ✅ 브라우저가 자동으로 캐싱
- ✅ POST/PUT 후 `Cache-Control: no-cache` 사용
- ✅ 304 응답 시 네트워크 절약

---

### 3. **Role Assignment API 캐싱**

**캐싱 방식**: HTTP (ETag)
**TTL**: 1초
**적용 엔드포인트**:
- `PUT /api/projects/{project_id}/users/{user_id}/role`
- `GET /api/projects/{project_id}/users`

**상세 가이드**: [📖 Role Assignment 캐시 클라이언트 가이드](./role-assignment-caching-guide.md)

**핵심 포인트**:
- ✅ 1초 내 중복 요청 방지
- ✅ PUT 후 즉시 GET 시 `no-cache` 필수
- ✅ ETag 활용으로 서버 부하 절감

---

### 4. **Subject API 캐싱** ⬆️ 신규

**캐싱 방식**: HTTP (ETag)
**TTL**: 60초
**적용 엔드포인트**:
- `GET /api/projects/{project_id}/subjects`

**상세 가이드**: [📖 Subject 캐시 클라이언트 가이드](./subject-cache-client-guide.md)

**핵심 포인트**:
- ✅ 69.7% 응답 시간 단축 (0.051s → 0.016s)
- ✅ POST/PUT/DELETE 후 `no-cache` 사용
- ✅ 프로젝트별 캐시 격리

---

### 5. **Project Data Access API 캐싱** ⬆️ 신규

**캐싱 방식**: HTTP (ETag)
**TTL**: 60초
**적용 엔드포인트**:
- `GET /api/project-data/{project_id}/studies`

**상세 가이드**: [📖 Project Data 캐시 클라이언트 가이드](./project-data-cache-client-guide.md)

**핵심 포인트**:
- ✅ 71.3% 응답 시간 단축 (0.053s → 0.015s)
- ✅ Study 할당 후 `no-cache` 사용
- ✅ 프로젝트별 캐시 격리

---

### 6. **Study List View API 캐싱** ⬆️ 신규

**캐싱 방식**: HTTP (ETag)
**TTL**: 60초
**적용 엔드포인트**:
- `GET /api/study-list-views?project_id={project_id}`

**상세 가이드**: [📖 Study List View 캐시 클라이언트 가이드](./study-list-view-cache-client-guide.md)

**핵심 포인트**:
- ✅ 47.9% 응답 시간 단축 (0.068s → 0.035s)
- ✅ View 생성/수정 후 `no-cache` 사용
- ✅ 필터 파라미터별 캐시 구분

---

## ⚠️ 공통 주의사항

### 1. **데이터 수정 후 즉시 조회**

```javascript
// ❌ 잘못된 예
await updateData();
const data = await fetch('/api/...'); // 캐시된 이전 데이터 가능

// ✅ 올바른 예
await updateData();
const data = await fetch('/api/...', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

---

### 2. **304 응답도 성공**

```javascript
const response = await fetch('/api/...');

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

### 3. **브라우저 개발자 도구 "Disable cache" 주의**

개발 중 "Disable cache" 옵션이 켜져 있으면 캐싱 동작을 확인할 수 없습니다.

---

## 🧪 캐시 동작 확인 방법

### **브라우저 개발자 도구**

1. Network 탭 열기
2. "Disable cache" **끄기**
3. API 요청 실행
4. 응답 헤더 확인:
   - `Cache-Control`
   - `ETag`
5. 재요청 시 캐시 동작 확인

### **서버 로그 확인 (QIDO-RS)**

```bash
tail -f backend.log | grep -E "(⚡|🔄)"
```

**출력 예시**:
```
[INFO] 🔄 Cache MISS - study=1.2.410..., project=2
[INFO] ⚡ Cache HIT - study=1.2.410..., project=2
```

---

## 📚 참고 자료

- [MDN - HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [MDN - ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- [MDN - Cache-Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)

---

## 🎯 요약

| 상황 | 권장 방법 |
|------|----------|
| 일반 조회 | 그냥 `fetch()` - 자동 캐싱 |
| 수정 후 조회 | `Cache-Control: no-cache` 헤더 추가 |
| 강제 새로고침 | `cache: 'reload'` 옵션 사용 |
| QIDO-RS 조회 | 특별한 처리 불필요 - 서버가 자동 처리 |

**핵심**: 대부분의 경우 **자동 캐싱**이 동작하며, **데이터 수정 후 즉시 조회**할 때만 캐시 무효화 필요! ✨

