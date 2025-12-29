# Note, Report, 가이드 이미지 API 정리

**작성일**: 2025-01-XX  
**버전**: 1.0  
**상태**: 현재 구현 상태 반영

---

## 📋 개요

이 문서는 PACS Extension Server의 Note, Report, 가이드 이미지 관련 API를 정리한 문서입니다.  
현재는 **project_id를 무시하고 데이터-사용자 중심의 API 구조**로 되어 있습니다.

---

## 📝 1. Note API (Series User Note)

### 1.1 전역 Note API (Project ID 무시)

#### Note 생성/수정
```http
PUT /api/series/{series_id}/note
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "content": "Series에 대한 노트 내용",
  "tags": ["tag1", "tag2"]
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "note": {
    "id": 1,
    "series_id": 123,
    "user_id": 456,
    "project_id": null,
    "content": "Series에 대한 노트 내용",
    "tags": ["tag1", "tag2"],
    "created_at": "2025-01-XXT10:00:00Z",
    "updated_at": "2025-01-XXT10:00:00Z"
  }
}
```

#### Note 조회
```http
GET /api/series/{series_id}/note
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "note": {
    "id": 1,
    "series_id": 123,
    "user_id": 456,
    "project_id": null,
    "content": "Series에 대한 노트 내용",
    "tags": ["tag1", "tag2"],
    "created_at": "2025-01-XXT10:00:00Z",
    "updated_at": "2025-01-XXT10:00:00Z"
  }
}
```

#### Series의 모든 Note 조회 (관리자용)
```http
GET /api/series/{series_id}/notes
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "notes": [
    {
      "id": 1,
      "series_id": 123,
      "user_id": 456,
      "project_id": null,
      "content": "Series에 대한 노트 내용",
      "tags": ["tag1", "tag2"],
      "created_at": "2025-01-XXT10:00:00Z",
      "updated_at": "2025-01-XXT10:00:00Z"
    }
  ]
}
```

#### Note 삭제
```http
DELETE /api/series/{series_id}/note
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Note deleted successfully"
}
```

### 1.2 프로젝트 종속 Note API (선택적 사용)

#### Note 생성/수정
```http
PUT /api/project-data/{project_id}/series/{series_id}/note
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

#### Note 조회
```http
GET /api/project-data/{project_id}/series/{series_id}/note
Authorization: Bearer <jwt_token>
```

#### Series의 모든 Note 조회
```http
GET /api/project-data/{project_id}/series/{series_id}/notes
Authorization: Bearer <jwt_token>
```

#### Note 삭제
```http
DELETE /api/project-data/{project_id}/series/{series_id}/note
Authorization: Bearer <jwt_token>
```

---

## 📄 2. Report API (Series User Report)

### 2.1 전역 Report API (Project ID 무시)

#### Report 생성/수정
```http
PUT /api/series/{series_id}/report
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "content": "Report 내용",
  "status": "draft",
  "tags": ["tag1", "tag2"]
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "report": {
    "id": 1,
    "series_id": 123,
    "user_id": 456,
    "project_id": null,
    "content": "Report 내용",
    "status": "draft",
    "tags": ["tag1", "tag2"],
    "created_at": "2025-01-XXT10:00:00Z",
    "updated_at": "2025-01-XXT10:00:00Z"
  }
}
```

#### Report 조회
```http
GET /api/series/{series_id}/report
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "report": {
    "id": 1,
    "series_id": 123,
    "user_id": 456,
    "project_id": null,
    "content": "Report 내용",
    "status": "draft",
    "tags": ["tag1", "tag2"],
    "created_at": "2025-01-XXT10:00:00Z",
    "updated_at": "2025-01-XXT10:00:00Z"
  }
}
```

#### Series의 모든 Report 조회
```http
GET /api/series/{series_id}/reports
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "reports": [
    {
      "id": 1,
      "series_id": 123,
      "user_id": 456,
      "project_id": null,
      "content": "Report 내용",
      "status": "draft",
      "tags": ["tag1", "tag2"],
      "created_at": "2025-01-XXT10:00:00Z",
      "updated_at": "2025-01-XXT10:00:00Z"
    }
  ]
}
```

#### Report 삭제
```http
DELETE /api/series/{series_id}/report
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Report deleted successfully"
}
```

### 2.2 프로젝트 종속 Report API (선택적 사용)

#### Report 생성/수정
```http
PUT /api/project-data/{project_id}/series/{series_id}/report
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

