# Project CRUD API (완전한 문서)

## 개요

프로젝트의 생성, 조회, 수정을 위한 완전한 CRUD API입니다. 새로 추가된 필드(sponsor, start_date, end_date, auto_complete)를 포함합니다.

**Base URL**: `/api/projects`

---

## API 엔드포인트

### 1. 프로젝트 생성

새로운 프로젝트를 생성합니다.

**Endpoint**: `POST /api/projects`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "sponsor": "서울대학교병원",
  "start_date": "2025-01-01",
  "end_date": "2025-12-31",
  "auto_complete": false
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `name` | string | Yes | 프로젝트 이름 | `"심장 질환 연구 프로젝트"` |
| `description` | string | No | 프로젝트 설명 | `"심장 질환 관련 DICOM 영상 분석 프로젝트"` |
| `sponsor` | string | Yes | 스폰서명 | `"서울대학교병원"` |
| `start_date` | date | Yes | 시작일 | `"2025-01-01"` |
| `end_date` | date | No | 종료일/목표일 | `"2025-12-31"` |
| `auto_complete` | boolean | No | 자동 완료 여부 | `false` |

#### Response

**Success Response** (201 Created)

```json
{
  "id": 1,
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "sponsor": "서울대학교병원",
  "start_date": "2025-01-01",
  "end_date": "2025-12-31",
  "auto_complete": false,
  "is_active": true,
  "status": "Planning",
  "created_at": "2025-01-01T00:00:00Z"
}
```

#### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | 프로젝트 ID |
| `name` | string | 프로젝트 이름 |
| `description` | string | 프로젝트 설명 |
| `sponsor` | string | 스폰서명 |
| `start_date` | date | 시작일 |
| `end_date` | date | 종료일/목표일 |
| `auto_complete` | boolean | 자동 완료 여부 |
| `is_active` | boolean | 활성 상태 |
| `status` | string | 프로젝트 상태 (Planning, Active, Completed, etc.) |
| `created_at` | string (ISO 8601) | 생성 시간 |

---

### 2. 프로젝트 조회

특정 프로젝트의 정보를 조회합니다.

**Endpoint**: `GET /api/projects/{project_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 조회할 프로젝트 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "sponsor": "서울대학교병원",
  "start_date": "2025-01-01",
  "end_date": "2025-12-31",
  "auto_complete": false,
  "is_active": true,
  "status": "Planning",
  "created_at": "2025-01-01T00:00:00Z"
}
```

---

### 3. 프로젝트 수정

프로젝트 정보를 수정합니다.

**Endpoint**: `PUT /api/projects/{project_id}`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 수정할 프로젝트 ID |

#### Request Body

```json
{
  "name": "업데이트된 프로젝트명",
  "description": "업데이트된 설명",
  "sponsor": "업데이트된 스폰서",
  "start_date": "2025-02-01",
  "end_date": "2026-01-31",
  "status": "ACTIVE",
  "auto_complete": true,
  "is_active": true
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `name` | string | No | 프로젝트 이름 | `"업데이트된 프로젝트명"` |
| `description` | string | No | 프로젝트 설명 | `"업데이트된 설명"` |
| `sponsor` | string | No | 스폰서명 | `"서울대학교병원"` |
| `start_date` | date | No | 시작일 | `"2025-02-01"` |
| `end_date` | date | No | 종료일/목표일 | `"2026-01-31"` |
| `status` | string | No | 프로젝트 상태 | `"ACTIVE"` |
| `auto_complete` | boolean | No | 자동 완료 여부 | `true` |
| `is_active` | boolean | No | 활성 상태 | `true` |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "name": "업데이트된 프로젝트명",
  "description": "업데이트된 설명",
  "sponsor": "서울대학교병원",
  "start_date": "2025-02-01",
  "end_date": "2026-01-31",
  "auto_complete": true,
  "is_active": true,
  "status": "Active",
  "created_at": "2025-01-01T00:00:00Z"
}
```

---

