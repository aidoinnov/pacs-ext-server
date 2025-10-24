# PACS Extension Server API 문서

## 개요

PACS Extension Server는 의료 영상 관리 및 뷰어 통합 환경을 위한 RESTful API를 제공합니다. 이 문서는 모든 API 엔드포인트의 사용법, 요청/응답 형식, 그리고 예제를 포함합니다.

## 기본 정보

- **Base URL**: `https://api.pacs-server.com/api`
- **API 버전**: v1.0.0-beta.7
- **인증 방식**: JWT Bearer Token
- **Content-Type**: `application/json`

## 인증

### JWT 토큰 획득

```http
POST /api/auth/login
Content-Type: application/json

{
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**응답:**
```json
{
  "user": {
    "id": 1,
    "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "hong_gd",
    "email": "hong@example.com",
    "full_name": "홍길동",
    "organization": "서울대학교병원",
    "department": "영상의학과",
    "phone": "010-1234-5678",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-02T00:00:00Z"
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

### 토큰 검증

```http
POST /api/auth/verify
Authorization: Bearer <jwt_token>
```

## 사용자 관리 API

### 사용자 생성

```http
POST /api/users
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "hong_gd",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

### 사용자 조회

```http
GET /api/users/{user_id}
Authorization: Bearer <jwt_token>
```

### 사용자 프로필 업데이트

```http
PUT /api/users/{user_id}
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "full_name": "홍길동",
  "email": "hong@example.com",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

### 사용자명으로 조회

```http
GET /api/users/username/{username}
Authorization: Bearer <jwt_token>
```

## 프로젝트 관리 API

### 프로젝트 생성

```http
POST /api/projects
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "name": "의료영상연구프로젝트",
  "description": "AI 기반 의료영상 분석 프로젝트",
  "is_active": true
}
```

### 프로젝트 조회

```http
GET /api/projects/{project_id}
Authorization: Bearer <jwt_token>
```

### 프로젝트 목록 조회

```http
GET /api/projects
Authorization: Bearer <jwt_token>
```

### 활성 프로젝트 조회

```http
GET /api/projects/active
Authorization: Bearer <jwt_token>
```

## 주석 관리 API

### 주석 생성

```http
POST /api/annotations
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "user_id": 1,
  "project_id": 1,
  "study_uid": "1.2.3.4.5.6.7.8.9.10",
  "series_uid": "1.2.3.4.5.6.7.8.9.11",
  "instance_uid": "1.2.3.4.5.6.7.8.9.12",
  "annotation_type": "measurement",
  "annotation_data": {
    "points": [[100, 200], [300, 400]],
    "measurements": [150.5]
  },
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "values": [42.3, 18.7],
      "unit": "mm"
    }
  ],
  "viewer_software": "OHIF Viewer"
}
```

### 주석 목록 조회

```http
GET /api/annotations?user_id=1&project_id=1&viewer_software=OHIF%20Viewer
Authorization: Bearer <jwt_token>
```

**쿼리 매개변수:**
- `user_id` (optional): 사용자 ID로 필터링
- `project_id` (optional): 프로젝트 ID로 필터링
- `study_uid` (optional): Study UID로 필터링
- `viewer_software` (optional): 뷰어 소프트웨어로 필터링

### 주석 조회

```http
GET /api/annotations/{annotation_id}
Authorization: Bearer <jwt_token>
```

### 주석 업데이트

```http
PUT /api/annotations/{annotation_id}
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "annotation_data": {
    "points": [[100, 200], [300, 400], [500, 600]],
    "measurements": [150.5, 200.3]
  },
  "measurement_values": [
    {
      "id": "m1",
      "type": "mean",
      "values": [45.2],
      "unit": "mm"
    }
  ]
}
```

### 주석 삭제

```http
DELETE /api/annotations/{annotation_id}
Authorization: Bearer <jwt_token>
```

## 마스크 그룹 관리 API

### 마스크 그룹 생성

```http
POST /api/annotations/{annotation_id}/mask-groups
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "name": "폐결절 마스크",
  "description": "AI가 생성한 폐결절 마스크",
  "mask_type": "ai_generated",
  "modality": "CT",
  "ai_model": "LungNoduleDetector v2.1"
}
```

### 마스크 그룹 목록 조회

```http
GET /api/annotations/{annotation_id}/mask-groups
Authorization: Bearer <jwt_token>
```

### 마스크 그룹 조회

```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}
Authorization: Bearer <jwt_token>
```

### 마스크 그룹 업데이트

```http
PUT /api/annotations/{annotation_id}/mask-groups/{group_id}
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "name": "폐결절 마스크 (수정됨)",
  "description": "수동으로 수정된 폐결절 마스크"
}
```

### 마스크 그룹 삭제

```http
DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}
Authorization: Bearer <jwt_token>
```

### 업로드 URL 생성

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "file_name": "lung_nodule_mask_001.png",
  "file_size": 1048576,
  "content_type": "image/png",
  "label_name": "nodule_001",
  "slice_index": 15,
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13"
}
```

### 업로드 완료 처리

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/complete-upload
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "file_name": "lung_nodule_mask_001.png",
  "file_size": 1048576,
  "checksum": "sha256:abc123...",
  "dimensions": {
    "width": 512,
    "height": 512
  }
}
```

## 마스크 관리 API

### 마스크 생성

```http
POST /api/annotations/{annotation_id}/masks
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "mask_group_id": 1,
  "file_name": "lung_nodule_mask_001.png",
  "file_size": 1048576,
  "content_type": "image/png",
  "label_name": "nodule_001",
  "slice_index": 15,
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13"
}
```

### 마스크 목록 조회

```http
GET /api/annotations/{annotation_id}/masks?mask_group_id=1
Authorization: Bearer <jwt_token>
```

### 마스크 조회

```http
GET /api/annotations/{annotation_id}/masks/{mask_id}
Authorization: Bearer <jwt_token>
```

### 다운로드 URL 생성

```http
POST /api/annotations/{annotation_id}/masks/{mask_id}/download-url
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "mask_id": 1,
  "expires_in": 3600
}
```

### 마스크 삭제

```http
DELETE /api/annotations/{annotation_id}/masks/{mask_id}
Authorization: Bearer <jwt_token>
```

## 역할 및 권한 관리 API

### 역할 생성

```http
POST /api/roles
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "name": "프로젝트 관리자",
  "description": "프로젝트 전체 관리 권한",
  "scope": "project"
}
```

### 역할 조회

```http
GET /api/roles/{role_id}
Authorization: Bearer <jwt_token>
```

### 글로벌 역할 목록

```http
GET /api/roles/global
Authorization: Bearer <jwt_token>
```

### 프로젝트 역할 목록

```http
GET /api/roles/project
Authorization: Bearer <jwt_token>
```

## 접근 제어 API

### DICOM 접근 로그 기록

```http
POST /api/access-control/logs
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "user_id": 1,
  "project_id": 1,
  "study_uid": "1.2.3.4.5.6.7.8.9.10",
  "series_uid": "1.2.3.4.5.6.7.8.9.11",
  "instance_uid": "1.2.3.4.5.6.7.8.9.12",
  "access_type": "view",
  "ip_address": "192.168.1.100",
  "user_agent": "OHIF Viewer 3.0.0"
}
```

### 사용자별 접근 로그

```http
GET /api/access-control/logs/user/{user_id}
Authorization: Bearer <jwt_token>
```

### 프로젝트별 접근 로그

```http
GET /api/access-control/logs/project/{project_id}
Authorization: Bearer <jwt_token>
```

### Study별 접근 로그

```http
GET /api/access-control/logs/study/{study_uid}
Authorization: Bearer <jwt_token>
```

### 권한 확인

```http
POST /api/access-control/permissions/check
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "user_id": 1,
  "project_id": 1,
  "permission": "annotation:create"
}
```

### 사용자 권한 목록

```http
GET /api/access-control/permissions/user/{user_id}/project/{project_id}
Authorization: Bearer <jwt_token>
```

### 프로젝트 접근 가능 여부

```http
GET /api/access-control/access/user/{user_id}/project/{project_id}
Authorization: Bearer <jwt_token>
```

## 에러 응답

### 표준 에러 형식

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "유효하지 않은 요청 데이터입니다.",
    "details": {
      "field": "email",
      "reason": "이메일 형식이 올바르지 않습니다."
    }
  }
}
```

### HTTP 상태 코드

- **200 OK**: 성공
- **201 Created**: 리소스 생성 성공
- **400 Bad Request**: 잘못된 요청
- **401 Unauthorized**: 인증 실패
- **403 Forbidden**: 권한 부족
- **404 Not Found**: 리소스를 찾을 수 없음
- **409 Conflict**: 리소스 충돌 (중복 등)
- **422 Unprocessable Entity**: 유효성 검사 실패
- **500 Internal Server Error**: 서버 내부 오류

### 에러 코드

- **VALIDATION_ERROR**: 유효성 검사 실패
- **NOT_FOUND**: 리소스를 찾을 수 없음
- **ALREADY_EXISTS**: 리소스가 이미 존재함
- **UNAUTHORIZED**: 인증 실패
- **FORBIDDEN**: 권한 부족
- **DATABASE_ERROR**: 데이터베이스 오류
- **EXTERNAL_SERVICE_ERROR**: 외부 서비스 오류

## 페이징

### 페이징 매개변수

- `page`: 페이지 번호 (기본값: 1)
- `limit`: 페이지당 항목 수 (기본값: 20, 최대: 100)
- `sort`: 정렬 필드
- `order`: 정렬 순서 (asc, desc)

### 페이징 응답

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "pages": 5,
    "has_next": true,
    "has_prev": false
  }
}
```

