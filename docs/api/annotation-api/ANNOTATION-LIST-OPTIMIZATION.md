# 📊 Annotation 목록 최적화 전략

## 🎯 문제 상황

Series에 annotation이 많을 때 (예: 100개 이상):
- 전체 데이터 로드 시 네트워크 트래픽 증가
- 렌더링 성능 저하
- 메모리 사용량 증가
- UI 반응성 저하

---

## 💡 해결 방안

### 방안 1️⃣: 요약 목록 API (권장) ⭐⭐⭐

**개념:**
- 전체 annotation 데이터 대신 **요약 정보만** 반환
- 사용자가 선택한 annotation만 상세 데이터 로드

**요약 정보 구성:**
```json
{
  "id": 1,
  "type": "rectangle",
  "label": "Tumor",
  "color": "#FF0000",
  "created_by": "Dr. Kim",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:05:00Z",
  "version": 2
}
```

**장점:**
- ✅ 네트워크 트래픽 80-90% 감소
- ✅ 빠른 목록 로드
- ✅ 메모리 효율적
- ✅ UI 반응성 우수

**단점:**
- ❌ 상세 데이터 필요 시 추가 요청

---

### 방안 2️⃣: 페이지네이션 (보조)

**개념:**
- 한 번에 20-50개씩 로드
- 사용자가 스크롤할 때 다음 페이지 로드

**요청:**
```bash
GET /api/annotations/summary?series_instance_uid={uid}&page=1&limit=20
```

**응답:**
```json
{
  "annotations": [...],
  "total": 150,
  "page": 1,
  "limit": 20,
  "total_pages": 8
}
```

**장점:**
- ✅ 초기 로드 빠름
- ✅ 메모리 효율적
- ✅ 무한 스크롤 가능

**단점:**
- ❌ 여러 번의 요청 필요

---

### 방안 3️⃣: 필터링 (보조)

**개념:**
- 특정 타입, 사용자, 날짜 범위로 필터링
- 필요한 annotation만 로드

**요청:**
```bash
GET /api/annotations/summary?series_instance_uid={uid}&type=rectangle&created_by=1
```

---

## 🏆 권장 전략: 요약 목록 + 페이지네이션

### 아키텍처

```
┌─────────────────────────────────────────┐
│     DICOM Viewer (Series 선택)          │
└─────────────────────────────────────────┘
                    ↓
        [1] GET /annotations/summary
            ?series_instance_uid={uid}
            &page=1&limit=20
                    ↓
┌─────────────────────────────────────────┐
│  Annotation 요약 목록 (20개)             │
│  - ID, Type, Label, Color               │
│  - Created By, Created At                │
│  - Version                              │
└─────────────────────────────────────────┘
                    ↓
        사용자가 annotation 선택
                    ↓
        [2] GET /annotations/{id}
            (상세 데이터 로드)
                    ↓
┌─────────────────────────────────────────┐
│  Annotation 상세 정보                    │
│  - annotation_data (좌표, 메타데이터)    │
│  - 전체 정보                            │
└─────────────────────────────────────────┘
```

---

## 📡 API 설계

### 1. 요약 목록 조회 (새로운 엔드포인트)

```http
GET /api/annotations/summary?series_instance_uid={uid}&page=1&limit=20
```

**파라미터:**
| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `series_instance_uid` | string | ✅ | Series Instance UID |
| `page` | number | ❌ | 페이지 번호 (기본값: 1) |
| `limit` | number | ❌ | 페이지당 항목 수 (기본값: 20, 최대: 100) |
| `type` | string | ❌ | Annotation 타입 필터 |
| `created_by` | number | ❌ | 생성자 ID 필터 |
| `sort` | string | ❌ | 정렬 (created_at, updated_at) |