#### Report 조회
```http
GET /api/project-data/{project_id}/series/{series_id}/report
Authorization: Bearer <jwt_token>
```

#### Series의 모든 Report 조회
```http
GET /api/project-data/{project_id}/series/{series_id}/reports
Authorization: Bearer <jwt_token>
```

#### Report 삭제
```http
DELETE /api/project-data/{project_id}/series/{series_id}/report
Authorization: Bearer <jwt_token>
```

### 2.3 Report 확장 기능 API

#### 오디오 파일 업로드 URL 생성 (Dictate)
```http
POST /api/reports/{report_id}/dictate/upload-url
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "file_name": "dictate_audio.mp3",
  "mime_type": "audio/mpeg",
  "file_size": 1024000
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "upload_url": "https://s3.example.com/...",
  "file_path": "reports/{report_id}/dictate/{file_name}",
  "expires_in": 600
}
```

#### 오디오 파일 업로드 완료 처리
```http
POST /api/reports/{report_id}/dictate/complete
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "file_path": "reports/{report_id}/dictate/{file_name}",
  "file_size": 1024000,
  "mime_type": "audio/mpeg"
}
```

#### 템플릿 적용
```http
POST /api/reports/{report_id}/apply-template
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "template_id": 1,
  "variables": {
    "patient_name": "홍길동",
    "study_date": "2025-01-XX"
  }
}
```

#### Report 가이드 목록 조회
```http
GET /api/reports/{report_id}/guides
Authorization: Bearer <jwt_token>
```

#### Report 가이드 추가
```http
POST /api/reports/{report_id}/guides
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "guide_template_id": 1,
  "image_url": "https://s3.example.com/...",
  "display_order": 1
}
```

#### Report 가이드 삭제
```http
DELETE /api/reports/{report_id}/guides/{guide_id}
Authorization: Bearer <jwt_token>
```

---

## 🖼️ 3. 가이드 이미지 API (Report Guide Template)

### 3.1 원본 템플릿 API

#### 템플릿 생성
```http
POST /api/report-guide-templates
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "name": "CT 폐 검사 가이드",
  "modality": "CT",
  "bodypart": "chest",
  "description": "폐 CT 검사 보고서 가이드",
  "template_content": "템플릿 내용...",
  "is_active": true
}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "name": "CT 폐 검사 가이드",
  "modality": "CT",
  "bodypart": "chest",
  "description": "폐 CT 검사 보고서 가이드",
  "template_content": "템플릿 내용...",
  "is_active": true,
  "created_at": "2025-01-XXT10:00:00Z",
  "updated_at": "2025-01-XXT10:00:00Z"
}
```

#### 템플릿 조회
```http
GET /api/report-guide-templates/{template_id}
Authorization: Bearer <jwt_token>
```

#### 템플릿 목록 조회
```http
GET /api/report-guide-templates?modality=CT&bodypart=chest&is_active=true
Authorization: Bearer <jwt_token>
```

**쿼리 파라미터:**
- `modality` (optional): 모달리티 필터
- `bodypart` (optional): 신체 부위 필터
- `is_active` (optional): 활성 상태 필터

