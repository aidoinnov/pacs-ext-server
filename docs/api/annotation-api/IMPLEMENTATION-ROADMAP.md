# 🚀 Annotation API 구현 로드맵

## 📋 개요

Phase 2-1, 2-2 완료 후 다음 단계 구현 계획입니다.

---

## 🎯 현재 상태

### ✅ 완료된 기능

| Phase | 기능 | 상태 | 커밋 |
|-------|------|------|------|
| **2-1** | Version Control (Optimistic Locking) | ✅ | `5029a6c` |
| **2-2** | HEAD 요청 (캐시 검증) | ✅ | `b1b119c` |
| **2-3** | 프론트엔드 통합 가이드 | ✅ | `fb54d34` |
| **2-4** | 요약 목록 최적화 전략 | ✅ | `bbc262b` |

---

## 🔄 다음 단계 (Phase 2-5)

### Phase 2-5: 요약 목록 API 구현

**목표:** Series 레벨에서 많은 annotation을 효율적으로 조회

**필수 포함 정보:**
- ✅ 도구 이름 (tool_name)
- ✅ 측정값 (measurements: width, height, area, perimeter)
- ✅ Annotation 작성자 이름 (created_by_name)
- ✅ Study Instance UID
- ✅ Series Instance UID
- ✅ SOP Instance UID

---

## 📊 Phase 2-5 상세 계획

### 1️⃣ 백엔드 구현 (1-2일)

#### 1.1 데이터베이스 인덱스 생성

```sql
-- Series별 빠른 조회
CREATE INDEX idx_annotation_series_uid 
ON annotation_annotation(series_instance_uid);

-- 정렬 성능 향상
CREATE INDEX idx_annotation_created_at 
ON annotation_annotation(created_at DESC);

-- 복합 인덱스 (최적화)
CREATE INDEX idx_annotation_series_created 
ON annotation_annotation(series_instance_uid, created_at DESC);
```

**예상 시간:** 30분

#### 1.2 DTO 정의

**파일:** `pacs-server/src/application/dto/annotation_dto.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub version: i32,
}

#[derive(Debug, Serialize)]
pub struct AnnotationSummaryListResponse {
    pub annotations: Vec<AnnotationSummaryDto>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub page: i32,
    pub limit: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetAnnotationSummaryQuery {
    pub series_instance_uid: String,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub annotation_type: Option<String>,
    pub created_by: Option<i32>,
    pub sort: Option<String>,
}
```

**예상 시간:** 30분

#### 1.3 리포지토리 메서드 구현

**파일:** `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`

```rust
pub async fn get_summary_list(
    &self,
    series_instance_uid: &str,
    page: i32,
    limit: i32,
    annotation_type: Option<&str>,
    created_by: Option<i32>,
    sort: Option<&str>,
) -> Result<(Vec<AnnotationSummaryDto>, i64), RepositoryError> {
    let offset = (page - 1) * limit;
    let sort_column = match sort {
        Some("updated_at") => "updated_at",
        _ => "created_at",
    };

    // 1. 전체 개수 조회
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM annotation_annotation WHERE series_instance_uid = $1"
    )
    .bind(series_instance_uid)
    .fetch_one(&self.pool)
    .await?;

    // 2. 요약 데이터 조회
    let annotations = sqlx::query_as::<_, AnnotationSummaryDto>(
        &format!(r#"
            SELECT 
                id,
                annotation_data->>'type' as annotation_type,
                annotation_data->>'label' as label,
                annotation_data->>'color' as color,
                annotation_data->>'tool_name' as tool_name,
                annotation_data->'measurements' as measurements,
                user_id,
                u.name as created_by_name,
                study_instance_uid,
                series_instance_uid,
                sop_instance_uid,
                created_at,
                updated_at,
                version
            FROM annotation_annotation a
            LEFT JOIN security_user u ON a.user_id = u.id
            WHERE a.series_instance_uid = $1
            ORDER BY a.{} DESC
            LIMIT $2 OFFSET $3
        "#, sort_column)
    )
    .bind(series_instance_uid)
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.pool)
    .await?;

    Ok((annotations, total))
}
```