### 4. 프로젝트 목록 조회 (페이지네이션 지원)

모든 프로젝트의 목록을 조회합니다. 페이지네이션, 정렬, 필터링을 지원합니다.

**Endpoint**: `GET /api/projects`

**Authentication**: Required (JWT Token)

#### Query Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `page` | integer | No | 페이지 번호 (기본값: 1) | `1` |
| `page_size` | integer | No | 페이지 크기 (기본값: 20) | `20` |
| `sort_by` | string | No | 정렬 기준 (created_at, name, start_date) | `name` |
| `sort_order` | string | No | 정렬 순서 (asc, desc) | `asc` |
| `status` | string | No | 상태 필터 | `ACTIVE` |
| `sponsor` | string | No | 스폰서 필터 | `서울대학교병원` |
| `start_date_from` | date | No | 시작일 시작 범위 | `2025-01-01` |
| `start_date_to` | date | No | 시작일 종료 범위 | `2025-12-31` |
| `end_date_from` | date | No | 종료일 시작 범위 | `2025-06-01` |
| `end_date_to` | date | No | 종료일 종료 범위 | `2025-12-31` |

#### Response

**Success Response** (200 OK)

```json
{
  "projects": [
    {
      "id": 1,
      "name": "심장 질환 연구 프로젝트",
      "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
      "sponsor": "서울대학교병원",
      "start_date": "2025-01-01",
      "end_date": "2025-12-31",
      "auto_complete": false,
      "is_active": true,
      "status": "Planning",
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 40,
    "total_pages": 2
  }
}
```

#### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `projects` | array | 프로젝트 목록 |
| `pagination.page` | integer | 현재 페이지 |
| `pagination.page_size` | integer | 페이지 크기 |
| `pagination.total` | integer | 전체 항목 수 |
| `pagination.total_pages` | integer | 전체 페이지 수 |

---

### 5. 활성 프로젝트 목록 조회 (페이지네이션 지원)

활성 상태인 프로젝트만 조회합니다. 페이지네이션과 정렬을 지원합니다.

**Endpoint**: `GET /api/projects/active`

**Authentication**: Required (JWT Token)

#### Query Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `page` | integer | No | 페이지 번호 (기본값: 1) | `1` |
| `page_size` | integer | No | 페이지 크기 (기본값: 20) | `20` |
| `sort_by` | string | No | 정렬 기준 (created_at, name, start_date) | `name` |
| `sort_order` | string | No | 정렬 순서 (asc, desc) | `asc` |

#### Response

**Success Response** (200 OK)

```json
{
  "projects": [
    {
      "id": 1,
      "name": "심장 질환 연구 프로젝트",
      "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
      "sponsor": "서울대학교병원",
      "start_date": "2025-01-01",
      "end_date": "2025-12-31",
      "auto_complete": false,
      "is_active": true,
      "status": "Active",
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 40,
    "total_pages": 2
  }
}
```

---

### 6. 프로젝트 삭제

프로젝트를 삭제합니다.

**Endpoint**: `DELETE /api/projects/{project_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 삭제할 프로젝트 ID |

#### Response

**Success Response** (204 No Content)

프로젝트가 성공적으로 삭제되면 내용 없이 204 상태 코드를 반환합니다.

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 404 | Project not found | `{"error": "Failed to delete project: ..."}` |
| 500 | Internal server error | `{"error": "..."}` |

---

## 사용 예시

### cURL 요청 예시

#### 프로젝트 생성

```bash
curl -X POST "http://localhost:8080/api/projects" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "심장 질환 연구 프로젝트",
    "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
    "sponsor": "서울대학교병원",
    "start_date": "2025-01-01",
    "end_date": "2025-12-31",
    "auto_complete": false
  }'
```

#### 프로젝트 조회

```bash
curl -X GET "http://localhost:8080/api/projects/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 프로젝트 수정