**응답 (200 OK):**
```json
{
  "annotations": [
    {
      "id": 1,
      "type": "rectangle",
      "label": "Tumor",
      "color": "#FF0000",
      "tool_name": "Rectangle Tool",
      "measurements": {
        "width": 100,
        "height": 100,
        "area": 10000
      },
      "created_by": 1,
      "created_by_name": "Dr. Kim",
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "1.2.3.4.5.6.7",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:05:00Z",
      "version": 2
    },
    {
      "id": 2,
      "type": "polygon",
      "label": "Lesion",
      "color": "#00FF00",
      "tool_name": "Polygon Tool",
      "measurements": {
        "area": 15000,
        "perimeter": 400
      },
      "created_by": 2,
      "created_by_name": "Dr. Lee",
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "1.2.3.4.5.6.8",
      "created_at": "2024-01-01T00:10:00Z",
      "updated_at": "2024-01-01T00:10:00Z",
      "version": 1
    }
  ],
  "pagination": {
    "total": 150,
    "page": 1,
    "limit": 20,
    "total_pages": 8
  }
}
```

**응답 헤더:**
```
ETag: "summary-1"
Last-Modified: Mon, 01 Jan 2024 00:00:00 +0000
Cache-Control: public, max-age=30
```

---

### 2. 상세 정보 조회 (기존 엔드포인트)

```http
GET /api/annotations/{annotation_id}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "project_id": 1,
  "user_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 200, 200],
    "label": "Tumor",
    "color": "#FF0000",
    "description": "Suspicious lesion",
    "measurements": {
      "width": 100,
      "height": 100,
      "area": 10000
    }
  },
  "version": 2,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:05:00Z"
}
```

---

## 💾 캐시 전략 (요약 목록)

### 캐시 구조

```javascript
const summaryCache = {
  "series:1.2.3.4.5.6:page:1": {
    version: 1,
    etag: "\"summary-1\"",
    data: [...],
    timestamp: 1704067200000,
    total: 150,
    totalPages: 8
  }
}
```

### 캐시 검증

```javascript
async function validateSummaryCache(seriesUid, page, cachedVersion) {
  const response = await fetch(
    `/api/annotations/summary?series_instance_uid=${seriesUid}&page=${page}`,
    {
      method: 'HEAD',
      headers: {
        'If-None-Match': `"${cachedVersion}"`
      }
    }
  );

  if (response.status === 304) {
    return { valid: true };
  } else if (response.status === 200) {
    return { valid: false };
  }
}
```

---

## 📊 성능 비교

### 시나리오: Series에 150개 annotation

| 방식 | 초기 로드 | 메모리 | 네트워크 | 사용성 |
|------|---------|--------|---------|--------|
| **전체 로드** | 2-3초 | 높음 | 높음 | 낮음 |
| **요약 목록** | 200-300ms | 낮음 | 낮음 | 높음 |
| **페이지네이션** | 200-300ms | 낮음 | 낮음 | 높음 |
| **요약 + 페이지** | 200-300ms | 낮음 | 낮음 | 매우 높음 |

---

## 🎨 UI 구현 예제

### 요약 목록 표시

```typescript
interface AnnotationSummary {
  id: number;
  type: string;
  label: string;
  color: string;
  created_by_name: string;
  created_at: string;
  version: number;
}

async function loadAnnotationSummaryList(seriesUid: string, page: number = 1) {
  const cacheKey = `summary:${seriesUid}:${page}`;
  
  // 캐시 확인
  const cached = summaryCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  // API 요청
  const response = await fetch(
    `/api/annotations/summary?series_instance_uid=${seriesUid}&page=${page}&limit=20`
  );

  const result = await response.json();
  
  // 캐시 저장
  summaryCache.set(cacheKey, result);
  
  return result;
}

// UI 렌더링
function renderAnnotationList(summaries: AnnotationSummary[]) {
  return summaries.map(summary => `
    <div class="annotation-item" data-id="${summary.id}">
      <div class="annotation-color" style="background: ${summary.color}"></div>
      <div class="annotation-info">
        <div class="annotation-type">${summary.type}</div>
        <div class="annotation-label">${summary.label}</div>
        <div class="annotation-meta">
          ${summary.created_by_name} • ${formatDate(summary.created_at)}
        </div>
      </div>
      <div class="annotation-version">v${summary.version}</div>
    </div>
  `).join('');
}

// 사용자가 annotation 선택
async function onAnnotationSelected(annotationId: number) {
  // 상세 정보 로드
  const detail = await fetch(`/api/annotations/${annotationId}`).then(r => r.json());
  
  // 상세 정보 표시
  displayAnnotationDetail(detail);
}
```

