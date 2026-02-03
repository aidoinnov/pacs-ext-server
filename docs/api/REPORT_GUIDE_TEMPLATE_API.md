# Report Guide Template API 가이드

**작성일**: 2026-02-01
**버전**: 3.5
**업데이트**: PUT 수정 시 modalities 반영 (원본·커스텀 템플릿)

---

## 📋 개요

Report Guide Template API는 리포트 작성 시 참고할 수 있는 가이드 템플릿과 이미지를 관리하는 API입니다.

### 핵심 개념

| 구분 | 접근 권한 | 설명 |
|------|-----------|------|
| **원본 템플릿** | 공용 | 관리자가 생성/관리. 모든 사용자가 조회 가능 |
| **커스텀 템플릿** | 본인만 | 사용자가 원본을 수정하면 생성됨. 원본은 그대로 유지되고, 수정본은 본인만 접근 |

**"따로 만든 게 없으면 공용이 나오고"**: 사용자가 커스텀 템플릿을 만들지 않았으면 원본(공용) 템플릿이 표시되고, 커스텀을 만들면 본인용 템플릿이 표시됩니다. 이미지는 템플릿에 연결(매핑)되어 동작합니다.

> **이미지 URL (Signed URL)**: 모든 응답에서 `image_url` 필드는 **다운로드용 Presigned URL (Signed URL)**로 반환됩니다.
>
> - **배경**: 기존에는 S3에 업로드는 되었으나, 조회 응답의 `image_url`이 Signed URL이 아니라 `image_path` 기반이었고, S3 버킷이 비공개 설정이라 클라이언트에서 이미지를 조회·표시할 수 없었습니다.
> - **개선**: 템플릿 이미지, 가이드 이미지, 리포트 가이드 등 **모든 조회 API**에서 `image_url`을 **다운로드용 Presigned URL**로 생성하여 반환합니다. 클라이언트는 이 URL로 직접 이미지를 표시할 수 있습니다.
> - Object Storage(S3/MinIO)에 접근 가능한 임시 URL이며, 일정 시간(예: 1시간) 후 만료됩니다. Object Storage 미설정 시 `image_path` 기반 URL이 반환될 수 있으나, 이 경우 이미지 조회가 불가할 수 있습니다.

### API 구성

1. **S3 이미지 업로드**: 업로드용 Signed URL 획득 → S3에 PUT → 업로드 완료 처리. 이후 조회 시 **다운로드용 Signed URL**이 응답에 포함됨.
2. **이미지 목록**: 기본 템플릿 이미지, 내가 업로드한 이미지
3. **Report Template**: 유효 목록(권장), 원본/커스텀 CRUD
4. **Report-템플릿 매핑**: Report 생성(PUT /api/series/{series_uid}/report) → 템플릿 적용. 리포트는 **사용자+시리즈당 1개**(project와 무관)

---

## 🔑 인증

모든 API는 JWT 토큰 인증이 필요합니다.

```http
Authorization: Bearer <jwt_token>
```

---

## 1. S3 이미지 업로드 API

템플릿과 무관하게 이미지를 먼저 업로드한 후, 템플릿 생성/수정 시 `image_ids`로 연결합니다.

### 1.1 업로드 URL 생성 (Signed URL)

```http
POST /api/guide-images/upload-url
Content-Type: application/json
```

**요청 Body:**
```json
{
  "file_name": "lung_anatomy.png",
  "mime_type": "image/png"
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "upload_url": "https://s3.amazonaws.com/bucket/guide-images/user5/lung_anatomy.png?X-Amz-...",
  "file_path": "guide-images/user5/lung_anatomy.png",
  "expires_in": 600
}
```

**필드 설명:**
- `upload_url`: 업로드용 S3 Presigned URL (이 URL로 PUT 요청으로 직접 파일 업로드)
- `file_path`: 1.2 API 호출 시 전달할 경로
- `expires_in`: URL 만료 시간 (초, 기본 10분)

---

### 1.2 업로드 완료 처리

S3 업로드 완료 후 서버에 이미지 정보를 등록합니다.

```http
POST /api/guide-images/complete
Content-Type: application/json
```

**요청 Body:**
```json
{
  "file_path": "guide-images/user5/lung_anatomy.png",
  "file_size": 2048576,
  "mime_type": "image/png",
  "is_shared": true
}
```

