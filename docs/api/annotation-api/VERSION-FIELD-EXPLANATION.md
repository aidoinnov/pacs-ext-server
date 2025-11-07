# 📌 AnnotationSummary의 Version 필드 용도

## 🎯 개요

`AnnotationSummary`에 포함된 `version` 필드의 정확한 용도와 활용 방법을 설명합니다.

---

## 🔍 Version 필드란?

### 정의

```rust
pub struct AnnotationSummaryDto {
    pub id: i32,
    pub annotation_type: String,
    pub label: Option<String>,
    pub color: Option<String>,
    pub tool_name: Option<String>,
    pub measurements: Option<serde_json::Value>,
    pub created_by: i32,
    pub created_by_name: String,
    pub study_instance_uid: String,
    pub series_instance_uid: String,
    pub sop_instance_uid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,  // ← 이것!
}
```

---

## 💡 Version 필드의 3가지 용도

### 1️⃣ **Optimistic Locking (낙관적 잠금)**

#### 목적
- 동시 편집 충돌 감지
- 데이터 무결성 보장

#### 동작 방식

```
사용자 A가 Annotation 조회
├─ version: 1
└─ 캐시에 저장

사용자 B가 같은 Annotation 수정
├─ version: 1 → 2로 증가
└─ 데이터베이스 업데이트

사용자 A가 수정 시도
├─ base_version: 1 (자신이 가진 버전)
├─ 서버에서 현재 version: 2 확인
├─ 버전 불일치 감지!
└─ 409 Conflict 응답
```

#### 요약 목록에서의 활용

```typescript
// 요약 목록에서 version 확인
const summaryList = await loadAnnotationSummaryList(seriesUid);

summaryList.annotations.forEach(annotation => {
  console.log(`Annotation ${annotation.id}: v${annotation.version}`);
  // Annotation 1: v1
  // Annotation 2: v2
  // Annotation 3: v1
});

// 사용자가 annotation 선택 시
async function onAnnotationSelected(annotation: AnnotationSummary) {
  // 상세 정보 로드
  const detail = await fetch(`/api/annotations/${annotation.id}`);
  const detailData = await detail.json();
  
  // 버전 확인 (요약과 상세가 같은지)
  if (annotation.version !== detailData.version) {
    console.warn('Version mismatch! Data may have changed');
  }
  
  // 수정 시 base_version 사용
  const updateResponse = await fetch(`/api/annotations/${annotation.id}`, {
    method: 'PUT',
    body: JSON.stringify({
      base_version: annotation.version,  // ← 요약에서 얻은 version 사용
      annotation_data: {...}
    })
  });
  
  if (updateResponse.status === 409) {
    // 버전 충돌! 최신 버전 조회 필요
    const latest = await fetch(`/api/annotations/${annotation.id}`);
    const latestData = await latest.json();
    console.log(`Current version: ${latestData.version}`);
  }
}
```

---

### 2️⃣ **캐시 검증 (Cache Validation)**

#### 목적
- 캐시된 데이터가 최신인지 확인
- 불필요한 데이터 재조회 방지

#### 동작 방식

```
Step 1: 요약 목록 로드 (첫 번째)
GET /api/annotations/summary?series_instance_uid={uid}&page=1
응답:
{
  "annotations": [
    { "id": 1, "version": 1, ... },
    { "id": 2, "version": 2, ... }
  ]
}
캐시에 저장 (version 포함)

Step 2: 나중에 같은 요약 목록 조회
HEAD /api/annotations/summary?series_instance_uid={uid}&page=1
If-None-Match: "summary-1"  // ETag 기반 캐시 검증

응답: 304 Not Modified
→ 캐시된 데이터 사용 (version도 그대로)

Step 3: 만약 데이터가 변경되었다면
HEAD 응답: 200 OK
ETag: "summary-2"  // 새로운 ETag
→ 전체 데이터 재조회
→ 새로운 version 정보 받음
```

#### 요약 목록에서의 활용

