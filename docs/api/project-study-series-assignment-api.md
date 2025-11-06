# Project Study/Series Assignment API

프로젝트에 DICOM Study 및 Series를 할당하고 해제하는 API 명세서입니다.

## ⚠️ 현재 구현 상태

| API | 상태 | 엔드포인트 |
|-----|------|-----------|
| Study 목록 조회 | ✅ 구현 완료 | `GET /api/project-data/{project_id}/studies` |
| Series 목록 조회 (Study별) | ✅ 구현 완료 | `GET /api/project-data/{project_id}/studies/{study_id}/series` |
| Study 할당 | ⏳ 미구현 | `POST /api/projects/{project_id}/studies/assign` |
| Study 할당 해제 | ⏳ 미구현 | `DELETE /api/projects/{project_id}/studies/{study_uid}` |
| Series 할당 | ⏳ 미구현 | `POST /api/projects/{project_id}/series/assign` |
| Series 할당 해제 | ⏳ 미구현 | `DELETE /api/projects/{project_id}/series/{series_uid}` |

**참고**: 현재는 **조회 API만 구현**되어 있습니다. 할당/해제 API는 향후 구현 예정입니다.

## 📋 목차

1. [개요](#개요)
2. [할당 목록 조회 API (구현 완료)](#할당-목록-조회-api)
3. [Study 할당 API (미구현)](#study-할당-api)
4. [Study 할당 해제 API (미구현)](#study-할당-해제-api)
5. [Series 할당 API (미구현)](#series-할당-api)
6. [Series 할당 해제 API (미구현)](#series-할당-해제-api)
7. [에러 코드](#에러-코드)

---

## 개요

### 계층적 리소스 구조

PACS Extension Server는 계층적 리소스 매핑 구조를 사용합니다:

```
Project
  └─ project_data (매핑 테이블)
       ├─ resource_level: STUDY
       │   └─ study_id → project_data_study (전역 Study)
       │
       └─ resource_level: SERIES
           ├─ study_id → project_data_study
           └─ series_id → project_data_series
```

### 리소스 레벨

- **STUDY**: Study 전체를 프로젝트에 포함
- **SERIES**: 특정 Series만 프로젝트에 포함
- **INSTANCE**: 특정 Instance만 프로젝트에 포함 (향후 지원)

### 인증

모든 API는 JWT Bearer Token 인증이 필요합니다.

```http
Authorization: Bearer <your_jwt_token>
```

---

## Study 할당 API

프로젝트에 DICOM Study를 할당합니다.

### Endpoint

```http
POST /api/projects/{project_id}/studies/assign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
  "study_description": "CT Chest with Contrast",
  "patient_id": "P001234",
  "patient_name": "홍길동",
  "patient_birth_date": "1980-05-15",
  "study_date": "2024-12-01"
}
```

#### Request Fields

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | DICOM Study Instance UID |
| `study_description` | string | ❌ | Study 설명 |
| `patient_id` | string | ❌ | 환자 ID |
| `patient_name` | string | ❌ | 환자 이름 |
| `patient_birth_date` | string (YYYY-MM-DD) | ❌ | 환자 생년월일 |
| `study_date` | string (YYYY-MM-DD) | ❌ | Study 촬영일 |

### Response

#### 성공 (201 Created)

```json
{
  "success": true,
  "message": "Study assigned to project successfully",
  "data": {
    "project_data_id": 123,
    "project_id": 1,
    "resource_level": "STUDY",
    "study": {
      "id": 456,
      "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
      "study_description": "CT Chest with Contrast",
      "patient_id": "P001234",
      "patient_name": "홍길동",
      "patient_birth_date": "1980-05-15",
      "study_date": "2024-12-01",
      "created_at": "2024-12-01T10:30:00Z",
      "updated_at": "2024-12-01T10:30:00Z"
    },
    "created_at": "2024-12-01T10:30:00Z"
  }
}
```

#### 에러 응답

**Study가 이미 할당된 경우 (409 Conflict)**

```json
{
  "success": false,
  "error": "ALREADY_ASSIGNED",
  "message": "Study is already assigned to this project"
}
```

**Study UID가 유효하지 않은 경우 (400 Bad Request)**

```json
{
  "success": false,
  "error": "INVALID_STUDY_UID",
  "message": "Invalid DICOM Study UID format"
}
```

**프로젝트를 찾을 수 없는 경우 (404 Not Found)**

```json
{
  "success": false,
  "error": "PROJECT_NOT_FOUND",
  "message": "Project not found"
}
```

### cURL 예제

```bash
curl -X POST "https://api.example.com/api/projects/1/studies/assign" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
    "study_description": "CT Chest with Contrast",
    "patient_id": "P001234",
    "patient_name": "홍길동",
    "study_date": "2024-12-01"
  }'
```

---

## Study 할당 해제 API

프로젝트에서 DICOM Study 할당을 해제합니다.

### Endpoint

```http
DELETE /api/projects/{project_id}/studies/{study_uid}
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `study_uid` | string | ✅ | DICOM Study Instance UID (URL 인코딩 필요) |

### Response

#### 성공 (200 OK)

```json
{
  "success": true,
  "message": "Study unassigned from project successfully",
  "data": {
    "project_id": 1,
    "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
    "deleted_count": 1
  }
}
```

#### 에러 응답

**Study가 할당되지 않은 경우 (404 Not Found)**

```json
{
  "success": false,
  "error": "STUDY_NOT_ASSIGNED",
  "message": "Study is not assigned to this project"
}
```

### cURL 예제

```bash
# Study UID를 URL 인코딩해야 합니다
STUDY_UID="1.2.840.113619.2.1.1.322987881.716.1234567890"
ENCODED_UID=$(echo -n "$STUDY_UID" | jq -sRr @uri)

curl -X DELETE "https://api.example.com/api/projects/1/studies/$ENCODED_UID" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## Series 할당 API

프로젝트에 DICOM Series를 할당합니다.

### Endpoint

```http
POST /api/projects/{project_id}/series/assign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
  "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
  "series_description": "Axial CT 5mm",
  "modality": "CT",
  "series_number": 1
}
```

#### Request Fields

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | DICOM Study Instance UID (부모 Study) |
| `series_uid` | string | ✅ | DICOM Series Instance UID |
| `series_description` | string | ❌ | Series 설명 |
| `modality` | string | ❌ | Modality (CT, MR, CR, DX 등) |
| `series_number` | integer | ❌ | Series 번호 |

### Response

#### 성공 (201 Created)

```json
{
  "success": true,
  "message": "Series assigned to project successfully",
  "data": {
    "project_data_id": 124,
    "project_id": 1,
    "resource_level": "SERIES",
    "study": {
      "id": 456,
      "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890"
    },
    "series": {
      "id": 789,
      "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
      "series_description": "Axial CT 5mm",
      "modality": "CT",
      "series_number": 1,
      "created_at": "2024-12-01T10:35:00Z"
    },
    "created_at": "2024-12-01T10:35:00Z"
  }
}
```

#### 에러 응답

**Series가 이미 할당된 경우 (409 Conflict)**

```json
{
  "success": false,
  "error": "ALREADY_ASSIGNED",
  "message": "Series is already assigned to this project"
}
```

**부모 Study를 찾을 수 없는 경우 (404 Not Found)**

```json
{
  "success": false,
  "error": "STUDY_NOT_FOUND",
  "message": "Parent study not found. Please assign the study first."
}
```

### cURL 예제

```bash
curl -X POST "https://api.example.com/api/projects/1/series/assign" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
    "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
    "series_description": "Axial CT 5mm",
    "modality": "CT",
    "series_number": 1
  }'
```

---

## Series 할당 해제 API

프로젝트에서 DICOM Series 할당을 해제합니다.

### Endpoint

```http
DELETE /api/projects/{project_id}/series/{series_uid}
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `series_uid` | string | ✅ | DICOM Series Instance UID (URL 인코딩 필요) |

### Response

#### 성공 (200 OK)

```json
{
  "success": true,
  "message": "Series unassigned from project successfully",
  "data": {
    "project_id": 1,
    "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
    "deleted_count": 1
  }
}
```

### cURL 예제

```bash
SERIES_UID="1.2.840.113619.2.1.2.322987881.716.1234567890.1"
ENCODED_UID=$(echo -n "$SERIES_UID" | jq -sRr @uri)

curl -X DELETE "https://api.example.com/api/projects/1/series/$ENCODED_UID" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 할당 목록 조회 API

프로젝트에 할당된 Study 및 Series 목록을 조회합니다.

### 1. Study 목록 조회

#### Endpoint

```http
GET /api/project-data/{project_id}/studies
```

#### Query Parameters

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `page` | integer | ❌ | 1 | 페이지 번호 (1부터 시작) |
| `page_size` | integer | ❌ | 20 | 페이지당 항목 수 (최대 100) |
| `patient_id` | string | ❌ | - | 환자 ID 필터 |
| `study_date_from` | string (YYYY-MM-DD) | ❌ | - | Study 시작일 필터 |
| `study_date_to` | string (YYYY-MM-DD) | ❌ | - | Study 종료일 필터 |

#### Response (200 OK)

```json
{
  "success": true,
  "studies": [
    {
      "id": 456,
      "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
      "study_description": "CT Chest with Contrast",
      "patient_id": "P001234",
      "patient_name": "홍길동",
      "patient_birth_date": "1980-05-15",
      "study_date": "2024-12-01",
      "created_at": "2024-12-01T10:30:00Z",
      "updated_at": "2024-12-01T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 45,
    "total_pages": 3
  }
}
```

#### cURL 예제

```bash
curl -X GET "https://api.example.com/api/project-data/1/studies?page=1&page_size=20" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 2. Series 목록 조회 (Study별)

#### Endpoint

```http
GET /api/project-data/{project_id}/studies/{study_id}/series
```

#### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `study_id` | integer | ✅ | Study ID |

#### Response (200 OK)

```json
{
  "success": true,
  "series": [
    {
      "study": {
        "id": 456,
        "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
        "study_description": "CT Chest with Contrast",
        "patient_id": "P001234",
        "patient_name": "홍길동",
        "patient_birth_date": "1980-05-15",
        "study_date": "2024-12-01",
        "created_at": "2024-12-01T10:30:00Z",
        "updated_at": "2024-12-01T10:30:00Z"
      },
      "series": {
        "id": 789,
        "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
        "series_description": "Axial CT 5mm",
        "modality": "CT",
        "series_number": 1,
        "created_at": "2024-12-01T10:35:00Z"
      },
      "assigned_at": "2024-12-01T10:35:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 3,
    "total_items": 3,
    "total_pages": 1
  }
}
```

#### cURL 예제

```bash
curl -X GET "https://api.example.com/api/project-data/1/studies/456/series" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 에러 코드

### HTTP 상태 코드

| 상태 코드 | 설명 |
|----------|------|
| 200 OK | 요청 성공 |
| 201 Created | 리소스 생성 성공 |
| 400 Bad Request | 잘못된 요청 (유효성 검증 실패) |
| 401 Unauthorized | 인증 실패 (토큰 없음 또는 만료) |
| 403 Forbidden | 권한 없음 |
| 404 Not Found | 리소스를 찾을 수 없음 |
| 409 Conflict | 리소스 충돌 (이미 존재) |
| 500 Internal Server Error | 서버 내부 오류 |

### 에러 응답 형식

```json
{
  "success": false,
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {
    "field": "Additional error details"
  }
}
```

### 에러 코드 목록

| 에러 코드 | HTTP 상태 | 설명 |
|----------|----------|------|
| `INVALID_STUDY_UID` | 400 | Study UID 형식이 유효하지 않음 |
| `INVALID_SERIES_UID` | 400 | Series UID 형식이 유효하지 않음 |
| `MISSING_REQUIRED_FIELD` | 400 | 필수 필드 누락 |
| `INVALID_DATE_FORMAT` | 400 | 날짜 형식이 유효하지 않음 (YYYY-MM-DD 필요) |
| `UNAUTHORIZED` | 401 | 인증 토큰이 없거나 만료됨 |
| `FORBIDDEN` | 403 | 프로젝트 접근 권한 없음 |
| `PROJECT_NOT_FOUND` | 404 | 프로젝트를 찾을 수 없음 |
| `STUDY_NOT_FOUND` | 404 | Study를 찾을 수 없음 |
| `SERIES_NOT_FOUND` | 404 | Series를 찾을 수 없음 |
| `STUDY_NOT_ASSIGNED` | 404 | Study가 프로젝트에 할당되지 않음 |
| `SERIES_NOT_ASSIGNED` | 404 | Series가 프로젝트에 할당되지 않음 |
| `ALREADY_ASSIGNED` | 409 | 리소스가 이미 할당됨 |
| `INTERNAL_ERROR` | 500 | 서버 내부 오류 |

---

## 사용 시나리오

### 시나리오 1: Study 전체를 프로젝트에 추가

```bash
# 1. Study 할당
curl -X POST "https://api.example.com/api/projects/1/studies/assign" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
    "study_description": "CT Chest",
    "patient_id": "P001234",
    "patient_name": "홍길동",
    "study_date": "2024-12-01"
  }'

# 2. 할당 확인
curl -X GET "https://api.example.com/api/project-data/1/studies" \
  -H "Authorization: Bearer $TOKEN"
```

### 시나리오 2: 특정 Series만 프로젝트에 추가

```bash
# 1. 부모 Study가 이미 존재하는지 확인
curl -X GET "https://api.example.com/api/project-data/1/studies" \
  -H "Authorization: Bearer $TOKEN"

# 2. Series 할당
curl -X POST "https://api.example.com/api/projects/1/series/assign" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.1.1.322987881.716.1234567890",
    "series_uid": "1.2.840.113619.2.1.2.322987881.716.1234567890.1",
    "series_description": "Axial CT 5mm",
    "modality": "CT",
    "series_number": 1
  }'

# 3. Series 목록 확인 (Study ID가 456이라고 가정)
curl -X GET "https://api.example.com/api/project-data/1/studies/456/series" \
  -H "Authorization: Bearer $TOKEN"
```

### 시나리오 3: Study 할당 해제

```bash
# Study UID 인코딩
STUDY_UID="1.2.840.113619.2.1.1.322987881.716.1234567890"
ENCODED_UID=$(echo -n "$STUDY_UID" | jq -sRr @uri)

# Study 할당 해제
curl -X DELETE "https://api.example.com/api/projects/1/studies/$ENCODED_UID" \
  -H "Authorization: Bearer $TOKEN"
```

---

## 주의사항

### 1. Study UID 인코딩

DICOM UID에는 `.` (점)이 포함되어 있으므로, URL 경로에 사용할 때는 반드시 URL 인코딩해야 합니다.

```bash
# 올바른 방법
ENCODED_UID=$(echo -n "$STUDY_UID" | jq -sRr @uri)
curl -X DELETE "https://api.example.com/api/projects/1/studies/$ENCODED_UID"

# 잘못된 방법 (점이 그대로 전달됨)
curl -X DELETE "https://api.example.com/api/projects/1/studies/$STUDY_UID"
```

### 2. Series 할당 시 부모 Study 필요

Series를 할당하려면 해당 Series가 속한 Study가 먼저 시스템에 존재해야 합니다. Study가 없으면 `STUDY_NOT_FOUND` 에러가 발생합니다.

### 3. 중복 할당 방지

동일한 Study 또는 Series를 같은 프로젝트에 중복 할당할 수 없습니다. 중복 할당 시도 시 `ALREADY_ASSIGNED` 에러가 발생합니다.

### 4. 권한 확인

프로젝트에 대한 적절한 권한이 있어야 Study/Series를 할당하거나 해제할 수 있습니다. 권한이 없으면 `FORBIDDEN` 에러가 발생합니다.

---

## 변경 이력

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2024-12-01 | 초기 버전 작성 |


