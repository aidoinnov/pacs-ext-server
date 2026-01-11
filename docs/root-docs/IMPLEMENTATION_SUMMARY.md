# Implementation Summary: Viewer APIs with Pagination

## 개요

**모든 Viewer API에 페이지네이션 기능이 추가되었습니다.**

1. ✅ `POST /api/v1/viewer/studies/meta` - Study Meta Batch API
2. ✅ `POST /api/v1/viewer/series/meta` - Series Meta Batch API
3. ✅ `GET /api/v1/viewer/studies/{study_uid}/series/meta` - Study Series Meta API

## 구현 완료 사항

### 1. DTO 수정 및 추가 (`pacs-server/src/application/dto/viewer_dto.rs`)

#### 수정된 DTO

**ViewerStudyMetaRequest** (기존 DTO 수정)
- ~~`max_count`~~ 제거
- `page`: 페이지 번호 (기본값: 1) 추가
- `page_size`: 페이지 크기 (기본값: 50, 최대: 200) 추가

**ViewerStudyMetaResponse** (기존 DTO 수정)
- `studies`: Study 메타데이터 배열
- `pagination`: 페이지네이션 정보 추가

**ViewerSeriesMetaRequest** (기존 DTO 수정)
- ~~`max_count`~~ 제거
- `page`: 페이지 번호 (기본값: 1) 추가
- `page_size`: 페이지 크기 (기본값: 50, 최대: 200) 추가

**ViewerSeriesMetaResponse** (기존 DTO 수정)
- `series`: Series 메타데이터 배열
- `pagination`: 페이지네이션 정보 추가

#### 새로운 DTO

**ViewerStudySeriesMetaQuery** (신규)
- `page`: 페이지 번호 (기본값: 1)
- `page_size`: 페이지 크기 (기본값: 50, 최대: 200)

**ViewerStudySeriesMetaResponse** (신규)
- `study_uid`: StudyInstanceUID
- `study_description`: Study 설명 (자동 포함)
- `series`: Series 메타데이터 배열 (페이지네이션 적용)
- `pagination`: 페이지네이션 정보

**ViewerPaginationInfo** (신규, 공통)
- `page`: 현재 페이지
- `page_size`: 페이지 크기
- `total_items`: 전체 항목 수
- `total_pages`: 전체 페이지 수
- `has_next`: 다음 페이지 존재 여부
- `has_previous`: 이전 페이지 존재 여부

### 2. Controller 수정 (`pacs-server/src/presentation/controllers/viewer_controller.rs`)

#### 수정된 엔드포인트

**1. `get_studies_meta` (POST /api/v1/viewer/studies/meta)**
- Request Body에서 `page`, `page_size` 파라미터 처리
- ~~`max_count`~~ 제거
- 모든 Study 조회 후 메모리에서 페이지네이션 적용
- `ViewerPaginationInfo` 생성 및 응답에 포함

**2. `get_series_meta` (POST /api/v1/viewer/series/meta)**
- Request Body에서 `page`, `page_size` 파라미터 처리
- ~~`max_count`~~ 제거
- 모든 Series 조회 후 메모리에서 페이지네이션 적용
- `ViewerPaginationInfo` 생성 및 응답에 포함

**3. `get_study_series_meta` (GET /api/v1/viewer/studies/{study_uid}/series/meta)** (신규)
- Query Parameters에서 `page`, `page_size` 파라미터 처리
- Study의 모든 Series 조회 후 메모리에서 페이지네이션 적용
- Study Description 자동 포함
- `ViewerPaginationInfo` 생성 및 응답에 포함

#### 공통 페이지네이션 로직

모든 엔드포인트는 동일한 페이지네이션 로직을 사용합니다:

```rust
// 1. 파라미터 처리
let page = request.page.unwrap_or(1).max(1);
let page_size = request.page_size.unwrap_or(50).clamp(1, 200);

// 2. 모든 데이터 조회 및 RBAC 필터링
let all_items: Vec<_> = /* ... */;
let total_items = all_items.len();

// 3. 페이지네이션 적용
let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as i32;
let offset = ((page - 1) * page_size) as usize;
let end = (offset + page_size as usize).min(total_items);
let paginated_items = all_items[offset..end].to_vec();

// 4. 페이지네이션 정보 생성
let pagination = ViewerPaginationInfo {
    page,
    page_size,
    total_items: total_items as i32,
    total_pages,
    has_next: page < total_pages,
    has_previous: page > 1,
};
```

### 3. 라우팅 등록 (`pacs-server/src/main.rs`)

```rust
// Study의 모든 Series Meta API
cfg.service(
    web::resource("/v1/viewer/studies/{study_uid}/series/meta")
        .route(web::get().to(
            presentation::controllers::viewer_controller::get_study_series_meta
        ))
);
```

### 4. OpenAPI 문서화 (`pacs-server/src/presentation/openapi.rs`)

#### Paths 추가
- `get_study_series_meta` 엔드포인트 등록

#### Schemas 추가
- `ViewerStudySeriesMetaQuery`: 쿼리 파라미터
- `ViewerStudySeriesMetaResponse`: 응답 DTO
- `ViewerPaginationInfo`: 페이지네이션 정보
- `SeriesQuery`: Study-Series 쌍

