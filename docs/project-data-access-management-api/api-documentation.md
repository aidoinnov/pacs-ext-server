# 프로젝트 데이터 접근 관리 API 문서

## 📋 API 개요

프로젝트 데이터 접근 관리 API는 프로젝트 참여자가 프로젝트에 포함된 DICOM Study 데이터에 대한 접근 상태를 조회하고 수정할 수 있는 RESTful API입니다.

**Base URL**: `http://localhost:8080/api`  
**인증**: JWT Bearer Token  
**Content-Type**: `application/json`

## 🔐 인증

모든 API 요청에는 JWT 토큰이 필요합니다.

```http
Authorization: Bearer <your-jwt-token>
```

## 📊 API 엔드포인트

### 1. 데이터 접근 상태 조회

프로젝트의 데이터 접근 상태를 페이지네이션과 검색 기능을 통해 조회합니다.

**엔드포인트**: `GET /api/projects/{project_id}/data-access`

#### 경로 파라미터
- `project_id` (string, required): 프로젝트 ID (UUID)

#### 쿼리 파라미터
- `page` (integer, optional): 페이지 번호 (기본값: 1, 최소: 1)
- `limit` (integer, optional): 페이지 크기 (기본값: 20, 최소: 1, 최대: 100)
- `search` (string, optional): 검색어 (study_uid, patient_id, patient_name 검색)
- `user_search` (string, optional): 사용자 검색어 (username, email 검색)
- `status` (string, optional): 상태 필터 (APPROVED, DENIED, PENDING)

#### 응답

**성공 응답 (200 OK)**:
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "project_data": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "patient_id": "P001",
        "patient_name": "John Doe",
        "study_date": "2025-01-27",
        "study_description": "Chest X-ray",
        "modality": "CR",
        "series_count": 1,
        "instance_count": 1
      },
      "user": {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "username": "john_doe",
        "email": "john@example.com",
        "full_name": "John Doe"
      },
      "access_status": "APPROVED",
      "granted_by": {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "username": "admin",
        "email": "admin@example.com",
        "full_name": "Administrator"
      },
      "granted_at": "2025-01-27T10:00:00Z",
      "created_at": "2025-01-27T09:00:00Z",
      "updated_at": "2025-01-27T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

**에러 응답**:
- `400 Bad Request`: 잘못된 요청 파라미터
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: 프로젝트를 찾을 수 없음
- `500 Internal Server Error`: 서버 내부 오류

### 2. 데이터 접근 상태 수정

특정 데이터에 대한 사용자의 접근 상태를 수정합니다.

**엔드포인트**: `PUT /api/projects/{project_id}/data-access/{data_id}`

#### 경로 파라미터
- `project_id` (string, required): 프로젝트 ID (UUID)
- `data_id` (string, required): 프로젝트 데이터 ID (UUID)

#### 요청 본문
```json
{
  "access_status": "APPROVED"
}
```

#### 요청 필드
- `access_status` (string, required): 접근 상태 (APPROVED, DENIED, PENDING)

#### 응답