#### 템플릿 수정
```http
PUT /api/report-guide-templates/{template_id}
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

#### 템플릿 삭제
```http
DELETE /api/report-guide-templates/{template_id}
Authorization: Bearer <jwt_token>
```

### 3.2 템플릿 이미지 API

#### 이미지 추가
```http
POST /api/report-guide-templates/{template_id}/images
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "image_path": "templates/{template_id}/images/image.png",
  "image_url": "https://s3.example.com/...",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 1,
  "is_shared": true
}
```

#### 이미지 업로드 URL 생성
```http
POST /api/report-guide-templates/{template_id}/images/upload-url
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "file_name": "guide_image.png",
  "mime_type": "image/png"
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "upload_url": "https://s3.example.com/...",
  "file_path": "templates/{template_id}/images/{file_name}",
  "expires_in": 600
}
```

#### 이미지 업로드 완료 처리
```http
POST /api/report-guide-templates/{template_id}/images/complete
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "file_path": "templates/{template_id}/images/{file_name}",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 1,
  "is_shared": true
}
```

#### 이미지 공유 설정 변경
```http
PUT /api/report-guide-templates/{template_id}/images/{image_id}/share
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "is_shared": true
}
```

#### 이미지 삭제
```http
DELETE /api/report-guide-templates/{template_id}/images/{image_id}
Authorization: Bearer <jwt_token>
```

### 3.3 사용자 커스텀 템플릿 API

#### 커스텀 템플릿 생성 (원본 복사)
```http
POST /api/user/custom-report-templates
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "base_template_id": 1,
  "name": "내 커스텀 템플릿",
  "description": "원본 템플릿을 기반으로 생성"
}
```

#### 커스텀 템플릿 생성 (원본 없이)
```http
POST /api/user/custom-report-templates/new
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "name": "새 커스텀 템플릿",
  "modality": "CT",
  "bodypart": "chest",
  "description": "완전히 새로운 템플릿",
  "template_content": "템플릿 내용..."
}
```

#### 커스텀 템플릿 목록 조회
```http
GET /api/user/custom-report-templates
Authorization: Bearer <jwt_token>
```

**응답 (200 OK):**
```json
{
  "success": true,
  "templates": [
    {
      "id": 1,
      "user_id": 456,
      "base_template_id": null,
      "name": "내 커스텀 템플릿",
      "modality": "CT",
      "bodypart": "chest",
      "description": "원본 템플릿을 기반으로 생성",
      "template_content": "템플릿 내용...",
      "created_at": "2025-01-XXT10:00:00Z",
      "updated_at": "2025-01-XXT10:00:00Z"
    }
  ]
}
```

#### 커스텀 템플릿 조회
```http
GET /api/user/custom-report-templates/{template_id}
Authorization: Bearer <jwt_token>
```

#### 커스텀 템플릿 수정
```http
PUT /api/user/custom-report-templates/{template_id}
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

#### 커스텀 템플릿 삭제
```http
DELETE /api/user/custom-report-templates/{template_id}
Authorization: Bearer <jwt_token>
```

