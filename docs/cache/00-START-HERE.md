# 🚀 PACS Extension Server - 캐시 문서 시작 가이드

**환영합니다!** 이 디렉토리는 PACS Extension Server의 모든 캐싱 관련 문서를 포함합니다.

---

## 📖 어디서부터 시작해야 할까요?

### **1️⃣ 프론트엔드 개발자라면?**

👉 **[통합 캐싱 가이드](./caching-guide.md)** 부터 읽으세요!

이 문서는:
- ✅ 모든 캐싱 전략을 한눈에 비교
- ✅ 클라이언트 사용 예시 (JavaScript/fetch)
- ✅ 상황별 권장 방법
- ✅ 각 API별 상세 가이드 링크

---

### **2️⃣ 특정 API 캐싱 구현을 알고 싶다면?**

#### **🆕 최근 구현된 API (2026-01-25)**

| API | 가이드 문서 | 성능 개선 |
|-----|------------|----------|
| **Subject** | [📖 Subject 캐시 가이드](./subject-cache-client-guide.md) | **69.7%** ⬆️ |
| **Project Data** | [📖 Project Data 캐시 가이드](./project-data-cache-client-guide.md) | **71.3%** ⬆️ |
| **Study List View** | [📖 Study List View 캐시 가이드](./study-list-view-cache-client-guide.md) | **47.9%** ⬆️ |

#### **기존 구현된 API**

| API | 가이드 문서 | 캐싱 방식 |
|-----|------------|----------|
| **Capability** | [📖 Capability 캐시 가이드](./capability-cache-client-guide.md) | ETag (63% 개선) |
| **Role Assignment** | [📖 Role Assignment 캐시 가이드](./role-assignment-caching-guide.md) | ETag (1초 TTL) |
| **QIDO-RS** | [📖 QIDO 캐시 가이드](./qido-cache-client-guide.md) | Redis (17-36% 개선) |
| **Membership** | [📖 Membership 캐시 가이드](./membership-cache-guide.md) | Redis (3-60% 개선) |

---

### **3️⃣ 전체 캐시 구현 현황을 보고 싶다면?**

👉 **[E2E & 캐시 현황](./E2E_AND_CACHE_STATUS.md)**

이 문서는:
- ✅ 27개 API 카테고리별 캐시 구현 현황
- ✅ E2E 테스트 커버리지 (67%)
- ✅ 캐시 권장 사항 (완료/권장/선택/불필요)

---

### **4️⃣ 캐시 구현이 충분한지 궁금하다면?**

👉 **[캐시 구현 검토](./CACHE_IMPLEMENTATION_REVIEW.md)**

이 문서는:
- ✅ 현재 캐시 구현 상태 분석
- ✅ 성능 개선 효과 측정 결과
- ✅ 추가 구현 권장 여부 (결론: 현재 충분함!)
- ✅ 캐시 가능 항목 대비 구현률: **62.5%**

---

## 🎯 빠른 참조

### **캐시 구현 현황**

- **ETag 캐시**: 8개 카테고리 (30%)
- **Redis 캐시**: 2개 카테고리 (7%)
- **총 캐시 구현**: 10개 카테고리 (37%)
- **캐시 가능 항목 대비**: 62.5% 구현 완료 ✅

### **성능 개선 효과**

| API | 개선율 |
|-----|--------|
| Project Data | **71.3%** 🥇 |
| Subject | **69.7%** 🥈 |
| Capability | **63.0%** 🥉 |
| Study List View | **47.9%** |
| QIDO Series | **36.0%** |

---

## 💡 핵심 개념

### **ETag 캐싱 (HTTP 캐싱)**
- 브라우저 캐시 활용
- 304 Not Modified 응답으로 네트워크 절약
- 클라이언트 제어 가능 (`Cache-Control: no-cache`)
- 데이터 변경 시 자동 감지

### **Redis 캐싱 (서버 사이드)**
- 서버에서 자동 처리 (클라이언트 투명)
- 모든 클라이언트가 캐시 공유
- 외부 API 호출 절감 (Dcm4chee)
- DB 쿼리 절감 (Membership)

---

## 🚀 클라이언트 개발자를 위한 핵심 팁

### **일반적인 경우**
```javascript
// ✅ 그냥 fetch 사용 - 자동 캐싱
const data = await fetch('/api/...');
```

### **데이터 수정 후 즉시 조회**
```javascript
// ✅ Cache-Control: no-cache 사용
await updateData();
const data = await fetch('/api/...', {
  headers: { 'Cache-Control': 'no-cache' }
});
```

### **QIDO-RS 조회**
```javascript
// ✅ 특별한 처리 불필요 - 서버가 자동 처리
const series = await fetch(`/api/me/dicom/studies/${studyUid}/series?project_id=${projectId}`);
```

---

## 📚 전체 문서 목록

### **필수 문서**
1. [통합 캐싱 가이드](./caching-guide.md) ⭐
2. [E2E & 캐시 현황](./E2E_AND_CACHE_STATUS.md)
3. [캐시 구현 검토](./CACHE_IMPLEMENTATION_REVIEW.md)

### **ETag 캐시 가이드 (5개)**
1. [Subject API](./subject-cache-client-guide.md) 🆕
2. [Project Data Access API](./project-data-cache-client-guide.md) 🆕
3. [Study List View API](./study-list-view-cache-client-guide.md) 🆕
4. [Capability API](./capability-cache-client-guide.md)
5. [Role Assignment API](./role-assignment-caching-guide.md)

### **Redis 캐시 가이드 (2개)**
1. [QIDO-RS API](./qido-cache-client-guide.md)
2. [Membership](./membership-cache-guide.md)

---

## 🎉 최근 업데이트 (2026-01-25)

✅ **3개 API에 ETag 캐시 구현 완료**
- Subject API (69.7% 성능 개선)
- Project Data Access API (71.3% 성능 개선)
- Study List View API (47.9% 성능 개선)

✅ **15개 E2E 테스트 추가**
- Subject: 6개 (캐시 무효화 포함)
- Project Data: 5개
- Study List View: 4개

✅ **3개 클라이언트 가이드 문서 작성**
- 실제 코드 예시 포함
- 성능 측정 결과 포함
- 브라우저/curl 테스트 방법 포함

---

**핵심**: 대부분의 경우 **자동 캐싱**이 동작하며, **데이터 수정 후 즉시 조회**할 때만 캐시 무효화 필요! ✨