---

## 🔄 무한 스크롤 구현

```typescript
class AnnotationListManager {
  private currentPage = 1;
  private totalPages = 1;
  private isLoading = false;

  async loadMore(seriesUid: string) {
    if (this.isLoading || this.currentPage >= this.totalPages) {
      return;
    }

    this.isLoading = true;
    this.currentPage++;

    try {
      const result = await loadAnnotationSummaryList(seriesUid, this.currentPage);
      this.totalPages = result.pagination.total_pages;
      
      // 기존 목록에 추가
      appendToList(result.annotations);
    } finally {
      this.isLoading = false;
    }
  }

  // 스크롤 이벤트 리스너
  setupScrollListener(container: HTMLElement, seriesUid: string) {
    container.addEventListener('scroll', () => {
      const { scrollTop, scrollHeight, clientHeight } = container;
      
      // 하단에서 100px 남았을 때 다음 페이지 로드
      if (scrollHeight - scrollTop - clientHeight < 100) {
        this.loadMore(seriesUid);
      }
    });
  }
}
```

---

## 📋 구현 체크리스트

### 백엔드
- [ ] `/api/annotations/summary` 엔드포인트 구현
- [ ] 페이지네이션 로직 구현
- [ ] 필터링 로직 구현
- [ ] 정렬 로직 구현
- [ ] 캐시 헤더 설정
- [ ] 데이터베이스 쿼리 최적화 (인덱스)

### 프론트엔드
- [ ] 요약 목록 캐시 구현
- [ ] 무한 스크롤 구현
- [ ] 상세 정보 로드 구현
- [ ] 필터링 UI 구현
- [ ] 성능 모니터링

---

## 🚀 구현 우선순위

### Phase 1 (필수)
1. `/api/annotations/summary` 엔드포인트
2. 기본 페이지네이션 (limit=20)
3. 프론트엔드 요약 목록 표시

### Phase 2 (권장)
1. 무한 스크롤
2. 필터링 (type, created_by)
3. 정렬 옵션

### Phase 3 (선택)
1. 검색 기능
2. 고급 필터링
3. 성능 최적화

---

## 💡 추가 고려사항

### 1. 데이터베이스 인덱스

```sql
-- Series별 annotation 빠른 조회
CREATE INDEX idx_annotation_series ON annotation_annotation(series_instance_uid);

-- 정렬 성능 향상
CREATE INDEX idx_annotation_created_at ON annotation_annotation(created_at DESC);
CREATE INDEX idx_annotation_updated_at ON annotation_annotation(updated_at DESC);
```

### 2. 응답 시간 최적화

```sql
-- 요약 정보만 조회 (annotation_data 제외)
SELECT 
  id, type, label, color, created_by, created_at, updated_at, version
FROM annotation_annotation
WHERE series_instance_uid = $1
ORDER BY created_at DESC
LIMIT 20 OFFSET 0;
```

### 3. 캐시 만료 시간

- 요약 목록: 30초 (자주 변경될 수 있음)
- 상세 정보: 5분 (덜 변경됨)

---

## 📊 예상 효과

| 항목 | 개선 효과 |
|------|---------|
| **초기 로드 시간** | 2-3초 → 200-300ms (90% 단축) |
| **네트워크 트래픽** | 500KB → 50KB (90% 감소) |
| **메모리 사용** | 10MB → 1MB (90% 감소) |
| **UI 반응성** | 매우 향상 |
| **사용자 경험** | 매우 향상 |

---

## 🎯 최종 권장사항

**요약 목록 API + 페이지네이션 조합 사용:**

1. ✅ 초기 로드 시 요약 목록 (20개) 표시
2. ✅ 사용자가 스크롤할 때 다음 페이지 로드
3. ✅ 사용자가 annotation 선택 시 상세 정보 로드
4. ✅ 모든 단계에서 캐시 활용

이 방식이 **성능, 사용성, 구현 복잡도** 모두에서 최적입니다! 🚀