**예상 시간:** 1시간

#### 1.4 Use Case 메서드 구현

**파일:** `pacs-server/src/application/use_cases/annotation_use_case.rs`

```rust
pub async fn get_annotation_summary_list(
    &self,
    series_instance_uid: &str,
    page: i32,
    limit: i32,
    annotation_type: Option<&str>,
    created_by: Option<i32>,
    sort: Option<&str>,
) -> Result<AnnotationSummaryListResponse, ServiceError> {
    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 20 } else if limit > 100 { 100 } else { limit };

    let (annotations, total) = self.repository
        .get_summary_list(
            series_instance_uid,
            page,
            limit,
            annotation_type,
            created_by,
            sort,
        )
        .await
        .map_err(|_| ServiceError::InternalServerError)?;

    let total_pages = (total as f64 / limit as f64).ceil() as i32;

    Ok(AnnotationSummaryListResponse {
        annotations,
        pagination: PaginationInfo {
            total,
            page,
            limit,
            total_pages,
        },
    })
}
```

**예상 시간:** 30분

#### 1.5 Controller 엔드포인트 구현

**파일:** `pacs-server/src/presentation/controllers/annotation_controller.rs`

```rust
pub async fn get_annotation_summary_list(
    query: web::Query<GetAnnotationSummaryQuery>,
    req: HttpRequest,
    use_case: web::Data<Arc<AnnotationUseCase<...>>>,
) -> impl Responder {
    match use_case.get_annotation_summary_list(
        &query.series_instance_uid,
        query.page.unwrap_or(1),
        query.limit.unwrap_or(20),
        query.annotation_type.as_deref(),
        query.created_by,
        query.sort.as_deref(),
    ).await {
        Ok(response) => {
            let etag = format!(
                "\"summary-{}-{}-{}\"",
                hash_string(&query.series_instance_uid),
                query.page.unwrap_or(1),
                query.limit.unwrap_or(20)
            );

            HttpResponse::Ok()
                .insert_header(("ETag", etag))
                .insert_header(("Last-Modified", Utc::now().to_rfc2822()))
                .insert_header(("Cache-Control", "public, max-age=30"))
                .json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// 라우트 등록
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/annotations")
            .route("/summary", web::get().to(get_annotation_summary_list))
            // ... 기존 라우트
    );
}
```

**예상 시간:** 1시간

#### 1.6 단위 테스트 작성

**파일:** `pacs-server/tests/annotation_summary_test.rs`

```rust
#[tokio::test]
async fn test_get_summary_list_basic() {
    // 테스트 구현
}

#[tokio::test]
async fn test_pagination_calculation() {
    // 테스트 구현
}

#[tokio::test]
async fn test_filter_by_type() {
    // 테스트 구현
}

#[tokio::test]
async fn test_required_fields_present() {
    // tool_name, measurements, created_by_name, UIDs 확인
}
```

**예상 시간:** 1시간

#### 1.7 통합 테스트 작성

**파일:** `pacs-server/tests/annotation_summary_integration_test.rs`

```rust
#[tokio::test]
async fn test_summary_list_with_many_annotations() {
    // 150개 annotation으로 테스트
}

#[tokio::test]
async fn test_summary_list_pagination() {
    // 페이지네이션 테스트
}

#[tokio::test]
async fn test_summary_list_cache_headers() {
    // ETag, Cache-Control 헤더 확인
}
```

**예상 시간:** 1시간

**백엔드 총 예상 시간:** 5-6시간

---

### 2️⃣ 프론트엔드 구현 (1-2일)

#### 2.1 캐시 매니저 구현

**파일:** `src/services/annotation-cache-manager.ts`

