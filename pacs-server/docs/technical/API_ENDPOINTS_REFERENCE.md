# 🌐 PACS Extension Server API 엔드포인트 참조

## 📋 개요

PACS Extension Server는 RESTful API를 통해 의료 영상 어노테이션 및 마스크 업로드 기능을 제공합니다. 모든 API는 JSON 형식으로 데이터를 주고받으며, JWT 토큰 기반 인증을 사용합니다.

## 🔐 인증

### JWT 토큰
모든 API 요청에는 Authorization 헤더에 JWT 토큰이 필요합니다.

```http
Authorization: Bearer <jwt-token>
```

### 토큰 획득
```http
POST /api/auth/login
Content-Type: application/json

{
  "keycloak_id": "uuid",
  "username": "string",
  "email": "string"
}
```

## 👥 사용자 관리 API

### 사용자 생성
```http
POST /api/users
Content-Type: application/json

{
  "keycloak_id": "uuid",
  "username": "string",
  "email": "string"
}
```

**응답**: `201 Created`
```json
{
  "id": 1,
  "keycloak_id": "uuid",
  "username": "string",
  "email": "string",
  "created_at": "2025-10-07T00:00:00Z"
}
```

### 사용자 조회
```http
GET /api/users/{user_id}
```

**응답**: `200 OK`
```json
{
  "id": 1,
  "keycloak_id": "uuid",
  "username": "string",
  "email": "string",
  "created_at": "2025-10-07T00:00:00Z"
}
```

### 사용자명으로 조회
```http
GET /api/users/username/{username}
```

## 🏢 프로젝트 관리 API

### 프로젝트 생성
```http
POST /api/projects
Content-Type: application/json

{
  "name": "string",
  "description": "string"
}
```

**응답**: `201 Created`
```json
{
  "id": 1,
  "name": "string",
  "description": "string",
  "is_active": true,
  "created_at": "2025-10-07T00:00:00Z"
}
```

### 프로젝트 조회
```http
GET /api/projects/{project_id}
```

### 프로젝트 목록 조회
```http
GET /api/projects
```

### 프로젝트 멤버 추가
```http
POST /api/projects/{project_id}/members
Content-Type: application/json

{
  "user_id": 1
}
```

## 🏷️ 어노테이션 관리 API

### 어노테이션 생성
```http
POST /api/annotations
Content-Type: application/json

{
  "study_instance_uid": "string",
  "series_instance_uid": "string",
  "sop_instance_uid": "string",
  "annotation_data": {},
  "viewer_software": "string",
  "tool_name": "string",
  "tool_version": "string",
  "description": "string"
}
```

**응답**: `201 Created`
```json
{
  "id": 1,
  "project_id": 1,
  "user_id": 1,
  "study_uid": "string",
  "series_uid": "string",
  "instance_uid": "string",
  "tool_name": "string",
  "tool_version": "string",
  "viewer_software": "string",
  "data": {},
  "description": "string",
  "is_shared": false,
  "created_at": "2025-10-07T00:00:00Z",
  "updated_at": "2025-10-07T00:00:00Z"
}
```

### 어노테이션 조회
```http
GET /api/annotations/{annotation_id}
```

### 어노테이션 수정
```http
PUT /api/annotations/{annotation_id}
Content-Type: application/json

{
  "annotation_data": {},
  "viewer_software": "string",
  "tool_name": "string",
  "tool_version": "string",
  "description": "string"
}
```

### 어노테이션 삭제
```http
DELETE /api/annotations/{annotation_id}
```

### 어노테이션 목록 조회
```http
GET /api/annotations?project_id={project_id}&user_id={user_id}&limit={limit}&offset={offset}
```

## 🎭 마스크 그룹 관리 API

### 마스크 그룹 생성
```http
POST /api/annotations/{annotation_id}/mask-groups
Content-Type: application/json

{
  "group_name": "string",
  "model_name": "string",
  "version": "string",
  "modality": "string",
  "slice_count": 100,
  "mask_type": "string",
  "description": "string"
}
```

