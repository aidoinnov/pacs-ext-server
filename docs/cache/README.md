# PACS Extension Server - 캐시 구현 문서 모음

이 디렉토리는 PACS Extension Server의 모든 캐싱 관련 문서를 포함합니다.

---

## 📚 문서 목록

### 🎯 **시작하기**

1. **[통합 캐싱 가이드](./caching-guide.md)** ⭐ **여기서 시작하세요!**
   - 모든 캐싱 전략 비교
   - 클라이언트 개발자를 위한 빠른 가이드
   - 각 API별 상세 가이드 링크

2. **[E2E & 캐시 현황](./E2E_AND_CACHE_STATUS.md)**
   - 전체 API 카테고리별 캐시 구현 현황
   - E2E 테스트 커버리지
   - 캐시 권장 사항

3. **[캐시 구현 검토](./CACHE_IMPLEMENTATION_REVIEW.md)**
   - 현재 캐시 구현 상태 분석
   - 성능 개선 효과 측정
   - 추가 구현 권장 여부

---

## 📖 클라이언트 가이드 문서

### **ETag 캐시 (HTTP 캐싱)**

#### 🆕 **최근 구현 (2026-01-25)**

1. **[Subject API 캐시 가이드](./subject-cache-client-guide.md)**
   - 엔드포인트: `GET /api/projects/{id}/subjects`
   - 성능 개선: **69.7%** (0.051s → 0.016s)
   - TTL: 60초

2. **[Project Data Access API 캐시 가이드](./project-data-cache-client-guide.md)**
   - 엔드포인트: `GET /api/project-data/{id}/studies`
   - 성능 개선: **71.3%** (0.053s → 0.015s)
   - TTL: 60초

3. **[Study List View API 캐시 가이드](./study-list-view-cache-client-guide.md)**
   - 엔드포인트: `GET /api/study-list-views`
   - 성능 개선: **47.9%** (0.068s → 0.035s)
   - TTL: 60초

#### **기존 구현**

4. **[Capability API 캐시 가이드](./capability-cache-client-guide.md)**
   - 엔드포인트: `GET /api/capabilities`, `GET /api/capabilities/{id}`, etc.
   - 성능 개선: **63%** (0.150s → 0.055s)
   - TTL: 60초

5. **[Role Assignment API 캐시 가이드](./role-assignment-caching-guide.md)**
   - 엔드포인트: `PUT /api/projects/{id}/users/{user_id}/role`, `GET /api/projects/{id}/users`
   - TTL: 1초 (중복 요청 방지)

---

### **Redis 캐시 (서버 사이드 캐싱)**

6. **[QIDO-RS API 캐시 가이드](./qido-cache-client-guide.md)**
   - 엔드포인트: `GET /api/me/dicom/studies/{study_uid}/series`, etc.
   - 성능 개선: **17-36%**
   - TTL: 60초
   - Dcm4chee 서버 부하 절감

7. **[Membership 캐시 가이드](./membership-cache-guide.md)**
   - RBAC 멤버십 확인 캐싱
   - 성능 개선: **3-60%**
   - TTL: 180초
   - DB 쿼리 80% 절감

---

## 📊 캐시 구현 현황 요약

### **ETag 캐시 (10개 카테고리 - 37%)** ⬆️
- ✅ Project Management
- ✅ Capability Management
- ✅ Role-Capability Matrix
- ✅ Role Assignment
- ✅ Role-Permission Matrix
- ✅ Subject
- ✅ Study Management (Study List View)
- ✅ Project Data Access
- ✅ **Permission Management** 🆕
- ✅ **Role Management** 🆕

### **Redis 캐시 (2개 카테고리 - 7%)**
- ✅ QIDO-RS (Studies, Series)
- ✅ Membership Check

### **총 캐시 구현: 12개 카테고리 (44%)** ⬆️
### **캐시 가능 항목 대비 구현률: 75%** ✅ ⬆️

---

## 🎯 빠른 참조

### **상황별 권장 방법**

| 상황 | 권장 방법 |
|------|----------|
| 일반 조회 | 그냥 `fetch()` - 자동 캐싱 |
| 데이터 수정 후 조회 | `Cache-Control: no-cache` 헤더 추가 |
| 강제 새로고침 | `cache: 'reload'` 옵션 사용 |
| QIDO-RS 조회 | 특별한 처리 불필요 - 서버가 자동 처리 |

### **성능 개선 효과**

| API | 캐싱 방식 | 성능 개선 |
|-----|----------|----------|
| Subject | ETag | **69.7%** |
| Project Data | ETag | **71.3%** |
| Study List View | ETag | **47.9%** |
| Capability | ETag | **63%** |
| Role (Project) | ETag | **45.3%** 🆕 |
| Role (Global) | ETag | **39.2%** 🆕 |
| QIDO Series | Redis | **36%** |
| QIDO Studies | Redis | **17%** |
| Membership | Redis | **3-60%** |

---

## 🚀 클라이언트 개발자를 위한 팁

### **1. 데이터 수정 후 즉시 조회**

```javascript
// ❌ 잘못된 방법
await updateData();
const data = await fetch('/api/...'); // 캐시된 이전 데이터 가능

// ✅ 올바른 방법
await updateData();
const data = await fetch('/api/...', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

### **2. ETag를 활용한 조건부 요청**

```javascript
let cachedETag = null;
let cachedData = null;

async function getData(forceRefresh = false) {
  const headers = {};
  
  if (!forceRefresh && cachedETag) {
    headers['If-None-Match'] = cachedETag;
  }
  
  const response = await fetch('/api/...', { headers });
  
  if (response.status === 304) {
    return cachedData; // 캐시된 데이터 반환
  }
  
  if (response.ok) {
    cachedETag = response.headers.get('ETag');
    cachedData = await response.json();
    return cachedData;
  }
}
```

---

## 📝 문서 작성 이력

- **2026-01-25 (최신)**: Permission Management & Role Management API 캐시 구현 완료 🎉
  - Permission Management API 캐시 가이드 추가
  - Role Management API 캐시 가이드 추가
  - 총 E2E 테스트: 73개 → 88개 (+15개)
  - 전체 캐시 구현률: 62.5% → 75%
- **2026-01-25**: Subject, Project Data Access, Study List View API 캐시 가이드 추가
- **2026-01-24**: Role-Permission Matrix API 캐시 구현
- **2026-01-XX**: 초기 캐시 가이드 문서 작성 (QIDO-RS, Membership, Capability, Role Assignment)

---

## 🔗 관련 문서

- [API 문서 메인](../api/README.md)
- [E2E 테스트 가이드](../../pacs-server/e2e/README.md)

---

**핵심**: 대부분의 경우 **자동 캐싱**이 동작하며, **데이터 수정 후 즉시 조회**할 때만 캐시 무효화 필요! ✨