## 필터링 및 검색

### 주석 필터링

```http
GET /api/annotations?user_id=1&project_id=1&viewer_software=OHIF%20Viewer&study_uid=1.2.3.4.5.6.7.8.9.10
```

### 사용자 검색

```http
GET /api/users?search=홍길동&organization=서울대학교병원
```

## Rate Limiting

- **기본 제한**: 분당 1000 요청
- **인증된 사용자**: 분당 5000 요청
- **관리자**: 분당 10000 요청

### Rate Limit 헤더

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Webhook

### Webhook 이벤트

- `annotation.created`: 주석 생성
- `annotation.updated`: 주석 업데이트
- `annotation.deleted`: 주석 삭제
- `mask_group.created`: 마스크 그룹 생성
- `mask.uploaded`: 마스크 업로드 완료

### Webhook 페이로드

```json
{
  "event": "annotation.created",
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "annotation_id": 1,
    "user_id": 1,
    "project_id": 1,
    "study_uid": "1.2.3.4.5.6.7.8.9.10"
  }
}
```

## SDK 및 클라이언트 라이브러리

### JavaScript/TypeScript

```bash
npm install @pacs-server/sdk
```

```typescript
import { PACSClient } from '@pacs-server/sdk';

const client = new PACSClient({
  baseUrl: 'https://api.pacs-server.com/api',
  token: 'your-jwt-token'
});

// 주석 생성
const annotation = await client.annotations.create({
  user_id: 1,
  project_id: 1,
  study_uid: '1.2.3.4.5.6.7.8.9.10',
  annotation_type: 'measurement',
  annotation_data: {
    points: [[100, 200], [300, 400]],
    measurements: [150.5]
  }
});
```

### Python

```bash
pip install pacs-server-sdk
```

```python
from pacs_server import PACSClient

client = PACSClient(
    base_url='https://api.pacs-server.com/api',
    token='your-jwt-token'
)

# 주석 생성
annotation = client.annotations.create(
    user_id=1,
    project_id=1,
    study_uid='1.2.3.4.5.6.7.8.9.10',
    annotation_type='measurement',
    annotation_data={
        'points': [[100, 200], [300, 400]],
        'measurements': [150.5]
    }
)
```

## OpenAPI/Swagger

API의 상세한 문서는 Swagger UI를 통해 확인할 수 있습니다:

- **Swagger UI**: `https://api.pacs-server.com/docs`
- **OpenAPI JSON**: `https://api.pacs-server.com/openapi.json`

## 지원 및 문의

- **문서**: [https://docs.pacs-server.com](https://docs.pacs-server.com)
- **이슈 트래커**: [https://github.com/pacs-server/issues](https://github.com/pacs-server/issues)
- **이메일**: support@pacs-server.com
- **Slack**: #pacs-server-support