**성공 응답 (200 OK)**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "project_data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
    "patient_id": "P001",
    "patient_name": "John Doe",
    "study_date": "2025-01-27",
    "study_description": "Chest X-ray",
    "modality": "CR",
    "series_count": 1,
    "instance_count": 1
  },
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "username": "john_doe",
    "email": "john@example.com",
    "full_name": "John Doe"
  },
  "access_status": "APPROVED",
  "granted_by": {
    "id": "550e8400-e29b-41d4-a716-446655440003",
    "username": "admin",
    "email": "admin@example.com",
    "full_name": "Administrator"
  },
  "granted_at": "2025-01-27T10:00:00Z",
  "created_at": "2025-01-27T09:00:00Z",
  "updated_at": "2025-01-27T10:00:00Z"
}
```

**에러 응답**:
- `400 Bad Request`: 잘못된 요청 데이터
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: 프로젝트 또는 데이터를 찾을 수 없음
- `422 Unprocessable Entity`: 유효성 검사 실패
- `500 Internal Server Error`: 서버 내부 오류

### 3. 데이터 접근 매트릭스 조회

프로젝트의 데이터 접근 매트릭스를 조회합니다. 각 데이터에 대한 모든 사용자의 접근 상태를 한 번에 확인할 수 있습니다.

**엔드포인트**: `GET /api/projects/{project_id}/data-access/matrix`

#### 경로 파라미터
- `project_id` (string, required): 프로젝트 ID (UUID)

#### 쿼리 파라미터
- `page` (integer, optional): 페이지 번호 (기본값: 1, 최소: 1)
- `limit` (integer, optional): 페이지 크기 (기본값: 20, 최소: 1, 최대: 100)
- `search` (string, optional): 검색어 (study_uid, patient_id, patient_name 검색)
- `user_search` (string, optional): 사용자 검색어 (username, email 검색)

#### 응답

**성공 응답 (200 OK)**:
```json
{
  "data": [
    {
      "project_data": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "patient_id": "P001",
        "patient_name": "John Doe",
        "study_date": "2025-01-27",
        "study_description": "Chest X-ray",
        "modality": "CR",
        "series_count": 1,
        "instance_count": 1
      },
      "user_access": [
        {
          "user": {
            "id": "550e8400-e29b-41d4-a716-446655440002",
            "username": "john_doe",
            "email": "john@example.com",
            "full_name": "John Doe"
          },
          "access_status": "APPROVED",
          "granted_at": "2025-01-27T10:00:00Z"
        },
        {
          "user": {
            "id": "550e8400-e29b-41d4-a716-446655440004",
            "username": "jane_doe",
            "email": "jane@example.com",
            "full_name": "Jane Doe"
          },
          "access_status": "PENDING",
          "granted_at": null
        }
      ]
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

**에러 응답**:
- `400 Bad Request`: 잘못된 요청 파라미터
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: 프로젝트를 찾을 수 없음
- `500 Internal Server Error`: 서버 내부 오류

## 📊 데이터 모델

### ProjectDataAccessDto
```json
{
  "id": "string (UUID)",
  "project_data": {
    "id": "string (UUID)",
    "study_uid": "string",
    "patient_id": "string",
    "patient_name": "string",
    "study_date": "string (ISO 8601 date)",
    "study_description": "string",
    "modality": "string",
    "series_count": "integer",
    "instance_count": "integer"
  },
  "user": {
    "id": "string (UUID)",
    "username": "string",
    "email": "string",
    "full_name": "string"
  },
  "access_status": "string (APPROVED | DENIED | PENDING)",
  "granted_by": {
    "id": "string (UUID)",
    "username": "string",
    "email": "string",
    "full_name": "string"
  },
  "granted_at": "string (ISO 8601 datetime)",
  "created_at": "string (ISO 8601 datetime)",
  "updated_at": "string (ISO 8601 datetime)"
}
```

### ProjectDataAccessMatrixDto
```json
{
  "project_data": {
    "id": "string (UUID)",
    "study_uid": "string",
    "patient_id": "string",
    "patient_name": "string",
    "study_date": "string (ISO 8601 date)",
    "study_description": "string",
    "modality": "string",
    "series_count": "integer",
    "instance_count": "integer"
  },
  "user_access": [
    {
      "user": {
        "id": "string (UUID)",
        "username": "string",
        "email": "string",
        "full_name": "string"
      },
      "access_status": "string (APPROVED | DENIED | PENDING)",
      "granted_at": "string (ISO 8601 datetime) | null"
    }
  ]
}
```

### UpdateDataAccessStatusRequest
```json
{
  "access_status": "string (APPROVED | DENIED | PENDING)"
}
```

### PaginationDto
```json
{
  "page": "integer",
  "limit": "integer",
  "total": "integer",
  "total_pages": "integer"
}
```

## 🔍 사용 예시

### 1. 데이터 접근 상태 조회

```bash
# 기본 조회
curl -X GET "http://localhost:8080/api/projects/550e8400-e29b-41d4-a716-446655440000/data-access" \
  -H "Authorization: Bearer your-jwt-token"

# 페이지네이션과 검색
curl -X GET "http://localhost:8080/api/projects/550e8400-e29b-41d4-a716-446655440000/data-access?page=1&limit=10&search=P001&status=APPROVED" \
  -H "Authorization: Bearer your-jwt-token"
```

### 2. 데이터 접근 상태 수정

```bash
curl -X PUT "http://localhost:8080/api/projects/550e8400-e29b-41d4-a716-446655440000/data-access/550e8400-e29b-41d4-a716-446655440001" \
  -H "Authorization: Bearer your-jwt-token" \
  -H "Content-Type: application/json" \
  -d '{
    "access_status": "APPROVED"
  }'
```

### 3. 데이터 접근 매트릭스 조회

```bash
curl -X GET "http://localhost:8080/api/projects/550e8400-e29b-41d4-a716-446655440000/data-access/matrix?page=1&limit=10" \
  -H "Authorization: Bearer your-jwt-token"
```

## 🚨 에러 코드

| HTTP 상태 코드 | 설명 | 해결 방법 |
|---------------|------|-----------|
| 400 | Bad Request | 요청 파라미터를 확인하세요 |
| 401 | Unauthorized | JWT 토큰을 확인하세요 |
| 403 | Forbidden | 프로젝트 참여 권한을 확인하세요 |
| 404 | Not Found | 프로젝트 또는 데이터가 존재하는지 확인하세요 |
| 422 | Unprocessable Entity | 요청 데이터의 유효성을 확인하세요 |
| 500 | Internal Server Error | 서버 관리자에게 문의하세요 |

## 📈 성능 고려사항

### 페이지네이션
- 기본 페이지 크기는 20개입니다
- 최대 페이지 크기는 100개입니다
- 대량의 데이터를 처리할 때는 페이지네이션을 사용하세요

### 검색 성능
- `search` 파라미터는 study_uid, patient_id, patient_name을 검색합니다
- `user_search` 파라미터는 username, email을 검색합니다
- 검색 성능을 위해 적절한 인덱스가 설정되어 있습니다

### 캐싱
- 자주 접근하는 데이터는 캐시됩니다
- 데이터가 변경되면 캐시가 자동으로 무효화됩니다

## 🔄 버전 관리

현재 API 버전: `v1`

API 버전은 URL 경로에 포함되지 않으며, 헤더를 통해 관리됩니다.

```http
Accept: application/vnd.pacs-api.v1+json
```

## 📞 지원

API 사용 중 문제가 발생하면 다음을 확인하세요:

1. **인증**: JWT 토큰이 유효한지 확인
2. **권한**: 프로젝트 참여 권한이 있는지 확인
3. **파라미터**: 요청 파라미터가 올바른지 확인
4. **네트워크**: 네트워크 연결 상태 확인

추가 지원이 필요한 경우 서버 관리자에게 문의하세요.