#### 커스텀 템플릿 이미지 추가
```http
POST /api/user/custom-report-templates/{template_id}/images
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**요청 본문:**
```json
{
  "image_path": "custom-templates/{template_id}/images/image.png",
  "image_url": "https://s3.example.com/...",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 1
}
```

#### 커스텀 템플릿 이미지 삭제
```http
DELETE /api/user/custom-report-templates/{template_id}/images/{image_id}
Authorization: Bearer <jwt_token>
```

---

## 📊 API 엔드포인트 요약표

### Note API

| Method | Endpoint | Description | Project ID |
|--------|----------|-------------|------------|
| PUT | `/api/series/{series_id}/note` | Note 생성/수정 | ❌ 무시 |
| GET | `/api/series/{series_id}/note` | Note 조회 | ❌ 무시 |
| GET | `/api/series/{series_id}/notes` | 모든 Note 조회 | ❌ 무시 |
| DELETE | `/api/series/{series_id}/note` | Note 삭제 | ❌ 무시 |
| PUT | `/api/project-data/{project_id}/series/{series_id}/note` | Note 생성/수정 | ✅ 사용 |
| GET | `/api/project-data/{project_id}/series/{series_id}/note` | Note 조회 | ✅ 사용 |
| GET | `/api/project-data/{project_id}/series/{series_id}/notes` | 모든 Note 조회 | ✅ 사용 |
| DELETE | `/api/project-data/{project_id}/series/{series_id}/note` | Note 삭제 | ✅ 사용 |

### Report API

| Method | Endpoint | Description | Project ID |
|--------|----------|-------------|------------|
| PUT | `/api/series/{series_id}/report` | Report 생성/수정 | ❌ 무시 |
| GET | `/api/series/{series_id}/report` | Report 조회 | ❌ 무시 |
| GET | `/api/series/{series_id}/reports` | 모든 Report 조회 | ❌ 무시 |
| DELETE | `/api/series/{series_id}/report` | Report 삭제 | ❌ 무시 |
| PUT | `/api/project-data/{project_id}/series/{series_id}/report` | Report 생성/수정 | ✅ 사용 |
| GET | `/api/project-data/{project_id}/series/{series_id}/report` | Report 조회 | ✅ 사용 |
| GET | `/api/project-data/{project_id}/series/{series_id}/reports` | 모든 Report 조회 | ✅ 사용 |
| DELETE | `/api/project-data/{project_id}/series/{series_id}/report` | Report 삭제 | ✅ 사용 |
| POST | `/api/reports/{report_id}/dictate/upload-url` | 오디오 업로드 URL 생성 | - |
| POST | `/api/reports/{report_id}/dictate/complete` | 오디오 업로드 완료 | - |
| POST | `/api/reports/{report_id}/apply-template` | 템플릿 적용 | - |
| GET | `/api/reports/{report_id}/guides` | 가이드 목록 조회 | - |
| POST | `/api/reports/{report_id}/guides` | 가이드 추가 | - |
| DELETE | `/api/reports/{report_id}/guides/{guide_id}` | 가이드 삭제 | - |

### 가이드 이미지 API (Report Guide Template)

| Method | Endpoint | Description | Project ID |
|--------|----------|-------------|------------|
| POST | `/api/report-guide-templates` | 템플릿 생성 | ❌ 무시 |
| GET | `/api/report-guide-templates` | 템플릿 목록 조회 | ❌ 무시 |
| GET | `/api/report-guide-templates/{template_id}` | 템플릿 조회 | ❌ 무시 |
| PUT | `/api/report-guide-templates/{template_id}` | 템플릿 수정 | ❌ 무시 |
| DELETE | `/api/report-guide-templates/{template_id}` | 템플릿 삭제 | ❌ 무시 |
| POST | `/api/report-guide-templates/{template_id}/images` | 이미지 추가 | ❌ 무시 |
| POST | `/api/report-guide-templates/{template_id}/images/upload-url` | 이미지 업로드 URL 생성 | ❌ 무시 |
| POST | `/api/report-guide-templates/{template_id}/images/complete` | 이미지 업로드 완료 | ❌ 무시 |
| PUT | `/api/report-guide-templates/{template_id}/images/{image_id}/share` | 이미지 공유 설정 변경 | ❌ 무시 |
| DELETE | `/api/report-guide-templates/{template_id}/images/{image_id}` | 이미지 삭제 | ❌ 무시 |
| POST | `/api/user/custom-report-templates` | 커스텀 템플릿 생성 (원본 복사) | ❌ 무시 |
| POST | `/api/user/custom-report-templates/new` | 커스텀 템플릿 생성 (신규) | ❌ 무시 |
| GET | `/api/user/custom-report-templates` | 커스텀 템플릿 목록 조회 | ❌ 무시 |
| GET | `/api/user/custom-report-templates/{template_id}` | 커스텀 템플릿 조회 | ❌ 무시 |
| PUT | `/api/user/custom-report-templates/{template_id}` | 커스텀 템플릿 수정 | ❌ 무시 |
| DELETE | `/api/user/custom-report-templates/{template_id}` | 커스텀 템플릿 삭제 | ❌ 무시 |
| POST | `/api/user/custom-report-templates/{template_id}/images` | 커스텀 템플릿 이미지 추가 | ❌ 무시 |
| DELETE | `/api/user/custom-report-templates/{template_id}/images/{image_id}` | 커스텀 템플릿 이미지 삭제 | ❌ 무시 |

---

## 🔐 인증

모든 API는 JWT 토큰 기반 인증을 사용합니다.

```http
Authorization: Bearer <jwt_token>
```

---

## 📝 참고사항

1. **Project ID 무시**: 전역 API (`/api/series/*`, `/api/reports/*`, `/api/report-guide-templates/*`, `/api/user/custom-report-templates/*`)는 project_id를 무시하고 사용자 중심으로 동작합니다.

2. **Project ID 사용**: 프로젝트 종속 API (`/api/project-data/{project_id}/series/*`)는 특정 프로젝트에 종속된 Note/Report를 관리합니다.

3. **사용자 인증**: 모든 API는 JWT 토큰을 통해 사용자를 인증하며, 요청한 사용자의 데이터만 접근할 수 있습니다.

4. **이미지 업로드**: 이미지 업로드는 Signed URL 방식을 사용하며, 먼저 업로드 URL을 생성한 후 클라이언트에서 직접 Object Storage에 업로드하고, 완료 후 서버에 알립니다.

---

## 🔗 관련 문서

- [Annotation API 가이드](../server/technical/ANNOTATION_API_GUIDE.md)
- [API 엔드포인트 참조](../server/technical/API_ENDPOINTS_REFERENCE.md)
- [Swagger UI](http://localhost:8080/swagger-ui/)

