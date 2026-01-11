# Viewer Session API 문서

## 개요

Viewer Session API는 여러 Study에 속한 Series를 선택하여 Viewer에서 출력하기 위한 세션 정보를 관리하는 API입니다. 
Selection ID를 통해 Viewer 상태를 재현할 수 있으며, Redis에 TTL 기반으로 저장됩니다.

**Base URL**: `http://localhost:8080/api/v1/view-selections`

---

## 1. ViewSelection 생성

여러 Study에 속한 Series를 선택하여 Viewer Selection을 생성합니다.

### 요청

**Endpoint**: `POST /api/v1/view-selections`

**Headers**:
```
Authorization: Bearer {JWT_TOKEN}
Content-Type: application/json
```

**Request Body**:
```json
{
  "series": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.123",
      "series_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.124"
    },
    {
      "study_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.125",
      "series_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.126"
    }
  ]
}
```

**필드 설명**:
- `series` (필수): 선택된 Series 목록
  - `study_uid` (필수): Study Instance UID (DICOM)
  - `series_uid` (필수): Series Instance UID (DICOM)

### 응답

**성공 (201 Created)**:
```json
{
  "selection_id": "sel_8f23ab"
}
```

**에러 응답**:
- `400 Bad Request`: 잘못된 요청 (빈 series 목록 등)
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 일부 Series에 대한 접근 권한 없음
- `500 Internal Server Error`: 서버 내부 오류

### 예시

```bash
curl -X POST http://localhost:8080/api/v1/view-selections \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "series": [
      {
        "study_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.123",
        "series_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.124"
      }
    ]
  }'
```

---

## 2. ViewSelection 조회

Selection ID로 ViewSelection을 조회합니다. Viewer 접근 시 TTL이 자동으로 연장됩니다.

### 요청

**Endpoint**: `GET /api/v1/view-selections/{selection_id}`

**Headers**:
```
Authorization: Bearer {JWT_TOKEN}
```

**Path Parameters**:
- `selection_id` (필수): Selection ID (예: `sel_8f23ab`)

### 응답

**성공 (200 OK)**:
```json
{
  "selection_id": "sel_8f23ab",
  "series": [
    {
      "study_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.123",
      "series_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.124"
    },
    {
      "study_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.125",
      "series_uid": "1.2.840.113619.2.55.3.604641477.123.1234567890.126"
    }
  ],
  "created_at": "2025-01-15T10:00:00Z",
  "expires_at": "2025-01-15T10:30:00Z",
  "user_id": 1
}
```

**필드 설명**:
- `selection_id`: Selection ID
- `series`: 선택된 Series 목록
  - `study_uid`: Study Instance UID
  - `series_uid`: Series Instance UID
- `created_at`: 생성 시각 (ISO 8601)
- `expires_at`: 만료 시각 (ISO 8601, 기본 30분)
- `user_id`: 생성한 사용자 ID

**에러 응답**:
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: Selection을 찾을 수 없음 또는 만료됨
- `500 Internal Server Error`: 서버 내부 오류

### 예시

```bash
curl -X GET http://localhost:8080/api/v1/view-selections/sel_8f23ab \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

**참고**: Viewer에서 이 API를 호출하면 TTL이 자동으로 연장됩니다.

---

## 3. ViewSelection 삭제

Selection ID로 ViewSelection을 삭제합니다.

### 요청

**Endpoint**: `DELETE /api/v1/view-selections/{selection_id}`

**Headers**:
```
Authorization: Bearer {JWT_TOKEN}
```

**Path Parameters**:
- `selection_id` (필수): Selection ID (예: `sel_8f23ab`)

### 응답

**성공 (204 No Content)**: 응답 본문 없음

**에러 응답**:
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: Selection을 찾을 수 없음
- `500 Internal Server Error`: 서버 내부 오류

### 예시

```bash
curl -X DELETE http://localhost:8080/api/v1/view-selections/sel_8f23ab \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 사용 시나리오

### 1. PACS UI에서 Series 선택 후 Viewer 열기

```javascript
// 1. 사용자가 여러 Study에서 Series 선택
const selectedSeries = [
  { study_uid: "1.2.3", series_uid: "1.2.3.4" },
  { study_uid: "1.2.5", series_uid: "1.2.5.6" }
];

// 2. Selection 생성
const response = await fetch('/api/v1/view-selections', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ series: selectedSeries })
});

const { selection_id } = await response.json();

// 3. Viewer 열기 (Selection ID를 URL 파라미터로 전달)
window.open(`/viewer/selections/${selection_id}`);
```

### 2. Viewer에서 Selection 정보 로드

```javascript
// Viewer가 Selection ID로 정보 조회
const response = await fetch(`/api/v1/view-selections/${selectionId}`, {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

const selection = await response.json();

// selection.series를 사용하여 DICOM 이미지 로드
selection.series.forEach(({ study_uid, series_uid }) => {
  loadDicomSeries(study_uid, series_uid);
});
```

---

## TTL (Time-To-Live) 정책

- **기본 TTL**: 30분 (1800초)
- **자동 연장**: Viewer에서 Selection을 조회할 때마다 TTL이 자동으로 연장됩니다
- **만료 후**: Selection이 만료되면 자동으로 삭제되며, 조회 시 404 에러가 반환됩니다

**환경 변수 설정**:
```bash
APP_REDIS__VIEW_SELECTION_TTL_SEC=1800  # 기본값: 30분
```

---

## 저장소

- **저장 위치**: Redis
- **키 형식**: `view_selection:{selection_id}`
- **데이터 형식**: JSON (ViewSelection 엔티티)
- **만료 정책**: TTL 기반 자동 삭제

---

## 인증 및 권한

- 모든 API는 JWT 토큰 인증이 필요합니다
- Selection 생성 시 선택된 모든 Series에 대한 접근 권한이 확인됩니다
- Selection 조회/삭제는 생성한 사용자만 가능합니다 (향후 구현 예정)

---

## 에러 코드

| HTTP 상태 코드 | 설명 |
|---------------|------|
| 201 | Selection 생성 성공 |
| 200 | Selection 조회 성공 |
| 204 | Selection 삭제 성공 |
| 400 | 잘못된 요청 (빈 series 목록 등) |
| 401 | 인증 실패 |
| 403 | 권한 없음 (일부 Series에 대한 접근 권한 없음) |
| 404 | Selection을 찾을 수 없음 또는 만료됨 |
| 500 | 서버 내부 오류 |

---

## OpenAPI 문서

Swagger UI에서 전체 API 문서를 확인할 수 있습니다:
```
http://localhost:8080/swagger-ui/
```

Tag: `view-selection`

