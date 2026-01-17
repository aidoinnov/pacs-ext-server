# Annotation Snapshot API 구현 안내

## 📋 개요

이 문서는 Annotation Snapshot 기능의 API 구현 상태와 사용 방법을 설명합니다.

---

## 🎯 구현된 기능

### 1. Snapshot 업로드 API

**엔드포인트**: `POST /api/v1/annotations/{annotation_id}/snapshot`

**설명**: Annotation의 스냅샷 이미지를 업로드합니다.

**요청**:
```http
POST /api/v1/annotations/123/snapshot HTTP/1.1
Content-Type: multipart/form-data
Authorization: Bearer {token}

--boundary
Content-Disposition: form-data; name="file"; filename="snapshot.png"
Content-Type: image/png

[이미지 바이너리 데이터]
--boundary--
```

**응답**:
```json
{
  "id": 123,
  "project_id": 1,
  "user_id": 42,
  "study_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
  "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
  "tool_name": "ruler",
  "tool_version": "1.0.0",
  "data": {
    "points": [[100, 200], [300, 400]],
    "length": 223.6
  },
  "is_shared": false,
  "snapshot_image_key": "snapshots/annotations/123/snapshot_1234567890.png",
  "snapshot_status": "uploaded",
  "snapshot_uploaded_at": "2026-01-14T12:34:56.789Z",
  "created_at": "2026-01-14T10:00:00.000Z",
  "updated_at": "2026-01-14T12:34:56.789Z",
  "version": 2,
  "viewer_software": "OHIF Viewer",
  "description": "Measurement of lesion",
  "measurement_values": {
    "length": 223.6,
    "unit": "mm"
  },
  "label": "Lesion A"
}
```

**상태 코드**:
- `200 OK`: 업로드 성공
- `400 Bad Request`: 잘못된 요청 (파일 없음, 잘못된 형식 등)
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: Annotation 없음
- `413 Payload Too Large`: 파일 크기 초과
- `500 Internal Server Error`: 서버 오류

---

### 2. Snapshot 조회 API

**엔드포인트**: `GET /api/v1/annotations/{annotation_id}/snapshot`

**설명**: Annotation의 스냅샷 이미지를 조회합니다.

**요청**:
```http
GET /api/v1/annotations/123/snapshot HTTP/1.1
Authorization: Bearer {token}
```

**응답**:
```http
HTTP/1.1 200 OK
Content-Type: image/png
Content-Length: 123456
Cache-Control: public, max-age=3600

[이미지 바이너리 데이터]
```

**상태 코드**:
- `200 OK`: 조회 성공
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: Annotation 또는 Snapshot 없음
- `500 Internal Server Error`: 서버 오류

---

### 3. Snapshot 삭제 API

**엔드포인트**: `DELETE /api/v1/annotations/{annotation_id}/snapshot`

**설명**: Annotation의 스냅샷 이미지를 삭제합니다.

**요청**:
```http
DELETE /api/v1/annotations/123/snapshot HTTP/1.1
Authorization: Bearer {token}
```

**응답**:
```http
HTTP/1.1 204 No Content
```

**상태 코드**:
- `204 No Content`: 삭제 성공
- `401 Unauthorized`: 인증 실패
- `403 Forbidden`: 권한 없음
- `404 Not Found`: Annotation 또는 Snapshot 없음
- `500 Internal Server Error`: 서버 오류

---

## 📊 Snapshot 상태 (SnapshotUploadStatus)

Snapshot은 다음 3가지 상태를 가질 수 있습니다:

```rust
pub enum SnapshotUploadStatus {
    Pending,    // 업로드 대기 중
    Uploaded,   // 업로드 완료
    Failed,     // 업로드 실패
}
```

### 상태 전이

```
[생성] → Pending
         ↓
    [업로드 성공] → Uploaded
         ↓
    [업로드 실패] → Failed
         ↓
    [재시도] → Pending
```

---

## 🔧 구현 세부사항

### 1. DB 스키마

```sql
-- annotation_annotation 테이블에 추가된 컬럼
ALTER TABLE annotation_annotation
ADD COLUMN snapshot_image_key VARCHAR(512),      -- S3 키
ADD COLUMN snapshot_status VARCHAR(50),          -- 상태 (pending, uploaded, failed)
ADD COLUMN snapshot_uploaded_at TIMESTAMP WITH TIME ZONE;  -- 업로드 시각
```

