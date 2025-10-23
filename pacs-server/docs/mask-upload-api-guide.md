# 마스크 업로드 및 조회 API 가이드

## 📋 개요

이 문서는 PACS Extension Server의 마스크 업로드 및 조회 API 사용법을 설명합니다. 클라이언트 개발자가 마스크 파일을 업로드하고 조회하는 방법을 단계별로 안내합니다.

## 🔗 기본 정보

- **Base URL**: `http://localhost:8080` (개발 환경)
- **API Base**: `/api`
- **인증 방식**: JWT Bearer Token
- **Content-Type**: `application/json`

## 🔐 인증

모든 API 요청에는 JWT 토큰이 필요합니다.

### 로그인
```http
POST /api/auth/login
Content-Type: application/json

{
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "TestUser2",
  "email": "user2@example.com"
}
```

**응답:**
```json
{
  "user_id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "TestUser2",
  "email": "user2@example.com",
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

## 📝 마스크 업로드 워크플로우

### 1단계: 어노테이션 생성

```http
POST /api/annotations
Authorization: Bearer {token}
Content-Type: application/json

{
  "project_id": 1,
  "study_instance_uid": "1.2.3.4.5.6.7.8.9.10",
  "series_instance_uid": "1.2.3.4.5.6.7.8.9.11",
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
  "annotation_data": {
    "type": "point",
    "coordinates": [100, 200],
    "label": "Test annotation"
  },
  "is_shared": false,
  "viewer_software": "ohif",
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "values": [42.3, 18.7],
      "unit": "mm"
    }
  ]
}
```

**응답:**
```json
{
  "id": 121,
  "user_id": 1,
  "study_instance_uid": "1.2.3.4.5.6.7.8.9.10",
  "series_instance_uid": "1.2.3.4.5.6.7.8.9.11",
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
  "annotation_data": {
    "coordinates": [100, 200],
    "label": "Test annotation",
    "type": "point"
  },
  "tool_name": "manual",
  "tool_version": null,
  "viewer_software": "ohif",
  "description": null,
  "measurement_values": [
    {
      "id": "m1",
      "type": "raw",
      "unit": "mm",
      "values": [42.3, 18.7]
    }
  ],
  "created_at": "2025-10-23T04:26:30.477643Z",
  "updated_at": "2025-10-23T04:26:30.477643Z"
}
```

### 2단계: 마스크 그룹 생성

```http
POST /api/annotations/{annotation_id}/mask-groups
Authorization: Bearer {token}
Content-Type: application/json

{
  "group_name": "Liver Segmentation v2",
  "model_name": "DeepLabV3+",
  "version": "1.0.0",
  "modality": "CT",
  "slice_count": 120,
  "mask_type": "segmentation",
  "description": "Liver segmentation using DeepLabV3+ model"
}
```

**응답:**
```json
{
  "id": 10,
  "annotation_id": 121,
  "group_name": "Liver Segmentation v2",
  "model_name": "DeepLabV3+",
  "version": "1.0.0",
  "modality": "CT",
  "slice_count": 120,
  "mask_type": "segmentation",
  "description": "Liver segmentation using DeepLabV3+ model",
  "created_by": 1,
  "created_at": "2025-10-23 04:26:30.569119 UTC",
  "updated_at": "2025-10-23 04:26:30.569119 UTC"
}
```

### 3단계: 업로드 URL 생성

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/upload-url
Authorization: Bearer {token}
Content-Type: application/json

{
  "mask_group_id": 10,
  "filename": "slice_001_liver.png",
  "mime_type": "image/png",
  "expires_in": 3600
}
```

**응답:**
```json
{
  "upload_url": "https://pacs-masks.s3.ap-northeast-2.amazonaws.com/masks/annotation_0/group_10/slice_001_liver.png?x-id=PutObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&...",
  "download_url": "https://pacs-masks.s3.ap-northeast-2.amazonaws.com/masks/annotation_0/group_10/slice_001_liver.png?x-id=PutObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&...",
  "file_path": "masks/annotation_0/group_10/slice_001_liver.png",
  "expires_in": 600,
  "expires_at": "2025-10-23 04:36:30.678557971 UTC"
}
```