```typescript
interface CacheEntry {
  version: number;
  etag: string;
  data: AnnotationSummary[];
  timestamp: number;
}

const summaryCache = new Map<string, CacheEntry>();

async function loadAnnotationSummaryWithCache(seriesUid: string, page: number) {
  const cacheKey = `summary:${seriesUid}:${page}`;
  const cached = summaryCache.get(cacheKey);

  if (cached) {
    // 캐시가 있으면 HEAD 요청으로 검증
    const headResponse = await fetch(
      `/api/annotations/summary?series_instance_uid=${seriesUid}&page=${page}`,
      {
        method: 'HEAD',
        headers: {
          'If-None-Match': cached.etag
        }
      }
    );

    if (headResponse.status === 304) {
      // 캐시 유효! 캐시된 데이터 사용
      console.log('Using cached data');
      return cached.data;
    }
  }

  // 캐시 없거나 만료됨 → 새로 조회
  const response = await fetch(
    `/api/annotations/summary?series_instance_uid=${seriesUid}&page=${page}`
  );
  const result = await response.json();

  // 새로운 version 정보로 캐시 업데이트
  summaryCache.set(cacheKey, {
    version: result.annotations[0]?.version || 0,
    etag: response.headers.get('ETag') || '',
    data: result.annotations,
    timestamp: Date.now()
  });

  return result.annotations;
}
```

---

### 3️⃣ **UI 표시 (사용자 정보 제공)**

#### 목적
- 사용자에게 데이터 상태 정보 제공
- 수정 이력 추적

#### 동작 방식

```
UI에 version 표시
├─ v1: 원본 버전
├─ v2: 1번 수정됨
├─ v3: 2번 수정됨
└─ ...
```

#### 요약 목록에서의 활용

```typescript
function AnnotationSummaryItem({ annotation }: { annotation: AnnotationSummary }) {
  return (
    <div className="annotation-item">
      <div className="annotation-header">
        <span className="annotation-label">{annotation.label}</span>
        <span className="annotation-version">v{annotation.version}</span>
      </div>
      
      <div className="annotation-details">
        <div>도구: {annotation.tool_name}</div>
        <div>작성자: {annotation.created_by_name}</div>
        <div>생성: {formatDate(annotation.created_at)}</div>
        <div>수정: {formatDate(annotation.updated_at)}</div>
        
        {/* Version 정보 표시 */}
        <div className="version-info">
          <span className="version-badge">Version {annotation.version}</span>
          {annotation.version > 1 && (
            <span className="modified-badge">수정됨 ({annotation.version - 1}회)</span>
          )}
        </div>
      </div>
    </div>
  );
}
```

---

## 🔄 Version 필드의 생명주기

### 시나리오: Annotation 생성 및 수정

```
Step 1: Annotation 생성
POST /api/annotations
응답: { id: 1, version: 1, ... }

Step 2: 요약 목록 조회
GET /api/annotations/summary
응답: [{ id: 1, version: 1, ... }]
캐시: version = 1

Step 3: 사용자가 annotation 수정
PUT /api/annotations/1
요청: { base_version: 1, annotation_data: {...} }
응답: { id: 1, version: 2, ... }  ← version 증가!

Step 4: 요약 목록 재조회
GET /api/annotations/summary
응답: [{ id: 1, version: 2, ... }]  ← 새로운 version
캐시 업데이트: version = 2

Step 5: 다시 수정 시도
PUT /api/annotations/1
요청: { base_version: 1, annotation_data: {...} }  ← 구버전 사용!
응답: 409 Conflict
{
  "error": "Version Conflict",
  "current_version": 2,
  "client_version": 1
}

Step 6: 최신 버전으로 재시도
GET /api/annotations/summary  ← 최신 version 확인
응답: [{ id: 1, version: 2, ... }]

PUT /api/annotations/1
요청: { base_version: 2, annotation_data: {...} }  ← 최신 버전 사용
응답: { id: 1, version: 3, ... }  ← 성공!
```

---

## 📊 Version 필드 사용 패턴

### 패턴 1: 요약 목록에서 상세 정보로

```typescript
// 요약 목록에서 version 확인
const summary = summaryList.annotations[0];
console.log(`Summary version: ${summary.version}`);

// 상세 정보 로드
const detail = await fetch(`/api/annotations/${summary.id}`);
const detailData = await detail.json();
console.log(`Detail version: ${detailData.version}`);

// 버전 비교
if (summary.version === detailData.version) {
  console.log('✅ 데이터 일치');
} else {
  console.log('⚠️ 데이터 변경됨');
}
```