**응답**: `201 Created`
```json
{
  "id": 1,
  "annotation_id": 1,
  "group_name": "string",
  "model_name": "string",
  "version": "string",
  "modality": "string",
  "slice_count": 100,
  "mask_type": "string",
  "description": "string",
  "created_by": 1,
  "created_at": "2025-10-07T00:00:00Z",
  "updated_at": "2025-10-07T00:00:00Z"
}
```

### 마스크 그룹 조회
```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}
```

### 마스크 그룹 수정
```http
PUT /api/annotations/{annotation_id}/mask-groups/{group_id}
Content-Type: application/json

{
  "group_name": "string",
  "model_name": "string",
  "version": "string",
  "modality": "string",
  "slice_count": 100,
  "mask_type": "string",
  "description": "string"
}
```

### 마스크 그룹 삭제
```http
DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}
```

### 마스크 그룹 목록 조회
```http
GET /api/annotations/{annotation_id}/mask-groups?created_by={user_id}&modality={modality}&mask_type={type}&limit={limit}&offset={offset}
```

### 업로드 URL 생성
```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url
Content-Type: application/json

{
  "filename": "string",
  "mime_type": "string",
  "ttl_seconds": 3600
}
```

**응답**: `200 OK`
```json
{
  "upload_url": "https://s3.amazonaws.com/bucket/path?signature=...",
  "download_url": "https://s3.amazonaws.com/bucket/path?signature=...",
  "file_path": "string",
  "expires_in": 3600,
  "expires_at": "2025-10-07T01:00:00Z"
}
```

### 업로드 완료 처리
```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/complete-upload
Content-Type: application/json

{
  "slice_count": 100,
  "labels": ["string"],
  "uploaded_files": ["string"]
}
```

**응답**: `200 OK`
```json
{
  "success": true,
  "status": "success",
  "processed_masks": 100,
  "uploaded_files": ["string"],
  "message": "Upload completed successfully"
}
```

## 🎨 마스크 관리 API

### 마스크 생성
```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/masks
Content-Type: application/json

{
  "file_path": "string",
  "mime_type": "string",
  "slice_index": 1,
  "sop_instance_uid": "string",
  "label_name": "string",
  "file_size": 102400,
  "checksum": "string",
  "width": 512,
  "height": 512
}
```

**응답**: `201 Created`
```json
{
  "id": 1,
  "mask_group_id": 1,
  "slice_index": 1,
  "sop_instance_uid": "string",
  "label_name": "string",
  "file_path": "string",
  "mime_type": "string",
  "file_size": 102400,
  "checksum": "string",
  "width": 512,
  "height": 512,
  "created_at": "2025-10-07T00:00:00Z",
  "updated_at": "2025-10-07T00:00:00Z"
}
```

### 마스크 조회
```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
```

### 마스크 수정
```http
PUT /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
Content-Type: application/json

{
  "file_path": "string",
  "mime_type": "string",
  "slice_index": 1,
  "sop_instance_uid": "string",
  "label_name": "string",
  "file_size": 102400,
  "checksum": "string",
  "width": 512,
  "height": 512
}
```

### 마스크 삭제
```http
DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
```

### 마스크 목록 조회
```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks?slice_index={index}&label_name={label}&limit={limit}&offset={offset}
```

### 다운로드 URL 생성
```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url
Content-Type: application/json

{
  "file_path": "string",
  "expires_in": 3600
}
```

**응답**: `200 OK`
```json
{
  "download_url": "https://s3.amazonaws.com/bucket/path?signature=...",
  "file_path": "string",
  "expires_in": 3600,
  "expires_at": "2025-10-07T01:00:00Z"
}
```

### 마스크 통계 조회
```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/stats
```

**응답**: `200 OK`
```json
{
  "total_masks": 100,
  "total_size_bytes": 10485760,
  "slice_count": 100,
  "label_distribution": {
    "liver": 50,
    "spleen": 30,
    "kidney": 20
  }
}
```

## 🔐 권한 관리 API