```typescript
interface AnnotationSummary {
  id: number;
  annotation_type: string;
  label?: string;
  color?: string;
  tool_name?: string;
  measurements?: {
    width?: number;
    height?: number;
    area?: number;
    perimeter?: number;
  };
  created_by: number;
  created_by_name: string;
  study_instance_uid: string;
  series_instance_uid: string;
  sop_instance_uid?: string;
  created_at: string;
  updated_at: string;
  version: number;
}

class AnnotationSummaryCache {
  private cache: Map<string, CacheEntry> = new Map();
  private readonly CACHE_TTL = 30 * 1000; // 30초

  set(key: string, data: AnnotationSummary[], version: number) {
    this.cache.set(key, {
      version,
      data,
      timestamp: Date.now()
    });
  }

  get(key: string): AnnotationSummary[] | null {
    const entry = this.cache.get(key);
    if (!entry) return null;
    
    if (Date.now() - entry.timestamp > this.CACHE_TTL) {
      this.cache.delete(key);
      return null;
    }
    return entry.data;
  }
}
```

**예상 시간:** 1시간

#### 2.2 API 서비스 구현

**파일:** `src/services/annotation-service.ts`

```typescript
async function loadAnnotationSummaryList(
  seriesUid: string,
  page: number = 1,
  limit: number = 20
): Promise<{
  annotations: AnnotationSummary[];
  pagination: PaginationInfo;
}> {
  const cacheKey = `summary:${seriesUid}:${page}`;
  
  // 캐시 확인
  const cached = summaryCache.get(cacheKey);
  if (cached) {
    return { annotations: cached, pagination: {...} };
  }

  // API 요청
  const response = await fetch(
    `/api/annotations/summary?series_instance_uid=${seriesUid}&page=${page}&limit=${limit}`
  );

  const result = await response.json();
  summaryCache.set(cacheKey, result.annotations, result.annotations[0]?.version);
  
  return result;
}
```

**예상 시간:** 1시간

#### 2.3 UI 컴포넌트 구현

**파일:** `src/components/AnnotationSummaryList.tsx`

```typescript
function AnnotationSummaryList({ seriesUid }: { seriesUid: string }) {
  const [annotations, setAnnotations] = useState<AnnotationSummary[]>([]);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadSummaryList();
  }, [seriesUid, page]);

  async function loadSummaryList() {
    setIsLoading(true);
    try {
      const result = await loadAnnotationSummaryList(seriesUid, page);
      setAnnotations(result.annotations);
      setTotalPages(result.pagination.total_pages);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="annotation-summary-list">
      {annotations.map(annotation => (
        <div key={annotation.id} className="annotation-item">
          <div className="annotation-color" style={{ background: annotation.color }} />
          <div className="annotation-info">
            <div className="annotation-type">{annotation.annotation_type}</div>
            <div className="annotation-label">{annotation.label}</div>
            <div className="annotation-tool">{annotation.tool_name}</div>
            <div className="annotation-measurements">
              {annotation.measurements && (
                <>
                  {annotation.measurements.width && `W: ${annotation.measurements.width}px`}
                  {annotation.measurements.height && ` H: ${annotation.measurements.height}px`}
                  {annotation.measurements.area && ` Area: ${annotation.measurements.area}px²`}
                </>
              )}
            </div>
            <div className="annotation-meta">
              {annotation.created_by_name} • {formatDate(annotation.created_at)}
            </div>
            <div className="annotation-uids">
              <small>Study: {annotation.study_instance_uid}</small>
              <small>Series: {annotation.series_instance_uid}</small>
              {annotation.sop_instance_uid && <small>SOP: {annotation.sop_instance_uid}</small>}
            </div>
          </div>
          <div className="annotation-version">v{annotation.version}</div>
        </div>
      ))}
      
      {/* 페이지네이션 */}
      <div className="pagination">
        <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page === 1}>
          이전
        </button>
        <span>{page} / {totalPages}</span>
        <button onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page === totalPages}>
          다음
        </button>
      </div>
    </div>
  );
}
```

**예상 시간:** 2시간

#### 2.4 무한 스크롤 구현

**파일:** `src/hooks/useInfiniteScroll.ts`

```typescript
function useInfiniteScroll(
  seriesUid: string,
  onLoadMore: (page: number) => Promise<void>
) {
  const [page, setPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleScroll = () => {
      const { scrollTop, scrollHeight, clientHeight } = container;
      
      if (scrollHeight - scrollTop - clientHeight < 100 && !isLoading) {
        setIsLoading(true);
        setPage(p => p + 1);
      }
    };

    container.addEventListener('scroll', handleScroll);
    return () => container.removeEventListener('scroll', handleScroll);
  }, [isLoading]);

  useEffect(() => {
    if (page > 1) {
      onLoadMore(page).finally(() => setIsLoading(false));
    }
  }, [page]);

  return { containerRef, page };
}
```