### 패턴 2: 수정 시 버전 확인

```typescript
async function updateAnnotation(
  annotationId: number,
  currentVersion: number,
  newData: any
) {
  try {
    const response = await fetch(`/api/annotations/${annotationId}`, {
      method: 'PUT',
      body: JSON.stringify({
        base_version: currentVersion,  // ← 요약에서 얻은 version
        annotation_data: newData
      })
    });

    if (response.status === 409) {
      // 버전 충돌 처리
      const conflict = await response.json();
      console.log(`
        충돌 발생!
        현재 버전: ${conflict.current_version}
        클라이언트 버전: ${conflict.client_version}
      `);
      
      // 최신 버전 조회
      const latest = await fetch(`/api/annotations/${annotationId}`);
      const latestData = await latest.json();
      
      // 사용자에게 알림
      alert(`
        다른 사용자가 이 annotation을 수정했습니다.
        최신 버전: v${latestData.version}
        다시 시도해주세요.
      `);
      
      return false;
    }

    const updated = await response.json();
    console.log(`✅ 수정 완료! 새 버전: v${updated.version}`);
    return true;
  } catch (error) {
    console.error('수정 실패:', error);
    return false;
  }
}
```

### 패턴 3: 캐시 무효화

```typescript
class AnnotationCacheManager {
  private cache = new Map<string, CacheEntry>();

  invalidateByVersion(annotationId: number, newVersion: number) {
    // 특정 버전 이상의 캐시 무효화
    for (const [key, entry] of this.cache.entries()) {
      if (entry.data.some(a => a.id === annotationId && a.version < newVersion)) {
        this.cache.delete(key);
        console.log(`Cache invalidated: ${key}`);
      }
    }
  }

  updateVersion(annotationId: number, newVersion: number) {
    // 캐시의 version 정보 업데이트
    for (const entry of this.cache.values()) {
      const annotation = entry.data.find(a => a.id === annotationId);
      if (annotation) {
        annotation.version = newVersion;
      }
    }
  }
}
```

---

## ⚠️ Version 필드 주의사항

### 1. Version은 증가만 한다

```
❌ 잘못된 예:
version: 1 → 2 → 1 (감소 불가!)

✅ 올바른 예:
version: 1 → 2 → 3 → 4 (항상 증가)
```

### 2. Version은 Annotation마다 독립적

```
Annotation 1: v1 → v2 → v3
Annotation 2: v1 → v2
Annotation 3: v1

각 annotation의 version은 독립적으로 관리됨
```

### 3. Version 0은 없다

```
❌ 잘못된 예:
version: 0 (존재하지 않음)

✅ 올바른 예:
version: 1 (최소값)
```

### 4. 요약 목록의 version은 스냅샷

```
요약 목록 조회 시점의 version
├─ 이후 다른 사용자가 수정하면
├─ 요약 목록의 version은 변경 안 됨
└─ 다시 조회해야 최신 version 확인 가능
```

---

## 🎯 결론

### Version 필드의 3가지 역할

| 역할 | 용도 | 활용 |
|------|------|------|
| **Optimistic Locking** | 동시 편집 충돌 감지 | 수정 시 `base_version` 사용 |
| **Cache Validation** | 캐시 유효성 확인 | HEAD 요청 시 version 비교 |
| **User Information** | 사용자 정보 제공 | UI에 수정 이력 표시 |

### 요약 목록에서의 활용

```typescript
// 요약 목록 로드
const summaryList = await loadAnnotationSummaryList(seriesUid);

// 각 annotation의 version 확인
summaryList.annotations.forEach(annotation => {
  // 1. 캐시 검증에 사용
  validateCache(annotation.id, annotation.version);
  
  // 2. 수정 시 base_version으로 사용
  updateAnnotation(annotation.id, annotation.version, newData);
  
  // 3. UI에 표시
  displayAnnotationWithVersion(annotation);
});
```

---

## 📚 관련 문서

- `ANNOTATION-LIST-OPTIMIZATION.md` - 목록 최적화
- `BACKEND-SUMMARY-API-IMPLEMENTATION.md` - 백엔드 구현
- `FRONTEND-IMPLEMENTATION-EXAMPLE.md` - 프론트엔드 예제