**필드 설명:**
- `is_shared`: `true` = 다른 사용자도 사용 가능 (관리자 공용), `false` = 본인만 (개인용). 기본값: `true`

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Image uploaded successfully",
  "image": {
    "id": 123,
    "image_path": "guide-images/user5/lung_anatomy.png",
    "image_url": "https://s3.amazonaws.com/bucket/guide-images/user5/lung_anatomy.png?X-Amz-Signature=...&X-Amz-Expires=3600",
    "file_size": 2048576,
    "mime_type": "image/png",
    "is_shared": true,
    "uploaded_by": 5,
    "created_at": "2026-02-01T10:00:00Z"
  }
}
```

- `image_url`: **Signed URL** (다운로드용 Presigned URL). 브라우저/앱에서 직접 이미지 조회 가능. 만료 시간 내 유효.
- `image_source`: 항상 `"guide"`. 삭제 시 `DELETE /api/guide-images/{id}` 사용.

---

### 1.3 가이드 이미지 삭제

본인이 업로드한 이미지만 삭제 가능합니다. **`image_source="guide"`인 이미지만** 이 API로 삭제합니다.

```http
DELETE /api/guide-images/{image_id}
```

**image_source 구분**: 응답의 `image_source` 필드로 삭제 API를 선택하세요.
- `image_source="guide"` → `DELETE /api/guide-images/{id}` (이 API)
- `image_source="template"` → `DELETE /api/report-guide-templates/{template_id}/images/{id}` (template_id 필요)
- `image_source="custom_template"` → `DELETE /api/user/custom-report-templates/{custom_template_id}/images/{id}`

**응답 (200 OK):**
```json
{
  "success": true,
  "message": "Image deleted successfully"
}
```

**에러 (403 Forbidden):**
```json
{
  "error": "Forbidden",
  "message": "You can only delete your own images"
}
```

**에러 (404 Not Found):** 해당 ID가 `guide_image` 테이블에 없을 때. `image_source="template"` 이미지는 템플릿별 삭제 API 사용.

---

## 2. 이미지 목록 API

### 2.1 내가 업로드한 이미지 목록

```http
GET /api/guide-images/my-uploads
```

**Query Parameters:**
- `is_shared` (optional): 공유 여부 필터 (`true` / `false`)

**응답 (200 OK):**
```json
{
  "success": true,
  "images": [
    {
      "id": 123,
      "image_path": "guide-images/user5/lung_anatomy.png",
      "image_url": "https://s3.amazonaws.com/bucket/guide-images/user5/lung_anatomy.png?X-Amz-Signature=...&X-Amz-Expires=3600",
      "file_size": 2048576,
      "mime_type": "image/png",
      "is_shared": true,
      "uploaded_by": 5,
      "created_at": "2026-02-01T10:00:00Z"
    }
  ],
  "total_count": 1
}
```

- `image_url`: **Signed URL**로 반환. 이미지 다운로드/표시에 사용.
- `image_source`: 항상 `"guide"`. 삭제 시 `DELETE /api/guide-images/{id}` 사용.

---

### 2.2 기본(원본) 템플릿 이미지 목록

원본 템플릿 상세 조회 시 `images` 배열로 함께 반환됩니다.

```http
GET /api/report-guide-templates/{template_id}
```

응답의 `images` 필드에 해당 템플릿에 연결된 이미지 목록이 포함됩니다. (섹션 3.1.2 참고)

---

## 3. Report Template API

### 3.0 유효 템플릿 목록 (권장)

**따로 수정 안 했으면 원본, 수정했으면 커스텀 + 처음부터 만든 커스텀**을 한 번에 조회합니다. 클라이언트는 이 API만 호출하면 됩니다.

```http
GET /api/user/report-templates
```

**Query Parameters:**
- `modality` (optional): 모달리티 필터
- `bodypart` (optional): 신체 부위 필터

**응답 (200 OK):**
```json
{
  "success": true,
  "templates": [
    {
      "source": "custom",
      "template_id": null,
      "custom_template_id": 5,
      "base_template_id": 1,
      "description": "개인화된 가이드",
      "conclusion": "커스텀 결론",
      "bodypart": "chest",
      "modalities": ["CT"],
      "images": [...],
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-01T10:00:00Z"
    },
    {
      "source": "original",
      "template_id": 2,
      "custom_template_id": null,
      "base_template_id": null,
      "description": "...",
      "images": [...],
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-01T10:00:00Z"
    }
  ]
}
```

**필드 설명:**
- `images[].image_source`: `"guide"` | `"template"`. 삭제 API 선택용. `"guide"`→`/api/guide-images/{id}`, `"template"`→`/api/report-guide-templates/{template_id}/images/{id}`
- `images[].template_id`: `image_source="template"`일 때 삭제 API에 필요한 template_id
- `images[].image_url`: **Signed URL** (다운로드용 Presigned URL)
- `source`: `"original"` (공용) | `"custom"` (본인용)
- `template_id`: 원본일 때 사용. Report-가이드 연결 시 `template_id`로 전달
- `custom_template_id`: 커스텀일 때 사용. Report-가이드 연결 시 `custom_template_id`로 전달
- `base_template_id`: 커스텀이 원본 기반이면 원본 ID, 처음부터 만든 커스텀이면 `null`

---

### 3.1 원본 템플릿 (공용)

관리자만 생성/수정/삭제 가능. 모든 사용자가 조회 가능.

#### 3.1.1 목록 조회

```http
GET /api/report-guide-templates
```

**Query Parameters:**
- `modality` (optional): 모달리티 필터 (예: "CT", "MR")
- `bodypart` (optional): 신체 부위 필터 (예: "chest", "brain")
- `is_active` (optional): 활성화 상태 필터 (true/false)

**응답 (200 OK):**
```json
{
  "success": true,
  "templates": [
    {
      "id": 1,
      "description": "폐 CT 검사 보고서 작성 가이드",
      "conclusion": "결론 템플릿 내용",
      "bodypart": "chest",
      "modalities": ["CT"],
      "images": [
        {
          "id": 123,
          "image_url": "https://s3.amazonaws.com/bucket/...?X-Amz-Signature=...&X-Amz-Expires=3600",
          "display_order": 0,
          "is_shared": true
        }
      ],
      "is_shared": true,
      "is_active": true,
      "created_by": 1,
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-01T10:00:00Z"
    }
  ]
}
```

- `images[].image_url`: **Signed URL**로 반환. 클라이언트에서 이미지 표시 가능.

---

#### 3.1.2 상세 조회

```http
GET /api/report-guide-templates/{template_id}
```

**응답 (200 OK):** 위와 동일한 구조. `images`에 템플릿에 연결된 이미지 목록 포함. 각 이미지의 `image_url`은 **Signed URL**로 반환됨.

---

#### 3.1.3 생성 (관리자)

이미 업로드된 가이드 이미지(1.1, 1.2)를 `image_ids`로 연결하여 생성합니다.

```http
POST /api/report-guide-templates
Content-Type: application/json
```

**요청 Body:**
```json
{
  "description": "폐 CT 검사 보고서 작성 가이드",
  "conclusion": "결론 템플릿 내용",
  "bodypart": "chest",
  "modalities": ["CT"],
  "image_ids": [123, 124],
  "is_shared": true
}
```

- `image_ids` (optional): 1.2 API로 업로드한 이미지 ID 배열. 빈 배열 또는 생략 시 이미지 없이 생성.

---

#### 3.1.4 수정 (관리자)

```http
PUT /api/report-guide-templates/{template_id}
Content-Type: application/json
```

**요청 Body:** (필요한 필드만 전송)
```json
{
  "description": "수정된 설명",
  "conclusion": "수정된 결론",
  "bodypart": "chest",
  "modalities": ["CT", "MR"],
  "image_ids": [123, 125],
  "is_shared": true,
  "is_active": true
}
```

- `modalities` (optional): 지정 시 기존 모달리티를 모두 제거하고 새로 교체. 조회 시 `modalities`에 반영됨.
- `image_ids`: 지정 시 기존 이미지를 모두 제거하고 새로 교체. 빈 배열 시 모든 이미지 제거.

---

#### 3.1.5 삭제 (관리자)

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

### 3.2 커스텀 템플릿 (본인만)

**원본을 수정하면**: 원본은 그대로 두고, 사용자 전용 커스텀 템플릿이 생성됩니다. 본인만 조회/수정 가능.

**따로 만든 게 없으면**: 원본(공용) 템플릿이 표시됩니다.

#### 3.2.1 목록 조회

```http
GET /api/user/custom-report-templates
```

**Query Parameters:**
- `modality` (optional), `bodypart` (optional)

**응답 (200 OK):**
```json
{
  "success": true,
  "templates": [
    {
      "id": 1,
      "user_id": 1,
      "base_template_id": 1,
      "description": "개인화된 가이드",
      "conclusion": "커스텀 결론",
      "bodypart": "chest",
      "modalities": ["CT"],
      "images": [...],
      "created_at": "2026-02-01T10:00:00Z",
      "updated_at": "2026-02-01T10:00:00Z"
    }
  ]
}
```

- `images[].image_url`: **Signed URL**로 반환. 이미지 표시에 사용.

---

#### 3.2.2 상세 조회

```http
GET /api/user/custom-report-templates/{template_id}
```

본인 소유 템플릿만 조회 가능. `images[].image_url`은 **Signed URL**로 반환됨.

---

#### 3.2.3 원본에서 커스텀 생성

원본 템플릿을 기반으로 본인용 커스텀 템플릿을 생성합니다. 원본 이미지를 참조하거나, `image_ids`로 다른 이미지 사용 가능.

```http
POST /api/user/custom-report-templates
Content-Type: application/json
```

**요청 Body:**
```json
{
  "base_template_id": 1,
  "description": "개인화된 가이드",
  "conclusion": "커스텀 결론",
  "image_ids": [123, 124]
}
```

- `image_ids` (optional): 1.2 API로 업로드한 이미지 ID. 생략 시 원본의 이미지를 그대로 참조.

---

#### 3.2.4 새 커스텀 템플릿 생성

원본 없이 처음부터 생성합니다.

```http
POST /api/user/custom-report-templates/new
Content-Type: application/json
```

**요청 Body:**
```json
{
  "description": "처음부터 만든 가이드",
  "conclusion": "나만의 결론",
  "bodypart": "chest",
  "modalities": ["CT", "MR"],
  "image_ids": [125, 126]
}
```

---

#### 3.2.5 수정

```http
PUT /api/user/custom-report-templates/{template_id}
Content-Type: application/json
```

**요청 Body:**
```json
{
  "description": "수정된 설명",
  "conclusion": "수정된 결론",
  "bodypart": "abdomen",
  "modalities": ["CT", "MR"],
  "image_ids": [127, 128]
}
```

- `modalities` (optional): 지정 시 기존 모달리티를 모두 제거하고 새로 교체. 조회 시 `modalities`에 반영됨.

---

#### 3.2.6 삭제

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

### 3.3 하위 호환: 템플릿별 이미지 업로드 (DEPRECATED)

> ⚠️ **DEPRECATED**: 1.1, 1.2 API 사용을 권장합니다.

- `POST /api/report-guide-templates/{template_id}/images/upload-url`
- `POST /api/report-guide-templates/{template_id}/images/complete`
- `PUT /api/report-guide-templates/{template_id}/images/{image_id}/share`
- `DELETE /api/report-guide-templates/{template_id}/images/{image_id}`

### 3.4 하위 호환: 커스텀 템플릿 이미지 추가 (DEPRECATED)

> ⚠️ **DEPRECATED**: 1.1, 1.2로 이미지 업로드 후 `image_ids`로 연결하세요.

- `POST /api/user/custom-report-templates/{template_id}/images`
- `DELETE /api/user/custom-report-templates/{template_id}/images/{image_id}`

---

## 4. Report-템플릿 매핑 API

Report에 원본 또는 커스텀 템플릿을 적용합니다. **리포트 1개 = 템플릿 1개**이며, 적용 시점의 이미지는 스냅샷되어 템플릿 변경과 무관하게 유지됩니다.

> **리포트 식별**: Report는 **사용자(user_id) + 시리즈(series_id)** 당 1개입니다. **project와 무관**합니다.  
> **접근 권한**: 해당 Report의 `user_id`와 일치하는 인증 사용자만 §4.1~§4.3 API를 호출할 수 있습니다.

### 4.0 Report 생성/조회 (선행 조건)

§4.1~§4.3을 사용하려면 먼저 Report를 생성하고 `report_id`를 확보해야 합니다.

```http
PUT /api/series/{series_uid}/report
Content-Type: application/json
```

**요청 Body:**
```json
{
  "status": "unread",
  "description": "리포트 내용",
  "conclusion": "결론",
  "bodypart": "chest"
}
```

**응답 (200 OK):**
```json
{
  "success": true,
  "id": 123,
  "description": "리포트 내용",
  "conclusion": "결론"
}
```

- `id`: Report ID. §4.1~§4.3 호출 시 `report_id`로 사용.
- `series_uid`: DICOM Series Instance UID (project_data_series에 등록된 시리즈)

**Report 조회:**
```http
GET /api/series/{series_uid}/report
```

동일한 형식으로 응답하며, `id`가 Report ID입니다.

### 4.1 Report에 템플릿 적용

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

- `template_id` 또는 `custom_template_id` 중 하나만 지정.
- 기존에 적용된 템플릿이 있으면 **덮어씀** (이미지 스냅샷도 갱신).

### 4.2 Report의 템플릿·이미지 조회

```http
GET /api/reports/{report_id}/guides
```

**응답 예시** (가이드 0개 또는 1개, 이미지 스냅샷 포함):
```json
{
  "success": true,
  "guides": [
    {
      "id": 123,
      "report_id": 123,
      "template_id": 5,
      "custom_template_id": null,
      "display_order": 0,
      "images": [
        {
          "id": 101,
          "image_path": "...",
          "image_url": "https://s3.amazonaws.com/bucket/...?X-Amz-Signature=...&X-Amz-Expires=3600",
          "display_order": 0
        }
      ],
      "created_at": "..."
    }
  ]
}
```

- `images`: 리포트에 스냅샷된 가이드 이미지 (템플릿 수정과 무관). `image_url`은 **Signed URL**로 반환됨.

### 4.3 Report에서 템플릿 제거

```http
DELETE /api/reports/{report_id}/guides/{guide_id}
```

- 1:1 구조이므로 `guide_id`는 `report_id`와 동일. 템플릿 및 이미지 스냅샷 제거.

---

## 5. 전체 워크플로우 (권장)

```
1. 이미지 업로드
   POST /api/guide-images/upload-url  →  S3에 PUT  →  POST /api/guide-images/complete

