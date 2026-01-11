# Viewer API Pagination Summary

## 개요

모든 Viewer API에 **페이지네이션 기능**이 추가되었습니다.

## 페이지네이션이 적용된 API

### 1. POST /api/v1/viewer/studies/meta
- **설명**: 여러 Study의 메타데이터를 Batch로 조회
- **페이지네이션**: ✅ 지원
- **파라미터**: `page`, `page_size` (Request Body)

### 2. POST /api/v1/viewer/series/meta
- **설명**: 여러 Series의 메타데이터를 Batch로 조회
- **페이지네이션**: ✅ 지원
- **파라미터**: `page`, `page_size` (Request Body)

### 3. GET /api/v1/viewer/studies/{study_uid}/series/meta
- **설명**: 특정 Study의 모든 Series 메타데이터 조회
- **페이지네이션**: ✅ 지원
- **파라미터**: `page`, `page_size` (Query Parameters)

## 공통 페이지네이션 사양

### Request Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 범위 | 설명 |
|---------|------|------|--------|------|------|
| `page` | Integer | ❌ | 1 | 1 이상 | 페이지 번호 (1부터 시작) |
| `page_size` | Integer | ❌ | 50 | 1~200 | 페이지 크기 |

### Response Structure

모든 API는 다음과 같은 페이지네이션 정보를 포함합니다:

```json
{
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

#### Pagination 필드 설명

| 필드 | 타입 | 설명 |
|------|------|------|
| `page` | Integer | 현재 페이지 번호 |
| `page_size` | Integer | 페이지 크기 |
| `total_items` | Integer | 전체 항목 수 |
| `total_pages` | Integer | 전체 페이지 수 |
| `has_next` | Boolean | 다음 페이지 존재 여부 |
| `has_previous` | Boolean | 이전 페이지 존재 여부 |

## 구현 방식

### 메모리 기반 페이지네이션

모든 API는 **메모리 기반 페이지네이션**을 사용합니다:

1. QIDO에서 모든 데이터 조회
2. RBAC 필터링 적용
3. 메모리에서 페이지네이션 적용
4. 페이지네이션 정보 생성

### 장점

- **정확한 total_items**: RBAC 필터링 후 실제 접근 가능한 항목 수
- **간단한 구현**: QIDO offset 처리 불필요
- **안정적**: 대부분의 Study/Series는 수백 개 이하로 메모리 부담 적음

## 사용 예시

### 1. Study Meta API

```bash
# Request
curl -X POST "http://localhost:8080/api/v1/viewer/studies/meta" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uids": ["1.2.840...", "1.2.840..."],
    "page": 1,
    "page_size": 10
  }'

# Response
{
  "studies": [...],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_items": 25,
    "total_pages": 3,
    "has_next": true,
    "has_previous": false
  }
}
```

### 2. Series Meta API

```bash
# Request
curl -X POST "http://localhost:8080/api/v1/viewer/series/meta" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "series_queries": [
      {"study_uid": "1.2.840...", "series_uid": "1.2.840...1"},
      {"study_uid": "1.2.840...", "series_uid": "1.2.840...2"}
    ],
    "page": 1,
    "page_size": 10
  }'

# Response
{
  "series": [...],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_items": 50,
    "total_pages": 5,
    "has_next": true,
    "has_previous": false
  }
}
```

### 3. Study Series Meta API

```bash
# Request
curl -X GET "http://localhost:8080/api/v1/viewer/studies/{study_uid}/series/meta?page=1&page_size=10" \
  -H "Authorization: Bearer $TOKEN"

# Response
{
  "study_uid": "1.2.840...",
  "study_description": "Chest CT",
  "series": [...],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_items": 100,
    "total_pages": 10,
    "has_next": true,
    "has_previous": false
  }
}
```

## 변경 사항

### DTO 변경

1. **ViewerStudyMetaRequest**: `page`, `page_size` 필드 추가
2. **ViewerStudyMetaResponse**: `pagination` 필드 추가
3. **ViewerSeriesMetaRequest**: `page`, `page_size` 필드 추가
4. **ViewerSeriesMetaResponse**: `pagination` 필드 추가
5. **ViewerStudySeriesMetaQuery**: 신규 생성 (Query Parameters)
6. **ViewerPaginationInfo**: 신규 생성 (공통 페이지네이션 정보)

### Controller 변경

- `get_studies_meta`: 페이지네이션 로직 추가
- `get_series_meta`: 페이지네이션 로직 추가
- `get_study_series_meta`: 페이지네이션 로직 추가

## 테스트

각 API별 테스트 스크립트가 제공됩니다:

- `test_study_meta_api.sh` (예정)
- `test_series_meta_api.sh` (예정)
- `test_study_series_meta_api.sh` (기존)