### 역할 생성
```http
POST /api/roles
Content-Type: application/json

{
  "name": "string",
  "description": "string",
  "scope": "GLOBAL"
}
```

### 역할 조회
```http
GET /api/roles/{role_id}
```

### 전역 역할 목록 조회
```http
GET /api/roles/global
```

### 프로젝트 역할 목록 조회
```http
GET /api/roles/project/{project_id}
```

### 권한 할당
```http
POST /api/roles/{role_id}/permissions
Content-Type: application/json

{
  "permission_id": 1
}
```

## 📊 접근 제어 API

### DICOM 접근 로그 기록
```http
POST /api/access-logs/dicom
Content-Type: application/json

{
  "user_id": 1,
  "project_id": 1,
  "study_uid": "string",
  "series_uid": "string",
  "instance_uid": "string",
  "action": "VIEW",
  "ip_address": "string",
  "user_agent": "string"
}
```

### 사용자 접근 로그 조회
```http
GET /api/access-logs/user/{user_id}?limit={limit}&offset={offset}
```

### 프로젝트 접근 로그 조회
```http
GET /api/access-logs/project/{project_id}?limit={limit}&offset={offset}
```

### Study 접근 로그 조회
```http
GET /api/access-logs/study/{study_uid}?limit={limit}&offset={offset}
```

### 권한 확인
```http
POST /api/access-control/check-permission
Content-Type: application/json

{
  "user_id": 1,
  "project_id": 1,
  "resource_type": "string",
  "action": "string"
}
```

**응답**: `200 OK`
```json
{
  "user_id": 1,
  "project_id": 1,
  "resource_type": "string",
  "action": "string",
  "has_permission": true
}
```

### 사용자 권한 조회
```http
GET /api/access-control/user/{user_id}/permissions?project_id={project_id}
```

### 프로젝트 접근 가능 여부 확인
```http
GET /api/access-control/project/{project_id}/can-access?user_id={user_id}
```

## 🏥 헬스체크 API

### 서버 상태 확인
```http
GET /health
```

**응답**: `200 OK`
```json
{
  "status": "healthy",
  "timestamp": "2025-10-07T00:00:00Z",
  "version": "0.2.0",
  "database": "connected",
  "object_storage": "connected"
}
```

## 📚 API 문서

### Swagger UI
- **URL**: `http://localhost:8080/swagger-ui/`
- **설명**: 대화형 API 문서

### OpenAPI 스펙
- **URL**: `http://localhost:8080/api-docs/openapi.json`
- **설명**: OpenAPI 3.0 스펙 파일

## ⚠️ 에러 응답

### 표준 에러 형식
```json
{
  "error": "Error Type",
  "message": "Detailed error message",
  "code": "ERROR_CODE",
  "timestamp": "2025-10-07T00:00:00Z"
}
```

### HTTP 상태 코드
- `200 OK` - 성공
- `201 Created` - 생성 성공
- `400 Bad Request` - 잘못된 요청
- `401 Unauthorized` - 인증 실패
- `403 Forbidden` - 권한 없음
- `404 Not Found` - 리소스 없음
- `409 Conflict` - 충돌 (중복)
- `500 Internal Server Error` - 서버 오류

## 🔧 요청/응답 예시

### cURL 예시
```bash
# 어노테이션 생성
curl -X POST http://localhost:8080/api/annotations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "study_instance_uid": "1.2.3.4.5.6.7.8.9.10",
    "series_instance_uid": "1.2.3.4.5.6.7.8.9.11",
    "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
    "annotation_data": {
      "type": "polygon",
      "coordinates": [[100, 200], [150, 250], [200, 200]]
    }
  }'

# 마스크 그룹 생성
curl -X POST http://localhost:8080/api/annotations/1/mask-groups \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "group_name": "Liver Segmentation v1.0",
    "model_name": "UNet3D",
    "version": "1.0.0",
    "modality": "CT",
    "slice_count": 100,
    "mask_type": "segmentation"
  }'
```

---

**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
