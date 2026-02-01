# Report Guide Template API 가이드

**작성일**: 2026-01-29  
**버전**: 1.0

---

## 📋 개요

Report Guide Template API는 리포트 작성 시 참고할 수 있는 가이드 템플릿과 이미지를 관리하는 API입니다.

### 주요 기능
- ✅ 원본 템플릿 관리 (관리자용)
- ✅ 사용자 커스텀 템플릿 관리
- ✅ 가이드 이미지 업로드 및 관리
- ✅ Report에 가이드 이미지 연결

---

## 🔑 인증

모든 API는 JWT 토큰 인증이 필요합니다.

```http
Authorization: Bearer <jwt_token>
```

---

## 📚 1. 원본 템플릿 API

### 1.1 템플릿 목록 조회

```http
GET /api/report-guide-templates
```

**Query Parameters:**
- `modality` (optional): 모달리티 필터 (예: "CT", "MR")
- `bodypart` (optional): 신체 부위 필터 (예: "chest", "brain")
- `is_active` (optional): 활성화 상태 필터 (true/false)

**응답 (200 OK):**
```json
[
  {
    "id": 1,
    "name": "CT 폐 검사 가이드",
    "description": "폐 CT 검사 보고서 작성 가이드",
    "conclusion": "결론 템플릿 내용",
    "bodypart": "chest",
    "modalities": ["CT"],
    "images": [
      {
        "id": 1,
        "image_url": "https://s3.example.com/templates/1/images/guide1.png",
        "display_order": 0,
        "is_shared": true
      }
    ],
    "is_shared": true,
    "is_active": true,
    "created_by": 1,
    "created_at": "2026-01-29T10:00:00Z",
    "updated_at": "2026-01-29T10:00:00Z"
  }
]
```

---

### 1.2 템플릿 상세 조회

```http
GET /api/report-guide-templates/{template_id}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "name": "CT 폐 검사 가이드",
  "description": "폐 CT 검사 보고서 작성 가이드",
  "conclusion": "결론 템플릿 내용",
  "bodypart": "chest",
  "modalities": ["CT"],
  "images": [
    {
      "id": 1,
      "image_url": "https://s3.example.com/templates/1/images/guide1.png",
      "image_path": "templates/1/images/guide1.png",
      "file_size": 1024000,
      "mime_type": "image/png",
      "display_order": 0,
      "is_shared": true,
      "uploaded_by": 1,
      "created_at": "2026-01-29T10:00:00Z"
    }
  ],
  "is_shared": true,
  "is_active": true,
  "created_by": 1,
  "created_at": "2026-01-29T10:00:00Z",
  "updated_at": "2026-01-29T10:00:00Z"
}
```

---

### 1.3 템플릿 생성 (관리자)

```http
POST /api/report-guide-templates
Content-Type: application/json
```

**요청 Body:**
```json
{
  "name": "CT 폐 검사 가이드",
  "description": "폐 CT 검사 보고서 작성 가이드",
  "conclusion": "결론 템플릿 내용",
  "bodypart": "chest",
  "modalities": ["CT"],
  "is_shared": true
}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "name": "CT 폐 검사 가이드",
  "description": "폐 CT 검사 보고서 작성 가이드",
  "conclusion": "결론 템플릿 내용",
  "bodypart": "chest",
  "modalities": ["CT"],
  "images": [],
  "is_shared": true,
  "is_active": true,
  "created_by": 1,
  "created_at": "2026-01-29T10:00:00Z",
  "updated_at": "2026-01-29T10:00:00Z"
}
```

---

### 1.4 템플릿 수정 (관리자)

```http
PUT /api/report-guide-templates/{template_id}
Content-Type: application/json
```

**요청 Body:**
```json
{
  "name": "CT 폐 검사 가이드 (수정)",
  "description": "수정된 설명",
  "conclusion": "수정된 결론",
  "bodypart": "chest",
  "is_shared": true,
  "is_active": true
}
```

---

### 1.5 템플릿 삭제 (관리자)

```http
DELETE /api/report-guide-templates/{template_id}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Template deleted successfully"
}
```

---

## 🖼️ 2. 가이드 이미지 업로드 API

### 2.1 이미지 업로드 URL 생성

```http
POST /api/report-guide-templates/{template_id}/images/upload-url
Content-Type: application/json
```

**요청 Body:**
```json
{
  "file_name": "guide_image.png",
  "mime_type": "image/png",
  "file_size": 1024000
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "upload_url": "https://s3.example.com/presigned-upload-url?...",
  "file_path": "templates/1/images/guide_image.png",
  "expires_in": 600
}
```

---

### 2.2 이미지 업로드 완료

S3에 이미지 업로드 후 호출하여 DB에 이미지 정보를 저장합니다.

```http
POST /api/report-guide-templates/{template_id}/images/complete
Content-Type: application/json
```