**예상 시간:** 1시간

#### 2.5 프론트엔드 테스트 작성

**파일:** `src/__tests__/annotation-summary.test.ts`

```typescript
describe('AnnotationSummaryList', () => {
  test('should load summary list', async () => {
    // 테스트 구현
  });

  test('should display required fields', () => {
    // tool_name, measurements, created_by_name, UIDs 확인
  });

  test('should handle pagination', async () => {
    // 페이지네이션 테스트
  });

  test('should cache results', async () => {
    // 캐시 테스트
  });
});
```

**예상 시간:** 1시간

**프론트엔드 총 예상 시간:** 6-7시간

---

## 📅 전체 일정

| 단계 | 작업 | 예상 시간 | 담당 |
|------|------|---------|------|
| **1** | 데이터베이스 인덱스 | 30분 | DBA/백엔드 |
| **2** | DTO 정의 | 30분 | 백엔드 |
| **3** | 리포지토리 구현 | 1시간 | 백엔드 |
| **4** | Use Case 구현 | 30분 | 백엔드 |
| **5** | Controller 구현 | 1시간 | 백엔드 |
| **6** | 백엔드 테스트 | 2시간 | 백엔드 |
| **7** | 프론트엔드 캐시 | 1시간 | 프론트엔드 |
| **8** | 프론트엔드 서비스 | 1시간 | 프론트엔드 |
| **9** | UI 컴포넌트 | 2시간 | 프론트엔드 |
| **10** | 무한 스크롤 | 1시간 | 프론트엔드 |
| **11** | 프론트엔드 테스트 | 1시간 | 프론트엔드 |
| **12** | 통합 테스트 | 1시간 | 전체 |

**총 예상 시간:** 12-13시간 (2-3일)

---

## ✅ 완료 기준

### 백엔드
- [ ] 데이터베이스 인덱스 생성
- [ ] DTO 정의 완료
- [ ] 리포지토리 메서드 구현
- [ ] Use Case 메서드 구현
- [ ] Controller 엔드포인트 구현
- [ ] 단위 테스트 작성 및 통과
- [ ] 통합 테스트 작성 및 통과
- [ ] 빌드 성공 (경고 없음)
- [ ] 필수 필드 포함 확인:
  - [ ] tool_name
  - [ ] measurements
  - [ ] created_by_name
  - [ ] study_instance_uid
  - [ ] series_instance_uid
  - [ ] sop_instance_uid

### 프론트엔드
- [ ] 캐시 매니저 구현
- [ ] API 서비스 구현
- [ ] UI 컴포넌트 구현
- [ ] 무한 스크롤 구현
- [ ] 테스트 작성 및 통과
- [ ] 필수 필드 표시 확인:
  - [ ] 도구 이름 표시
  - [ ] 측정값 표시
  - [ ] 작성자 이름 표시
  - [ ] UID 정보 표시

### 통합
- [ ] 백엔드 + 프론트엔드 통합 테스트
- [ ] 성능 테스트 (150개 annotation)
- [ ] 캐시 동작 확인
- [ ] 페이지네이션 동작 확인

---

## 🚀 다음 단계 (Phase 2-6 이후)

### Phase 2-6: WebSocket (선택사항)
- 실시간 annotation 동기화
- 여러 사용자 동시 보기

### Phase 2-7: Collaborative Lock (선택사항)
- 협업 편집 지원
- 편집자 표시

---

## 📝 참고 문서

- `ANNOTATION-LIST-OPTIMIZATION.md` - 최적화 전략
- `BACKEND-SUMMARY-API-IMPLEMENTATION.md` - 백엔드 구현 가이드
- `FRONTEND-IMPLEMENTATION-EXAMPLE.md` - 프론트엔드 예제
- `FRONTEND-API-SPEC.md` - API 명세