### 4단계: 파일 업로드 (S3에 직접 업로드)

```http
PUT {upload_url}
Content-Type: image/png
x-amz-acl: private
x-amz-meta-annotation_id: 121
x-amz-meta-file_type: mask
x-amz-meta-mask_group_id: 10
x-amz-meta-user_id: 1
x-amz-storage-class: STANDARD

[파일 바이너리 데이터]
```

**응답:**
```
HTTP/1.1 200 OK
```

### 5단계: 업로드 완료 확인

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/complete-upload
Authorization: Bearer {token}
Content-Type: application/json

{
  "mask_group_id": 10,
  "slice_count": 1,
  "labels": ["liver"],
  "uploaded_files": ["masks/annotation_0/group_10/slice_001_liver.png"]
}
```

**응답:**
```json
{
  "success": true,
  "status": "success",
  "processed_masks": 1,
  "uploaded_files": ["masks/annotation_0/group_10/slice_001_liver.png"],
  "message": "Upload completed successfully"
}
```

### 6단계: 마스크 생성

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/masks
Authorization: Bearer {token}
Content-Type: application/json

{
  "mask_group_id": 10,
  "slice_index": 1,
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13",
  "label_name": "liver",
  "file_path": "masks/annotation_0/group_10/slice_001_liver.png",
  "mime_type": "image/png",
  "file_size": 1024000,
  "checksum": "sha256:abc123def456",
  "width": 512,
  "height": 512
}
```

**응답:**
```json
{
  "id": 4,
  "mask_group_id": 10,
  "slice_index": 1,
  "sop_instance_uid": "1.2.3.4.5.6.7.8.9.13",
  "label_name": "liver",
  "file_path": "masks/annotation_0/group_10/slice_001_liver.png",
  "mime_type": "image/png",
  "file_size": 1024000,
  "checksum": "sha256:abc123def456",
  "width": 512,
  "height": 512,
  "created_at": "2025-10-23 04:26:30.879769 UTC",
  "updated_at": "2025-10-23 04:26:30.879769 UTC"
}
```

## 📦 배치 업로드 워크플로우

여러 파일을 한 번에 업로드하는 경우:

### 1. 여러 업로드 URL 생성

```javascript
const files = [
  { filename: "slice_001_liver.png", mime_type: "image/png" },
  { filename: "slice_002_liver.png", mime_type: "image/png" },
  { filename: "slice_003_liver.png", mime_type: "image/png" }
];

const uploadUrls = [];
for (const file of files) {
  const response = await fetch(`/api/annotations/${annotationId}/mask-groups/${groupId}/upload-url`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      mask_group_id: groupId,
      filename: file.filename,
      mime_type: file.mime_type,
      expires_in: 3600
    })
  });
  
  const uploadInfo = await response.json();
  uploadUrls.push(uploadInfo);
}
```

### 2. 병렬 파일 업로드

```javascript
const uploadPromises = uploadUrls.map(async (uploadInfo, index) => {
  const file = files[index];
  const fileData = await readFile(file.filename);
  
  return fetch(uploadInfo.upload_url, {
    method: 'PUT',
    headers: {
      'Content-Type': file.mime_type,
      'x-amz-acl': 'private',
      'x-amz-meta-annotation_id': annotationId.toString(),
      'x-amz-meta-file_type': 'mask',
      'x-amz-meta-mask_group_id': groupId.toString(),
      'x-amz-meta-user_id': userId.toString(),
      'x-amz-storage-class': 'STANDARD'
    },
    body: fileData
  });
});

await Promise.all(uploadPromises);
```

### 3. 업로드 완료 확인

```javascript
const response = await fetch(`/api/annotations/${annotationId}/mask-groups/${groupId}/complete-upload`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    mask_group_id: groupId,
    slice_count: files.length,
    labels: ["liver"],
    uploaded_files: uploadUrls.map(info => info.file_path)
  })
});
```

## 🔍 마스크 조회 API

### 마스크 그룹 목록 조회