**요청 Body:**
```json
{
  "file_path": "templates/1/images/guide_image.png",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 0,
  "is_shared": true
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Image uploaded and added to template successfully",
  "image": {
    "id": 1,
    "template_id": 1,
    "image_url": "https://s3.example.com/templates/1/images/guide_image.png",
    "image_path": "templates/1/images/guide_image.png",
    "file_size": 1024000,
    "mime_type": "image/png",
    "display_order": 0,
    "is_shared": true,
    "uploaded_by": 1,
    "created_at": "2026-01-29T10:00:00Z"
  }
}
```

---

### 2.3 이미지 목록 조회

템플릿의 이미지 목록은 템플릿 상세 조회 시 함께 반환됩니다.

```http
GET /api/report-guide-templates/{template_id}
```

응답의 `images` 배열에 이미지 목록이 포함됩니다.

---

### 2.4 이미지 삭제

```http
DELETE /api/report-guide-templates/{template_id}/images/{image_id}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Image deleted successfully"
}
```

---

### 2.5 이미지 공유 상태 업데이트

```http
PUT /api/report-guide-templates/{template_id}/images/{image_id}/share
Content-Type: application/json
```

**요청 Body:**
```json
{
  "is_shared": true
}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "template_id": 1,
  "image_url": "https://s3.example.com/templates/1/images/guide_image.png",
  "image_path": "templates/1/images/guide_image.png",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 0,
  "is_shared": true,
  "uploaded_by": 1,
  "created_at": "2026-01-29T10:00:00Z"
}
```

---

## 👤 3. 사용자 커스텀 템플릿 API

### 3.1 커스텀 템플릿 목록 조회

```http
GET /api/user/custom-report-templates
```

**Query Parameters:**
- `modality` (optional): 모달리티 필터
- `bodypart` (optional): 신체 부위 필터

**응답 (200 OK):**
```json
[
  {
    "id": 1,
    "user_id": 1,
    "base_template_id": 1,
    "name": "내 CT 폐 검사 가이드",
    "description": "개인화된 가이드",
    "conclusion": "커스텀 결론",
    "bodypart": "chest",
    "modalities": ["CT"],
    "images": [
      {
        "id": 1,
        "image_url": "https://s3.example.com/custom/1/images/my_guide.png",
        "display_order": 0
      }
    ],
    "created_at": "2026-01-29T10:00:00Z",
    "updated_at": "2026-01-29T10:00:00Z"
  }
]
```

---

### 3.2 커스텀 템플릿 상세 조회

```http
GET /api/user/custom-report-templates/{template_id}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "user_id": 1,
  "base_template_id": 1,
  "name": "내 CT 폐 검사 가이드",
  "description": "개인화된 가이드",
  "conclusion": "커스텀 결론",
  "bodypart": "chest",
  "modalities": ["CT"],
  "images": [
    {
      "id": 1,
      "custom_template_id": 1,
      "image_url": "https://s3.example.com/custom/1/images/my_guide.png",
      "image_path": "custom/1/images/my_guide.png",
      "file_size": 512000,
      "mime_type": "image/png",
      "display_order": 0,
      "uploaded_by": 1,
      "created_at": "2026-01-29T10:00:00Z"
    }
  ],
  "created_at": "2026-01-29T10:00:00Z",
  "updated_at": "2026-01-29T10:00:00Z"
}
```

---

### 3.3 원본 템플릿에서 커스텀 템플릿 생성

```http
POST /api/user/custom-report-templates
Content-Type: application/json
```

**요청 Body:**
```json
{
  "base_template_id": 1,
  "name": "내 CT 폐 검사 가이드",
  "description": "개인화된 가이드",
  "conclusion": "커스텀 결론"
}
```

**응답 (200 OK):**
```json
{
  "id": 1,
  "user_id": 1,
  "base_template_id": 1,
  "name": "내 CT 폐 검사 가이드",
  "description": "개인화된 가이드",
  "conclusion": "커스텀 결론",
  "bodypart": "chest",
  "modalities": ["CT"],
  "images": [],
  "created_at": "2026-01-29T10:00:00Z",
  "updated_at": "2026-01-29T10:00:00Z"
}
```

---

### 3.4 새로운 커스텀 템플릿 생성

```http
POST /api/user/custom-report-templates/new
Content-Type: application/json
```

**요청 Body:**
```json
{
  "name": "나만의 가이드",
  "description": "처음부터 만든 가이드",
  "conclusion": "나만의 결론",
  "bodypart": "chest",
  "modalities": ["CT", "MR"]
}
```

---

### 3.5 커스텀 템플릿 수정

```http
PUT /api/user/custom-report-templates/{template_id}
Content-Type: application/json
```

**요청 Body:**
```json
{
  "name": "수정된 가이드",
  "description": "수정된 설명",
  "conclusion": "수정된 결론"
}
```

---

### 3.6 커스텀 템플릿 삭제

```http
DELETE /api/user/custom-report-templates/{template_id}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Custom template deleted successfully"
}
```