```bash
curl -X PUT "http://localhost:8080/api/projects/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "업데이트된 프로젝트명",
    "description": "업데이트된 설명",
    "sponsor": "서울대학교병원",
    "start_date": "2025-02-01",
    "end_date": "2026-01-31",
    "status": "ACTIVE",
    "auto_complete": true,
    "is_active": true
  }'
```

#### 프로젝트 목록 조회 (페이지네이션)

```bash
# 기본 조회 (page=1, page_size=20)
curl -X GET "http://localhost:8080/api/projects?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 정렬 포함
curl -X GET "http://localhost:8080/api/projects?page=1&page_size=10&sort_by=name&sort_order=asc" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 필터링 포함
curl -X GET "http://localhost:8080/api/projects?page=1&page_size=10&status=ACTIVE&sponsor=서울대학교병원" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 활성 프로젝트 목록 조회 (페이지네이션)

```bash
# 기본 조회
curl -X GET "http://localhost:8080/api/projects/active?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 정렬 포함
curl -X GET "http://localhost:8080/api/projects/active?page=1&page_size=10&sort_by=created_at&sort_order=desc" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 프로젝트 삭제

```bash
curl -X DELETE "http://localhost:8080/api/projects/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 프로젝트 상태

### status 필드

프로젝트의 생명주기 상태를 나타냅니다.

- `PLANNING`: 기획중 - 프로젝트가 기획 단계
- `ACTIVE`: 진행중 - 프로젝트가 활발히 진행 중
- `COMPLETED`: 완료 - 프로젝트가 성공적으로 완료됨
- `SUSPENDED`: 보류 - 프로젝트가 일시적으로 중단됨
- `CANCELLED`: 취소 - 프로젝트가 취소됨
- `PENDING_COMPLETION`: 완료 대기 - 프로젝트 종료 대기 중
- `OVER_PLANNING`: 계획 초과 - 프로젝트 계획 초과 상태

### is_active 필드

프로젝트의 아카이브 상태를 나타냅니다. `status`와는 별개로 관리됩니다.

- `true`: 프로젝트가 활성 상태 (기본값)
- `false`: 프로젝트가 비활성/아카이브 상태

### auto_complete 필드

프로젝트의 자동 완료 여부를 나타냅니다.

- `true`: 종료일(end_date) 도달 시 자동으로 완료 상태로 전환
- `false`: 수동으로만 완료 처리 (기본값)

---

## 에러 처리

### 공통 에러 응답 형식

```json
{
  "error": "[에러 메시지]"
}
```

### 에러 코드별 설명

| HTTP Status | 에러 내용 | 설명 |
|-------------|---------|------|
| 400 | Invalid request | 요청 데이터가 유효하지 않음 |
| 401 | Unauthorized | 인증되지 않은 요청 |
| 404 | Project not found | 프로젝트가 존재하지 않음 |
| 500 | Internal server error | 서버 내부 오류 |

---

## 중요한 필드 설명

### sponsor

프로젝트의 스폰서명을 나타냅니다. 예: "서울대학교병원", "삼성서울병원", "아산병원" 등.

### start_date

프로젝트의 시작일을 나타냅니다. `YYYY-MM-DD` 형식으로 지정합니다.

### end_date

프로젝트의 종료일 또는 목표일을 나타냅니다. `YYYY-MM-DD` 형식으로 지정합니다. 선택 필드이며, 지정하지 않으면 `null`입니다.

### auto_complete

이 필드가 `true`인 경우, `end_date`에 도달하면 프로젝트 상태가 자동으로 `COMPLETED`로 변경됩니다.

### is_active와 status의 관계

- `is_active`: 아카이브 여부 (true: 활성, false: 아카이브)
- `status`: 프로젝트 진행 상태 (PLANNING, ACTIVE, COMPLETED, etc.)

예를 들어, 완료된 프로젝트는 `status=COMPLETED`이고 `is_active=true`일 수 있습니다. 아카이브하려면 `is_active=false`로 설정하면 됩니다.

---

**최종 업데이트**: 2025-01-27