```http
GET /api/annotations/{annotation_id}/mask-groups
Authorization: Bearer {token}
```

**쿼리 파라미터:**
- `page`: 페이지 번호 (기본값: 1)
- `page_size`: 페이지당 항목 수 (기본값: 20)
- `modality`: 모달리티 필터 (예: CT, MR)
- `mask_type`: 마스크 타입 필터 (예: segmentation, detection)

### 마스크 그룹 상세 조회

```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}
Authorization: Bearer {token}
```

### 마스크 목록 조회

```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks
Authorization: Bearer {token}
```

**쿼리 파라미터:**
- `page`: 페이지 번호 (기본값: 1)
- `page_size`: 페이지당 항목 수 (기본값: 20)
- `label_name`: 라벨 이름 필터
- `slice_index`: 슬라이스 인덱스 필터
- `sop_instance_uid`: SOP Instance UID 필터

### 마스크 상세 조회

```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
Authorization: Bearer {token}
```

### 다운로드 URL 생성

```http
POST /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url
Authorization: Bearer {token}
Content-Type: application/json

{
  "mask_id": 4,
  "file_path": "masks/annotation_0/group_10/slice_001_liver.png",
  "expires_in": 3600
}
```

**응답:**
```json
{
  "download_url": "https://pacs-masks.s3.ap-northeast-2.amazonaws.com/masks/annotation_0/group_10/slice_001_liver.png?x-id=GetObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&...",
  "file_path": "masks/annotation_0/group_10/slice_001_liver.png",
  "expires_in": 3600,
  "expires_at": "2025-10-23 05:22:36.136641425 UTC"
}
```

## 🛠️ 마스크 관리 API

### 마스크 수정

```http
PUT /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "label_name": "liver_updated",
  "file_size": 2048000,
  "checksum": "sha256:new_checksum",
  "width": 1024,
  "height": 1024
}
```

### 마스크 삭제

```http
DELETE /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}
Authorization: Bearer {token}
```

## 📊 통계 API

### 마스크 통계 조회

```http
GET /api/annotations/{annotation_id}/mask-groups/{group_id}/masks/stats
Authorization: Bearer {token}
```

**응답:**
```json
{
  "total_masks": 120,
  "total_size": 122880000,
  "labels": {
    "liver": 60,
    "spleen": 30,
    "kidney": 30
  },
  "file_types": {
    "image/png": 120
  }
}
```

## 🚨 에러 처리

### 일반적인 HTTP 상태 코드

- `200 OK`: 요청 성공
- `201 Created`: 리소스 생성 성공
- `204 No Content`: 삭제 성공
- `400 Bad Request`: 잘못된 요청
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: 리소스 없음
- `500 Internal Server Error`: 서버 오류

### 에러 응답 형식

```json
{
  "error": "Bad Request",
  "message": "Json deserialize error: missing field 'mask_group_id'",
  "details": "필수 필드가 누락되었습니다."
}
```

## 💡 사용 팁

### 1. 업로드 최적화
- 여러 파일을 업로드할 때는 병렬 처리를 사용하세요
- 파일 크기가 큰 경우 청크 업로드를 고려하세요
- 업로드 URL은 10분 후 만료되므로 빠르게 사용하세요

### 2. 에러 처리
- 모든 API 호출에 대해 적절한 에러 처리를 구현하세요
- 네트워크 오류 시 재시도 로직을 구현하세요
- 업로드 실패 시 롤백 로직을 구현하세요

### 3. 보안
- JWT 토큰을 안전하게 저장하세요
- 업로드 URL을 로그에 남기지 마세요
- 민감한 정보는 클라이언트에 노출하지 마세요

## 📞 지원

API 사용 중 문제가 발생하면 다음을 확인하세요:

1. **서버 상태**: `GET /health`
2. **인증 토큰**: 만료 시간 확인
3. **요청 형식**: JSON 형식 및 필수 필드 확인
4. **권한**: 프로젝트 멤버십 확인

추가 지원이 필요한 경우 개발팀에 문의하세요.

---

**문서 버전**: 1.0.0  
**최종 업데이트**: 2025-10-23  
**API 버전**: v1