2. 내 이미지 목록 확인
   GET /api/guide-images/my-uploads   ← image_url이 Signed URL로 반환, 이미지 표시 가능

3. 템플릿 생성/수정
   - 관리자: POST/PUT /api/report-guide-templates (modalities, image_ids)
   - 사용자: POST/PUT /api/user/custom-report-templates (base_template_id, modalities, image_ids)

4. 템플릿 조회 (클라이언트용)
   GET /api/user/report-templates   ← 권장 (원본+커스텀 병합, images[].image_url은 Signed URL)

   또는 개별 조회:
   - 공용: GET /api/report-guide-templates
   - 본인용: GET /api/user/custom-report-templates

5. Report 생성 및 템플릿 적용
   PUT /api/series/{series_uid}/report  →  응답의 id를 report_id로 사용
   POST /api/reports/{report_id}/guides  →  template_id 또는 custom_template_id로 적용
   GET /api/reports/{report_id}/guides   →  적용된 템플릿·이미지 스냅샷 조회 (image_url은 Signed URL로 즉시 표시 가능)
```

---

## 6. 에러 응답

| 코드 | 설명 |
|------|------|
| 400 | Bad Request - 잘못된 요청 |
| 401 | Unauthorized - 인증 실패 |
| 403 | Forbidden - 권한 없음 |
| 404 | Not Found - 리소스 없음 |
| 500 | Internal Server Error |

---

## 7. 참고사항

### 이미지 URL (Signed URL)

**업로드 vs 조회**:
- **업로드** (1.1): `upload_url`은 업로드용 Presigned URL (PUT). S3에 파일을 올리는 데 사용.
- **조회**: `image_url`은 **다운로드용 Presigned URL** (GET). 이미지를 표시·다운로드하는 데 사용.

**개선 내용** (v3.3):
- 기존: S3 업로드는 되었으나, 조회 시 `image_url`이 Signed URL이 아니라서 S3 비공개 버킷 환경에서 **이미지가 표시되지 않음**.
- 개선: 모든 조회 API(가이드 이미지, 템플릿 이미지, 리포트 가이드)에서 `image_url`을 **다운로드용 Presigned URL**로 생성하여 반환. 클라이언트에서 바로 이미지 표시 가능.

**형식**: S3/MinIO Presigned URL (예: `?X-Amz-Signature=...&X-Amz-Expires=3600`). 만료 시간(예: 1시간) 내 유효. Object Storage 미설정 시 `image_path` 기반 URL이 반환될 수 있으나, 이미지 조회가 되려면 Object Storage 설정이 필요합니다.

### 이미지 업로드 제한
- **최대 파일 크기**: 10MB
- **지원 형식**: PNG, JPEG, GIF
- **업로드 URL 유효시간**: 10분 (600초)

### 권한 요약
- **관리자 공통 이미지** (`is_shared=true`): 모든 사용자 접근 가능
- **템플릿에 지정된 이미지**: 해당 템플릿 조회 시 모두 접근 가능
- **개인 이미지** (`is_shared=false`): 업로더 본인만 접근/사용 가능

### 이미지 출처 (image_source) 및 삭제 API
- 모든 이미지 응답에 `image_source` 포함. ID 충돌 방지를 위해 삭제 시 반드시 구분.
- `"guide"`: `guide_image` 테이블. `DELETE /api/guide-images/{id}` 사용.
- `"template"`: `report_guide_template_image` 테이블(기존 구조). `DELETE /api/report-guide-templates/{template_id}/images/{id}` 사용. `template_id`는 응답에 포함.
- `"custom_template"`: `user_custom_template_image` 테이블. `DELETE /api/user/custom-report-templates/{custom_template_id}/images/{id}` 사용.

### 데이터 구조
- `guide_image`: 독립 이미지 (여러 템플릿에서 재사용)
- `report_guide_template`: 원본(공용) 템플릿
- `report_guide_template_image_mapping`: 원본 ↔ 이미지 연결
- `user_custom_report_template`: 사용자별 커스텀 템플릿
- `user_custom_template_image_mapping`: 커스텀 ↔ 이미지 연결
- `series_user_report`: template_id, custom_template_id (리포트당 1개)
- `report_image`: 리포트의 가이드 이미지 스냅샷 (템플릿 변경과 무관)

---

## 부록 A. 구현 검토 (2026-02-01)

### 요구사항 대비 구현 상태

| 요구사항 | 구현 | 검증 |
|----------|------|------|
| 1. S3 이미지 업로드 (업로드용 Signed URL, 완료) | `POST /api/guide-images/upload-url`, `complete` | OK |
| 1-b. 조회 시 image_url Signed URL 반환 | 가이드 이미지, 템플릿, 리포트 가이드 등 모든 조회 API | OK |
| 2. 내가 업로드한 이미지 목록 | `GET /api/guide-images/my-uploads` | OK |
| 3. 기본 템플릿 이미지 목록 | 템플릿 상세 조회 `images` | OK |
| 4. 원본(공용) 템플릿 CRUD | `GET/POST/PUT/DELETE /api/report-guide-templates` | OK |
| 5. 커스텀(본인) 템플릿 CRUD | `GET/POST/PUT/DELETE /api/user/custom-report-templates` | OK |
| 6. 원본 수정 시 커스텀 생성 | `POST .../custom-report-templates` (base_template_id) → 원본 유지, 본인용 복사본 생성 | OK |
| 7. 이미지 권한: 관리자 공통 | `is_shared=true` → 모두 접근 | OK |
| 8. 이미지 권한: 템플릿 지정 | 템플릿에 연결된 이미지 → 조회 시 모두 노출 (필터 없음) | OK |
| 9. 이미지 권한: 개인 | `is_shared=false` → 본인만 (update_mappings에서 타인 private 사용 불가) | OK |
| 10. 하위 호환 (deprecated) | 기존 템플릿별 업로드 API 이미지 병합 반환 | OK |
| 11. 유효 템플릿 통합 조회 | `GET /api/user/report-templates` (원본+커스텀 병합) | OK |
| 12. Report 생성·조회 | `PUT/GET /api/series/{series_uid}/report` (사용자+시리즈당 1개) | OK |
| 13. Report-템플릿 적용 | `POST /api/reports/{id}/guides` (1:1, 이미지 스냅샷) | OK |
| 14. 모달리티 PUT 반영 | 원본·커스텀 템플릿 PUT 시 `modalities` 지정 → 조회 시 반영 | OK |

### 참고
- **`GET /api/user/report-templates`** 사용 시 "따로 만든 게 없으면 공용" 로직을 서버에서 처리하므로 클라이언트는 한 번의 호출로 충분합니다.
- Report-가이드 매핑(`/api/reports/{id}/guides`)은 `PUT/GET /api/series/{series_uid}/report`로 생성·조회한 Report와 연동됩니다.
- **리포트 식별**: 사용자+시리즈당 1개이며, project와 무관합니다.

---

**마지막 업데이트**: 2026-02-01 (v3.5: PUT 수정 시 modalities 반영)
