# Pagination Implementation for Viewer Study Series Meta API

## 개요

`GET /api/v1/viewer/studies/{study_uid}/series/meta` API에 **페이지네이션 기능**을 추가했습니다.

## 페이지네이션 사양

### Query Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 범위 | 설명 |
|---------|------|------|--------|------|------|
| `page` | Integer | ❌ | 1 | 1 이상 | 페이지 번호 (1부터 시작) |
| `page_size` | Integer | ❌ | 50 | 1~200 | 페이지 크기 |

### Response 구조

```json
{
  "study_uid": "...",
  "study_description": "...",
  "series": [...],  // 페이지네이션 적용된 Series 배열
  "pagination": {
    "page": 1,
    "page_size": 50,
    "total_items": 245,
    "total_pages": 5,
    "has_next": true,
    "has_previous": false
  }
}
```

## 구현 방식

### 1. 메모리 기반 페이지네이션

```rust
// 1. QIDO에서 모든 Series 조회
let all_series: Vec<ViewerSeriesMeta> = /* ... */;

// 2. 페이지네이션 파라미터 처리
let page = query.page.unwrap_or(1).max(1);
let page_size = query.page_size.unwrap_or(50).clamp(1, 200);

// 3. offset 계산 및 슬라이싱
let offset = ((page - 1) * page_size) as usize;
let end = (offset + page_size as usize).min(total_items);
let paginated_series = all_series[offset..end].to_vec();

// 4. 페이지네이션 정보 생성
let pagination = ViewerPaginationInfo {
    page,
    page_size,
    total_items,
    total_pages: ((total_items as f64) / (page_size as f64)).ceil() as i32,
    has_next: page < total_pages,
    has_previous: page > 1,
};
```

### 2. 왜 메모리 기반인가?

- **RBAC 필터링 후 페이지네이션**: QIDO에서 모든 Series를 가져온 후, 접근 권한이 있는 Series만 필터링하고 페이지네이션 적용
- **정확한 total_items**: 필터링 후 실제 접근 가능한 Series 개수를 정확히 계산
- **간단한 구현**: QIDO에 offset을 전달하는 것보다 구현이 단순하고 안정적

### 3. 성능 고려사항

- **대부분의 Study는 Series 개수가 적음**: 일반적으로 수십~수백 개 수준
- **메모리 효율**: Series 메타데이터는 크기가 작아 메모리 부담이 적음
- **네트워크 최적화**: 클라이언트는 필요한 만큼만 데이터를 받음

## 사용 예시

### 기본 요청 (page=1, page_size=50)
```bash
curl -X GET "http://localhost:8080/api/v1/viewer/studies/{study_uid}/series/meta" \
  -H "Authorization: Bearer $TOKEN"
```

### 페이지 지정
```bash
# 첫 번째 페이지, 10개씩
curl -X GET "http://localhost:8080/api/v1/viewer/studies/{study_uid}/series/meta?page=1&page_size=10" \
  -H "Authorization: Bearer $TOKEN"

# 두 번째 페이지, 10개씩
curl -X GET "http://localhost:8080/api/v1/viewer/studies/{study_uid}/series/meta?page=2&page_size=10" \
  -H "Authorization: Bearer $TOKEN"
```

### JavaScript 예시 (모든 페이지 조회)
```javascript
async function fetchAllSeries(studyUid, token) {
  let allSeries = [];
  let page = 1;
  let hasNext = true;

  while (hasNext) {
    const response = await fetch(
      `http://localhost:8080/api/v1/viewer/studies/${studyUid}/series/meta?page=${page}&page_size=50`,
      { headers: { 'Authorization': `Bearer ${token}` } }
    );

    const data = await response.json();
    allSeries = allSeries.concat(data.series);
    hasNext = data.pagination.has_next;
    page++;
  }

  return allSeries;
}
```

## 테스트

### 테스트 스크립트 실행
```bash
./test_study_series_meta_api.sh
```

### 테스트 시나리오
1. ✅ 기본 페이지네이션 (page=1, page_size=50)
2. ✅ 커스텀 페이지네이션 (page=1, page_size=10)
3. ✅ 두 번째 페이지 (page=2, page_size=10)
4. ✅ 큰 page_size (500 → 200으로 제한)

## 변경된 파일

1. **DTO** (`pacs-server/src/application/dto/viewer_dto.rs`)
   - `ViewerStudySeriesMetaQuery` 추가
   - `ViewerPaginationInfo` 추가
   - `ViewerStudySeriesMetaResponse` 수정 (pagination 필드 추가)

2. **Controller** (`pacs-server/src/presentation/controllers/viewer_controller.rs`)
   - `get_study_series_meta` 함수에 페이지네이션 로직 추가
   - Query 파라미터 처리
   - 페이지네이션 정보 생성

3. **OpenAPI** (`pacs-server/src/presentation/openapi.rs`)
   - 새로운 DTO 스키마 등록
   - Query 파라미터 문서화

4. **문서**
   - `docs/api/viewer-study-series-meta-api.md` 업데이트
   - `IMPLEMENTATION_SUMMARY.md` 업데이트
   - `test_study_series_meta_api.sh` 업데이트

## 빌드 및 실행

```bash
# 빌드 확인
./check_build.sh

# 또는 직접 빌드
cd pacs-server
cargo build

# 서버 실행
cargo run --bin pacs_server

# API 테스트
./test_study_series_meta_api.sh
```

## 결론

✅ **페이지네이션 완전 구현**  
✅ **메모리 기반 페이지네이션으로 정확한 결과 제공**  
✅ **page_size 제한 (1~200)으로 성능 보호**  
✅ **has_next, has_previous로 UI 구현 용이**  
✅ **OpenAPI 문서화 완료**  
✅ **테스트 스크립트 제공**