#### utoipa 어노테이션
- Path: `/api/v1/viewer/studies/{study_uid}/series/meta`
- Method: GET
- Tag: viewer
- Path Parameters: `study_uid`
- Query Parameters: `page`, `page_size`
- Responses: 200, 400, 401, 403, 404

### 5. 테스트 스크립트 (`test_study_series_meta_api.sh`)

- 로그인하여 JWT 토큰 획득
- 새로운 API 엔드포인트 호출
- 응답 검증 및 출력

### 6. API 문서 (`docs/api/viewer-study-series-meta-api.md`)

- API 개요 및 목적
- 엔드포인트 상세 설명
- 요청/응답 예시
- 에러 응답 형식
- 사용 예시 (cURL, JavaScript)
- 기능 상세 설명

## API 엔드포인트

```
GET /api/v1/viewer/studies/{study_uid}/series/meta
```

### Path Parameters
- `study_uid`: StudyInstanceUID (필수)

### Query Parameters
- `page`: 페이지 번호 (기본값: 1, 1부터 시작)
- `page_size`: 페이지 크기 (기본값: 50, 최소: 1, 최대: 200)

### Headers
- `Authorization: Bearer <JWT_TOKEN>` (필수)

### Response (200 OK)
```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
  "study_description": "Chest CT",
  "series": [
    {
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.1",
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "study_description": "Chest CT",
      "series_number": 1,
      "series_description": "Axial",
      "modality": "CT",
      "number_of_instances": 245
    }
  ],
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

## 기존 API와의 차이점

### 기존 Batch API (`POST /api/v1/viewer/series/meta`)
- 여러 Study의 특정 Series들을 조회
- Request Body에 Study-Series 쌍 배열 필요
- 클라이언트가 어떤 Series를 조회할지 명시해야 함
- 페이지네이션 없음 (max_count로 제한)

### 새로운 API (`GET /api/v1/viewer/studies/{study_uid}/series/meta`)
- 특정 Study의 **모든** Series를 조회
- Path Parameter로 Study UID만 전달
- 서버가 자동으로 모든 Series 조회
- Study Description 자동 포함
- **페이지네이션 지원** (page, page_size)
- 대량의 Series를 효율적으로 처리

## 사용 시나리오

1. **Viewer에서 Study 선택 시**
   - 사용자가 Study를 클릭
   - 해당 Study의 모든 Series 목록 표시
   - 각 Series의 썸네일 및 메타데이터 표시

2. **Study 상세 정보 페이지**
   - Study 정보와 함께 모든 Series 목록 표시
   - Series별 Instance 개수, Modality 등 표시

3. **Series 선택 UI**
   - 사용자가 특정 Series를 선택하기 위한 목록 제공
   - Series Number, Description으로 정렬 가능

## 빌드 및 테스트

### 빌드
```bash
cd pacs-server
cargo build
```

### 서버 실행
```bash
cargo run --bin pacs_server
```

### API 테스트
```bash
./test_study_series_meta_api.sh
```

## 주요 기능

### ✅ 페이지네이션 (모든 API 공통)
- **메모리 기반 페이지네이션**: QIDO에서 모든 데이터를 조회한 후 메모리에서 페이지네이션 적용
- **파라미터 검증**: page (1 이상), page_size (1~200)
- **페이지네이션 정보**: total_items, total_pages, has_next, has_previous
- **일관된 응답 구조**: 모든 API가 동일한 `ViewerPaginationInfo` 사용

### ✅ RBAC 기반 보안 (모든 API 공통)
- 사용자가 속한 프로젝트의 접근 권한 확인
- Project Data Access 테이블 검증
- 접근 불가능한 데이터는 자동 필터링

### ✅ Batch API 최적화
- **Study Meta API**: 여러 Study를 한 번에 조회
- **Series Meta API**: 여러 Series를 한 번에 조회
- **Study Series Meta API**: 특정 Study의 모든 Series 조회 + Study Description 자동 포함

### ✅ 효율적인 조회
- 한 번의 요청으로 필요한 메타데이터 획득
- 페이지네이션으로 네트워크 최적화
- 클라이언트가 필요한 만큼만 데이터 요청

## 파일 변경 사항

### 코드 변경
1. `pacs-server/src/application/dto/viewer_dto.rs` - DTO 수정 및 추가 (페이지네이션 포함)
2. `pacs-server/src/presentation/controllers/viewer_controller.rs` - 3개 Controller 모두 페이지네이션 로직 추가
3. `pacs-server/src/main.rs` - 라우팅 등록 (기존)
4. `pacs-server/src/presentation/openapi.rs` - OpenAPI 문서화 (페이지네이션 스키마)

### 문서 및 테스트
5. `docs/api/PAGINATION_SUMMARY.md` - 페이지네이션 종합 문서 (신규)
6. `docs/api/viewer-study-series-meta-api.md` - Study Series Meta API 문서 (페이지네이션 설명 포함)
7. `test_study_series_meta_api.sh` - 테스트 스크립트 (페이지네이션 테스트 포함)
8. `IMPLEMENTATION_SUMMARY.md` - 구현 요약 (업데이트)
9. `PAGINATION_IMPLEMENTATION.md` - 페이지네이션 구현 상세 (기존)

