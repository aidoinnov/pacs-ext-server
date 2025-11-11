# 프로젝트 데이터 할당/해제 API 문서

## 개요

프로젝트에 DICOM 데이터(Study, Series)를 할당하거나 해제하는 API입니다.

**Base URL**: `http://localhost:8080/api`

---

## 📋 목차

1. [Study 할당](#1-study-할당)
2. [Series 할당](#2-series-할당)
3. [Study 할당 해제](#3-study-할당-해제)
4. [Series 할당 해제](#4-series-할당-해제)
5. [프로젝트 Study 목록 조회](#5-프로젝트-study-목록-조회)
6. [프로젝트 Series 목록 조회](#6-프로젝트-series-목록-조회)
7. [프로젝트 Instance 목록 조회](#7-프로젝트-instance-목록-조회)

---

## 1. Study 할당

프로젝트에 Study를 할당합니다.

### Endpoint

```
POST /projects/{project_id}/studies/assign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "study_description": "Brain MRI Study",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1990-01-01",
  "study_date": "2025-11-10"
}
```

#### Request Body Fields

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID (DICOM 표준) |
| `study_description` | string | ❌ | Study 설명 |
| `patient_id` | string | ❌ | 환자 ID |
| `patient_name` | string | ❌ | 환자 이름 |
| `patient_birth_date` | string | ❌ | 환자 생년월일 (YYYY-MM-DD) |
| `study_date` | string | ❌ | Study 날짜 (YYYY-MM-DD) |

### Response

**성공 (200 OK)**

```json
{
  "id": 123,
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "study_description": "Brain MRI Study",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1990-01-01",
  "study_date": "2025-11-10",
  "created_at": "2025-11-10T12:00:00Z",
  "updated_at": "2025-11-10T12:00:00Z"
}
```

**에러 (400 Bad Request)**

```json
{
  "error": "Invalid request body"
}
```

### cURL 예제

```bash
curl -X POST http://localhost:8080/api/projects/150/studies/assign \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    "study_description": "Brain MRI Study",
    "patient_id": "P12345",
    "patient_name": "홍길동",
    "patient_birth_date": "1990-01-01",
    "study_date": "2025-11-10"
  }'
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.post(
  `${apiUrl}/projects/150/studies/assign`,
  {
    study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    study_description: "Brain MRI Study",
    patient_id: "P12345",
    patient_name: "홍길동",
    patient_birth_date: "1990-01-01",
    study_date: "2025-11-10"
  }
);

console.log(response.data);
```

---

## 2. Series 할당

프로젝트에 Series를 할당합니다.

### Endpoint

```
POST /projects/{project_id}/series/assign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### Request Body

```json
{
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "series_description": "T1 Axial",
  "modality": "MR",
  "series_number": 1
}
```

#### Request Body Fields

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `study_uid` | string | ✅ | Study Instance UID (부모 Study) |
| `series_uid` | string | ✅ | Series Instance UID (DICOM 표준) |
| `series_description` | string | ❌ | Series 설명 |
| `modality` | string | ❌ | 모달리티 (CT, MR, CR, DX 등) |
| `series_number` | integer | ❌ | Series 번호 |

### Response

**성공 (200 OK)**

```json
{
  "id": 456,
  "study_id": 123,
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "series_description": "T1 Axial",
  "modality": "MR",
  "series_number": 1,
  "created_at": "2025-11-10T12:00:00Z"
}
```

**에러 (400 Bad Request)**

```json
{
  "error": "Invalid request body"
}
```

### cURL 예제

```bash
curl -X POST http://localhost:8080/api/projects/150/series/assign \
  -H "Content-Type: application/json" \
  -d '{
    "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
    "series_description": "T1 Axial",
    "modality": "MR",
    "series_number": 1
  }'
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.post(
  `${apiUrl}/projects/150/series/assign`,
  {
    study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    series_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
    series_description: "T1 Axial",
    modality: "MR",
    series_number: 1
  }
);

console.log(response.data);
```

---

## 3. Study 할당 해제

프로젝트에서 Study를 할당 해제합니다.

### Endpoint

```
DELETE /projects/{project_id}/studies/{study_id}/unassign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `study_id` | integer | ✅ | Study ID (DB의 내부 ID) |

### Response

**성공 (200 OK)**

```json
{
  "message": "Study unassigned successfully"
}
```

**에러 (404 Not Found)**

```json
{
  "error": "Study not found in project"
}
```

### cURL 예제

```bash
curl -X DELETE http://localhost:8080/api/projects/150/studies/123/unassign
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.delete(
  `${apiUrl}/projects/150/studies/123/unassign`
);

console.log(response.data);
```

---

## 4. Series 할당 해제

프로젝트에서 Series를 할당 해제합니다.

### Endpoint

```
DELETE /projects/{project_id}/series/{series_id}/unassign
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `series_id` | integer | ✅ | Series ID (DB의 내부 ID) |

### Response

**성공 (200 OK)**

```json
{
  "message": "Series unassigned successfully"
}
```

**에러 (404 Not Found)**

```json
{
  "error": "Series not found in project"
}
```

### cURL 예제

```bash
curl -X DELETE http://localhost:8080/api/projects/150/series/456/unassign
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.delete(
  `${apiUrl}/projects/150/series/456/unassign`
);

console.log(response.data);
```

---

## 5. 프로젝트 Study 목록 조회

프로젝트에 할당된 Study 목록을 조회합니다.

### Endpoint

```
GET /project-data/{project_id}/studies
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |

### Response

**성공 (200 OK)**

```json
{
  "studies": [
    {
      "id": 123,
      "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
      "study_description": "Brain MRI Study",
      "patient_id": "P12345",
      "patient_name": "홍길동",
      "patient_birth_date": "1990-01-01",
      "study_date": "2025-11-10",
      "created_at": "2025-11-10T12:00:00Z",
      "updated_at": "2025-11-10T12:00:00Z"
    }
  ]
}
```

### cURL 예제

```bash
curl -X GET http://localhost:8080/api/project-data/150/studies
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.get(
  `${apiUrl}/project-data/150/studies`
);

console.log(response.data.studies);
```

---

## 6. 프로젝트 Series 목록 조회

프로젝트에 할당된 Series 목록을 조회합니다 (Study별).

### Endpoint

```
GET /project-data/{project_id}/studies/{study_id}/series
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `study_id` | integer | ✅ | Study ID (DB의 내부 ID) |

### Response

**성공 (200 OK)**

```json
{
  "series": [
    {
      "id": 456,
      "study_id": 123,
      "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
      "series_description": "T1 Axial",
      "modality": "MR",
      "series_number": 1,
      "created_at": "2025-11-10T12:00:00Z"
    },
    {
      "id": 457,
      "study_id": 123,
      "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
      "series_description": "T2 Axial",
      "modality": "MR",
      "series_number": 2,
      "created_at": "2025-11-10T12:00:00Z"
    }
  ]
}
```

### cURL 예제

```bash
curl -X GET http://localhost:8080/api/project-data/150/studies/123/series
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.get(
  `${apiUrl}/project-data/150/studies/123/series`
);

console.log(response.data.series);
```

---

## 7. 프로젝트 Instance 목록 조회

프로젝트에 할당된 Instance 목록을 조회합니다 (Series별).

### Endpoint

```
GET /project-data/{project_id}/series/{series_id}/instances
```

### Path Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | integer | ✅ | 프로젝트 ID |
| `series_id` | integer | ✅ | Series ID (DB의 내부 ID) |

### Response

**성공 (200 OK)**

```json
{
  "instances": [
    {
      "id": 789,
      "series_id": 456,
      "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.4",
      "instance_number": 1,
      "created_at": "2025-11-10T12:00:00Z"
    }
  ]
}
```

### cURL 예제

```bash
curl -X GET http://localhost:8080/api/project-data/150/series/456/instances
```

### JavaScript (Axios) 예제

```javascript
const response = await axios.get(
  `${apiUrl}/project-data/150/series/456/instances`
);

console.log(response.data.instances);
```

---

## 📝 주요 사항

### 1. DICOM 계층 구조

```
Study (검사)
  └── Series (시리즈)
        └── Instance (이미지)
```

### 2. UID vs ID

- **UID** (예: `study_uid`, `series_uid`): DICOM 표준 고유 식별자 (문자열)
- **ID** (예: `study_id`, `series_id`): 데이터베이스 내부 ID (정수)

### 3. 할당 vs 조회 엔드포인트

- **할당/해제**: `/projects/{id}/...` (project_controller)
- **조회**: `/project-data/{id}/...` (project_data_access_controller)

### 4. Series 할당 시 Study 자동 생성

Series를 할당할 때 해당 Study가 없으면 자동으로 생성됩니다.
단, **프로젝트에는 할당되지 않습니다** (전역 엔티티만 생성).

### 5. 할당 해제 시 주의사항

- Study를 할당 해제하면 해당 Study에 속한 모든 Series도 함께 해제됩니다.
- Series만 개별적으로 할당 해제할 수도 있습니다.

---

## 🔧 에러 코드

| HTTP 상태 코드 | 설명 |
|---------------|------|
| 200 OK | 성공 |
| 400 Bad Request | 잘못된 요청 (필수 필드 누락, 잘못된 형식 등) |
| 404 Not Found | 리소스를 찾을 수 없음 (프로젝트, Study, Series 등) |
| 500 Internal Server Error | 서버 내부 오류 |

---

## 📚 참고 자료

- [DICOM 표준](https://www.dicomstandard.org/)
- [프로젝트 관리 API 문서](./project-api.md)
- [사용자 관리 API 문서](./user-api.md)