---

### 3.7 커스텀 템플릿 이미지 추가

```http
POST /api/user/custom-report-templates/{template_id}/images
Content-Type: application/json
```

**요청 Body:**
```json
{
  "image_path": "custom/1/images/my_guide.png",
  "image_url": "https://s3.example.com/custom/1/images/my_guide.png",
  "file_size": 512000,
  "mime_type": "image/png",
  "display_order": 0
}
```

---

### 3.8 커스텀 템플릿 이미지 삭제

```http
DELETE /api/user/custom-report-templates/{template_id}/images/{image_id}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Custom template image deleted successfully"
}
```

---

## 🔗 4. Report-가이드 매핑 API

### 4.1 Report에 가이드 추가

```http
POST /api/reports/{report_id}/guides
Content-Type: application/json
```

**요청 Body:**
```json
{
  "template_id": 1,
  "custom_template_id": null,
  "display_order": 0
}
```

**참고:** `template_id` 또는 `custom_template_id` 중 하나만 지정합니다.

**응답 (200 OK):**
```json
{
  "id": 1,
  "report_id": 1,
  "template_id": 1,
  "custom_template_id": null,
  "display_order": 0,
  "created_at": "2026-01-29T10:00:00Z"
}
```

---

### 4.2 Report의 가이드 목록 조회

```http
GET /api/reports/{report_id}/guides
```

**응답 (200 OK):**
```json
[
  {
    "id": 1,
    "report_id": 1,
    "template_id": 1,
    "custom_template_id": null,
    "display_order": 0,
    "created_at": "2026-01-29T10:00:00Z"
  },
  {
    "id": 2,
    "report_id": 1,
    "template_id": null,
    "custom_template_id": 5,
    "display_order": 1,
    "created_at": "2026-01-29T10:00:00Z"
  }
]
```

---

### 4.3 Report에서 가이드 삭제

```http
DELETE /api/reports/{report_id}/guides/{guide_id}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Guide removed from report successfully"
}
```

---

## 📋 5. 완전한 워크플로우 예제

### 시나리오: 원본 템플릿에 이미지 업로드

#### Step 1: 업로드 URL 생성
```http
POST /api/report-guide-templates/1/images/upload-url
Content-Type: application/json

{
  "file_name": "guide_image.png",
  "mime_type": "image/png",
  "file_size": 1024000
}
```

**응답:**
```json
{
  "success": true,
  "upload_url": "https://s3.example.com/presigned-upload-url?...",
  "file_path": "templates/1/images/guide_image.png",
  "expires_in": 600
}
```

#### Step 2: S3에 이미지 업로드
```http
PUT https://s3.example.com/presigned-upload-url?...
Content-Type: image/png

[Binary image data]
```

#### Step 3: 업로드 완료 알림
```http
POST /api/report-guide-templates/1/images/complete
Content-Type: application/json

{
  "file_path": "templates/1/images/guide_image.png",
  "file_size": 1024000,
  "mime_type": "image/png",
  "display_order": 0,
  "is_shared": true
}
```

**응답:**
```json
{
  "success": true,
  "message": "Image uploaded and added to template successfully",
  "image": {
    "id": 1,
    "template_id": 1,
    "image_url": "https://s3.example.com/templates/1/images/guide_image.png",
    "image_path": "templates/1/images/guide_image.png",
    "file_size": 1024000,
    "mime_type": "image/png",
    "display_order": 0,
    "is_shared": true,
    "uploaded_by": 1,
    "created_at": "2026-01-29T10:00:00Z"
  }
}
```

---

## ⚠️ 6. 에러 응답

### 400 Bad Request
```json
{
  "error": "Bad Request",
  "message": "Invalid request parameters"
}
```

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing authorization token"
}
```

### 404 Not Found
```json
{
  "error": "Not Found",
  "message": "Template not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal Server Error",
  "message": "An unexpected error occurred"
}
```

---

## 📝 7. 참고사항

### 이미지 업로드 제한
- **최대 파일 크기**: 10MB
- **지원 형식**: PNG, JPEG, GIF
- **업로드 URL 유효시간**: 10분 (600초)

### 템플릿 권한
- **원본 템플릿**: 관리자만 생성/수정/삭제 가능
- **커스텀 템플릿**: 각 사용자가 자신의 템플릿만 관리 가능
- **이미지 공유**: `is_shared=true`인 이미지는 다른 사용자도 볼 수 있음

### 데이터베이스 테이블
- `report_guide_template`: 원본 템플릿
- `report_guide_template_modality`: 템플릿-모달리티 매핑
- `report_guide_template_image`: 원본 템플릿 이미지
- `user_custom_report_template`: 사용자 커스텀 템플릿
- `user_custom_template_image`: 커스텀 템플릿 이미지
- `series_user_report_guide`: Report-가이드 매핑

---

**문서 작성 완료** ✅
**마지막 업데이트**: 2026-01-29
