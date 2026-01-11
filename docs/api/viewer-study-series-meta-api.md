# Viewer Study Series Meta API

## 개요

특정 Study의 모든 Series 메타데이터를 한 번의 요청으로 조회하는 API입니다. **페이지네이션을 지원**하여 대량의 Series를 효율적으로 처리할 수 있습니다.

## 엔드포인트

```
GET /api/v1/viewer/studies/{study_uid}/series/meta
```

## 목적

- Viewer에서 특정 Study를 선택했을 때, 해당 Study의 모든 Series 메타데이터를 효율적으로 조회
- Study Description을 자동으로 포함하여 클라이언트의 추가 요청 불필요
- RBAC 기반 접근 제어를 통한 보안 강화
- **페이지네이션 지원**으로 대량의 Series 처리 최적화

## 인증

- **필수**: Bearer Token (JWT)
- **헤더**: `Authorization: Bearer <token>`

## Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| study_uid | String | ✅ | StudyInstanceUID |

## Query Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| page | Integer | ❌ | 1 | 페이지 번호 (1부터 시작) |
| page_size | Integer | ❌ | 50 | 페이지 크기 (최소: 1, 최대: 200) |

## 응답

### 성공 응답 (200 OK)

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
      "number_of_instances": 245,
      "body_part_examined": "CHEST",
      "series_date": "20240115",
      "series_time": "093012"
    },
    {
      "series_uid": "1.2.840.113619.2.55.3.604688433.1234.2",
      "study_uid": "1.2.840.113619.2.55.3.604688433.1234",
      "study_description": "Chest CT",
      "series_number": 2,
      "series_description": "Coronal",
      "modality": "CT",
      "number_of_instances": 180,
      "body_part_examined": "CHEST",
      "series_date": "20240115",
      "series_time": "093512"
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

#### 응답 필드 설명

| 필드 | 타입 | 설명 |
|------|------|------|
| study_uid | String | StudyInstanceUID |
| study_description | String (nullable) | Study 설명 |
| series | Array | Series 메타데이터 배열 (페이지네이션 적용) |
| pagination | Object | 페이지네이션 정보 |
| pagination.page | Integer | 현재 페이지 번호 |
| pagination.page_size | Integer | 페이지 크기 |
| pagination.total_items | Integer | 전체 Series 개수 |
| pagination.total_pages | Integer | 전체 페이지 수 |
| pagination.has_next | Boolean | 다음 페이지 존재 여부 |
| pagination.has_previous | Boolean | 이전 페이지 존재 여부 |

### 에러 응답

#### 401 Unauthorized
```json
{
  "error": "UNAUTHORIZED",
  "message": "Bearer token required"
}
```

#### 403 Forbidden
```json
{
  "error": "FORBIDDEN",
  "message": "Access denied to this study"
}
```

#### 404 Not Found
```json
{
  "error": "NOT_FOUND",
  "message": "No series found for this study"
}
```

#### 500 Internal Server Error
```json
{
  "error": "QIDO_ERROR",
  "message": "Failed to query series: ..."
}
```

## 사용 예시

### cURL

#### 기본 요청 (기본 페이지네이션)
```bash
curl -X GET "http://localhost:8080/api/v1/viewer/studies/1.2.840.113619.2.55.3.604688433.1234/series/meta" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 페이지네이션 지정
```bash
# 첫 번째 페이지, 10개씩
curl -X GET "http://localhost:8080/api/v1/viewer/studies/1.2.840.113619.2.55.3.604688433.1234/series/meta?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 두 번째 페이지, 10개씩
curl -X GET "http://localhost:8080/api/v1/viewer/studies/1.2.840.113619.2.55.3.604688433.1234/series/meta?page=2&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### JavaScript (Fetch API)

```javascript
const studyUid = '1.2.840.113619.2.55.3.604688433.1234';
const token = 'YOUR_JWT_TOKEN';

// 기본 요청
const response = await fetch(
  `http://localhost:8080/api/v1/viewer/studies/${studyUid}/series/meta`,
  {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const data = await response.json();
console.log(`Study: ${data.study_description}`);
console.log(`Total series: ${data.pagination.total_items}`);
console.log(`Page ${data.pagination.page} of ${data.pagination.total_pages}`);
console.log(`Showing ${data.series.length} series`);

data.series.forEach(series => {
  console.log(`- Series ${series.series_number}: ${series.series_description} (${series.number_of_instances} instances)`);
});

// 페이지네이션 사용
async function fetchAllSeries(studyUid, token) {
  let allSeries = [];
  let page = 1;
  let hasNext = true;

  while (hasNext) {
    const response = await fetch(
      `http://localhost:8080/api/v1/viewer/studies/${studyUid}/series/meta?page=${page}&page_size=50`,
      {
        method: 'GET',
        headers: { 'Authorization': `Bearer ${token}` }
      }
    );

    const data = await response.json();
    allSeries = allSeries.concat(data.series);
    hasNext = data.pagination.has_next;
    page++;
  }

  return allSeries;
}

// 사용 예시
const allSeries = await fetchAllSeries(studyUid, token);
console.log(`Fetched all ${allSeries.length} series`);
```

## 기능 상세

### 1. 인증 및 권한 검증

- JWT 토큰에서 사용자 ID 추출
- 사용자가 속한 모든 프로젝트에 대해 RBAC 평가
- Project Data Access 테이블 확인
- 하나 이상의 프로젝트에서 접근 권한이 있으면 허용

### 2. Study Description 자동 포함

- QIDO-RS를 통해 Study 메타데이터 조회
- StudyDescription (0008,1030) 추출
- 모든 Series 응답에 자동으로 포함

### 3. Series 메타데이터 조회

- QIDO-RS를 통해 해당 Study의 모든 Series 조회
- DICOMweb JSON 형식을 ViewerSeriesMeta DTO로 변환
- 필요한 DICOM 태그만 추출하여 응답 크기 최적화

### 4. 페이지네이션 처리

- **메모리 기반 페이지네이션**: QIDO에서 모든 Series를 조회한 후 메모리에서 페이지네이션 적용
- **파라미터 검증**:
  - `page`: 1 이상의 정수 (기본값: 1)
  - `page_size`: 1~200 범위로 제한 (기본값: 50)
- **페이지네이션 정보 제공**:
  - 현재 페이지, 페이지 크기
  - 전체 항목 수, 전체 페이지 수
  - 다음/이전 페이지 존재 여부

### 5. 성능 최적화

- **대량 Series 처리**: 수백 개의 Series가 있는 Study도 효율적으로 처리
- **클라이언트 제어**: 클라이언트가 필요한 만큼만 데이터 요청
- **네트워크 최적화**: 불필요한 데이터 전송 최소화

## 관련 API

- `POST /api/v1/viewer/studies/meta` - 여러 Study 메타데이터 Batch 조회
- `POST /api/v1/viewer/series/meta` - 여러 Series 메타데이터 Batch 조회

## 구현 위치

- **Controller**: `pacs-server/src/presentation/controllers/viewer_controller.rs`
- **DTO**: `pacs-server/src/application/dto/viewer_dto.rs`
- **Route**: `pacs-server/src/main.rs` (Viewer API 섹션)