### 2. Entity 구조체

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Annotation {
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    // ... 기존 필드들
    
    // Snapshot 관련 필드
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
    pub snapshot_uploaded_at: Option<DateTime<Utc>>,
    
    // ... 나머지 필드들
}
```

### 3. Repository 메서드

```rust
impl AnnotationRepository for AnnotationRepositoryImpl {
    // Snapshot 정보 업데이트
    async fn update_snapshot_info(
        &self,
        annotation_id: i32,
        snapshot_image_key: Option<String>,
        snapshot_status: Option<SnapshotUploadStatus>,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Option<Annotation>, sqlx::Error>;
}
```

---

## 📝 사용 예시

### Python (requests)

```python
import requests

# 1. Snapshot 업로드
with open('snapshot.png', 'rb') as f:
    response = requests.post(
        'http://localhost:8080/api/v1/annotations/123/snapshot',
        headers={'Authorization': f'Bearer {token}'},
        files={'file': ('snapshot.png', f, 'image/png')}
    )
    print(response.json())

# 2. Snapshot 조회
response = requests.get(
    'http://localhost:8080/api/v1/annotations/123/snapshot',
    headers={'Authorization': f'Bearer {token}'}
)
with open('downloaded_snapshot.png', 'wb') as f:
    f.write(response.content)

# 3. Snapshot 삭제
response = requests.delete(
    'http://localhost:8080/api/v1/annotations/123/snapshot',
    headers={'Authorization': f'Bearer {token}'}
)
print(response.status_code)  # 204
```

### JavaScript (fetch)

```javascript
// 1. Snapshot 업로드
const formData = new FormData();
formData.append('file', fileInput.files[0]);

const uploadResponse = await fetch(
  `http://localhost:8080/api/v1/annotations/123/snapshot`,
  {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`
    },
    body: formData
  }
);
const annotation = await uploadResponse.json();
console.log(annotation);

// 2. Snapshot 조회
const getResponse = await fetch(
  `http://localhost:8080/api/v1/annotations/123/snapshot`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);
const blob = await getResponse.blob();
const imageUrl = URL.createObjectURL(blob);
document.getElementById('snapshot').src = imageUrl;

// 3. Snapshot 삭제
const deleteResponse = await fetch(
  `http://localhost:8080/api/v1/annotations/123/snapshot`,
  {
    method: 'DELETE',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);
console.log(deleteResponse.status);  // 204
```

---

## 🔒 권한 관리

Snapshot API는 다음 권한 규칙을 따릅니다:

1. **업로드**: Annotation 소유자만 가능
2. **조회**: Annotation 조회 권한이 있는 사용자
3. **삭제**: Annotation 소유자만 가능

---

## 📏 제약사항

### 파일 크기
- **최대 크기**: 10MB
- **권장 크기**: 1MB 이하

### 파일 형식
- **지원 형식**: PNG, JPEG, WebP
- **권장 형식**: PNG (무손실 압축)

### 이미지 해상도
- **최대 해상도**: 4096 x 4096
- **권장 해상도**: 1920 x 1080

---

## 🚨 에러 처리

### 일반적인 에러

```json
{
  "error": "BadRequest",
  "message": "File size exceeds maximum allowed size of 10MB",
  "details": {
    "file_size": 15728640,
    "max_size": 10485760
  }
}
```

### 에러 코드

| 코드 | 설명 | 해결 방법 |
|------|------|-----------|
| `FILE_TOO_LARGE` | 파일 크기 초과 | 파일 크기를 10MB 이하로 줄이세요 |
| `INVALID_FILE_TYPE` | 지원하지 않는 파일 형식 | PNG, JPEG, WebP 형식을 사용하세요 |
| `ANNOTATION_NOT_FOUND` | Annotation 없음 | Annotation ID를 확인하세요 |
| `PERMISSION_DENIED` | 권한 없음 | Annotation 소유자인지 확인하세요 |
| `SNAPSHOT_NOT_FOUND` | Snapshot 없음 | Snapshot이 업로드되었는지 확인하세요 |

---

## 🧪 테스트

### E2E 테스트

```bash
cd pacs-server/e2e
python test_annotation_snapshot_e2e_refactored.py
```

### 테스트 시나리오

1. ✅ Snapshot 업로드
2. ✅ Snapshot 조회
3. ✅ Snapshot 삭제
4. ✅ 권한 검증
5. ✅ 파일 크기 제한
6. ✅ 파일 형식 검증

---

## 📖 관련 문서

- [README.md](./README.md) - 이슈 개요
- [체크리스트.md](./체크리스트.md) - Entity 변경 시 체크리스트
- [기술-분석.md](./기술-분석.md) - 기술적 분석

