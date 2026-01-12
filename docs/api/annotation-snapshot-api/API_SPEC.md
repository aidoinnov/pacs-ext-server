
# 📄 Annotation Snapshot API 명세서

> **최종 업데이트**: 2026-01-12
> **상태**: 구현 완료 ✅
> **버전**: v1.0

---

## 📋 목차

1. [개요](#1-개요)
2. [인증](#2-인증)
3. [API 엔드포인트](#3-api-엔드포인트)
4. [데이터 모델](#4-데이터-모델)
5. [에러 코드](#5-에러-코드)
6. [사용 예시](#6-사용-예시)
7. [테스트](#7-테스트)

---

## 1. 개요

어노테이션 스냅샷 이미지를 S3에 업로드하고 관리하는 API입니다.

### 주요 기능
- ✅ Presigned URL 기반 직접 업로드
- ✅ 업로드 상태 관리 (pending → uploading → completed/failed)
- ✅ 권한 기반 접근 제어
- ✅ S3 이미지 다운로드 URL 생성

### 기술 스택
- **Backend**: Rust (Actix-web)
- **Database**: PostgreSQL
- **Storage**: AWS S3
- **인증**: JWT (Keycloak)

---

## 2. 인증

### 인증 방법

모든 API는 JWT 토큰 기반 인증이 필요합니다.

**헤더**:
```
Authorization: Bearer <JWT_TOKEN>
```

**개발 모드** (X-User-ID 헤더 사용):
```
X-User-ID: 1
```

### 공통 응답 포맷

**성공**:
```json
{
  "id": 123,
  "snapshot_image_key": "annotations/123/snapshots/...",
  "snapshot_status": "completed",
  "snapshot_uploaded_at": "2026-01-12T10:00:00Z"
}
```

**실패**:
```json
{
  "error": "Not Found",
  "message": "Annotation 123 not found"
}
```

---

## 3. API 엔드포인트

### 3.1 스냅샷 업로드 URL 생성

**POST** `/api/annotations/{annotation_id}/snapshot/upload-url`

어노테이션 스냅샷 이미지를 S3에 업로드하기 위한 Presigned URL을 생성합니다.

#### Request

**Headers**:
```
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
```

**Path Parameters**:
- `annotation_id` (integer, required): 어노테이션 ID

**Body**:
```json
{
  "filename": "snapshot_20260112_120000.png",
  "mime_type": "image/png",
  "file_size": 524288,
  "ttl_seconds": 600
}
```

**필드 설명**:
- `filename` (string, required): 파일명
- `mime_type` (string, required): MIME 타입 (image/png, image/jpeg, image/webp)
- `file_size` (integer, optional): 파일 크기 (바이트)
- `ttl_seconds` (integer, optional): URL 유효 시간 (초, 기본값: 600)

#### Response 200 OK

```json
{
  "upload_url": "https://pacs-masks.s3.ap-northeast-2.amazonaws.com/...",
  "download_url": "https://pacs-masks.s3.ap-northeast-2.amazonaws.com/...",
  "image_key": "annotations/123/snapshots/20260112_120000_snapshot.png",
  "expires_in": 600,
  "expires_at": "2026-01-12T12:10:00Z"
}
```

**필드 설명**:
- `upload_url` (string): S3 업로드용 Presigned URL
- `download_url` (string): S3 다운로드용 Presigned URL
- `image_key` (string): S3 object key (DB에 저장할 값)
- `expires_in` (integer): 만료 시간 (초)
- `expires_at` (string): 만료 시간 (ISO 8601)

#### Errors

- `401 Unauthorized`: 인증 실패 또는 권한 없음
- `404 Not Found`: 어노테이션을 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

---

### 3.2 스냅샷 업로드 완료 처리

**POST** `/api/annotations/{annotation_id}/snapshot/complete-upload`

S3 업로드 완료 후 서버에 알림하여 DB 상태를 업데이트합니다.

#### Request

**Headers**:
```
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
```

**Path Parameters**:
- `annotation_id` (integer, required): 어노테이션 ID

**Body**:
```json
{
  "image_key": "annotations/123/snapshots/20260112_120000_snapshot.png",
  "success": true
}
```

**필드 설명**:
- `image_key` (string, required): S3 object key (업로드 URL 생성 시 받은 값)
- `success` (boolean, optional): 업로드 성공 여부 (기본값: true)

#### Response 200 OK

```json
{
  "id": 123,
  "project_id": 1,
  "user_id": 1,
  "study_uid": "1.2.840.113619.2.55.3.604688119.868",
  "series_uid": "1.2.840.113619.2.55.3.604688119.868.1",
  "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1.1",
  "tool_name": "Circle Tool",
  "label": "Tumor",
  "snapshot_image_key": "annotations/123/snapshots/20260112_120000_snapshot.png",
  "snapshot_status": "Completed",
  "snapshot_uploaded_at": "2026-01-12T10:00:00Z",
  "created_at": "2026-01-12T09:00:00Z",
  "updated_at": "2026-01-12T10:00:00Z"
}
```

**중요**: `snapshot_uploaded_at`은 서버에서 자동으로 생성됩니다 (클라이언트가 보내지 않음).

#### Errors

- `401 Unauthorized`: 인증 실패 또는 권한 없음
- `404 Not Found`: 어노테이션을 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

---

### 3.3 스냅샷 상태 조회

**GET** `/api/annotations/{annotation_id}/snapshot/status`

어노테이션 스냅샷의 현재 상태를 조회합니다.

#### Request

**Headers**:
```
Authorization: Bearer <JWT_TOKEN>
```

**Path Parameters**:
- `annotation_id` (integer, required): 어노테이션 ID

#### Response 200 OK

```json
{
  "annotation_id": 123,
  "image_key": "annotations/123/snapshots/20260112_120000_snapshot.png",
  "status": "completed",
  "uploaded_at": "2026-01-12T10:00:00Z"
}
```

**필드 설명**:
- `annotation_id` (integer): 어노테이션 ID
- `image_key` (string|null): S3 object key
- `status` (string): 업로드 상태 (none, pending, uploading, completed, failed)
- `uploaded_at` (string|null): 업로드 완료 시간 (ISO 8601)

#### Errors

- `401 Unauthorized`: 인증 실패 또는 권한 없음
- `404 Not Found`: 어노테이션을 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

---

### 3.4 E2E 테스트 실행 (테스트용)

**GET** `/api/test/annotation-snapshot-e2e`

전체 워크플로우를 테스트하는 Python 스크립트를 실행합니다.

#### Request

**Headers**: 없음

#### Response 200 OK

```
🚀 Annotation Snapshot E2E Test 시작...
✅ 로그인 성공
✅ 어노테이션 생성 성공!
✅ 업로드 URL 생성 성공!
✅ 이미지 생성 완료!
✅ S3 업로드 성공!
✅ 업로드 완료 처리 성공!
✅ 상태 조회 성공!
✅ 어노테이션 조회 성공!

🎉 모든 테스트 통과!
```

**Content-Type**: `text/plain; charset=utf-8`

#### Errors

- `500 Internal Server Error`: 테스트 실행 실패
- `408 Request Timeout`: 테스트 실행 타임아웃 (120초 초과)

---

## 4. 데이터 모델

### Annotation (스냅샷 관련 필드)

```typescript
interface Annotation {
  id: number;
  snapshot_image_key: string | null;
  snapshot_status: SnapshotUploadStatus | null;
  snapshot_uploaded_at: string | null;  // ISO 8601
  // ... 기타 필드
}
```

### SnapshotUploadStatus (ENUM)

```typescript
enum SnapshotUploadStatus {
  Pending = "pending",      // URL 생성됨, 업로드 대기 중
  Uploading = "uploading",  // 업로드 진행 중
  Completed = "completed",  // 업로드 완료
  Failed = "failed"         // 업로드 실패
}
```

### 상태 전이 다이어그램

```
NULL → pending → uploading → completed
                     ↓
                  failed
```

---

## 5. 에러 코드

| HTTP | 에러 코드 | 설명 |
|------|---------|------|
| 400 | Bad Request | 잘못된 요청 파라미터 |
| 401 | Unauthorized | 인증 실패 또는 권한 없음 |
| 404 | Not Found | 리소스를 찾을 수 없음 |
| 408 | Request Timeout | 요청 타임아웃 (E2E 테스트) |
| 500 | Internal Server Error | 서버 내부 오류 |

---

## 6. 사용 예시

### 6.1 전체 워크플로우

```bash
# 1. 어노테이션 생성
curl -X POST "http://localhost:8080/api/annotations" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "study_uid": "1.2.840.113619.2.55.3.604688119.868",
    "series_uid": "1.2.840.113619.2.55.3.604688119.868.1",
    "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1.1",
    "tool_name": "Circle Tool",
    "label": "Tumor",
    "data": {"type": "circle", "radius": 10}
  }'

# 응답: {"id": 123, ...}

# 2. 스냅샷 업로드 URL 요청
curl -X POST "http://localhost:8080/api/annotations/123/snapshot/upload-url" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "filename": "snapshot_20260112_120000.png",
    "mime_type": "image/png",
    "ttl_seconds": 600
  }'

# 응답: {"upload_url": "https://...", "image_key": "annotations/123/..."}

# 3. S3에 이미지 업로드
curl -X PUT "{upload_url}" \
  -H "Content-Type: image/png" \
  --data-binary @snapshot.png

# 4. 업로드 완료 알림
curl -X POST "http://localhost:8080/api/annotations/123/snapshot/complete-upload" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "image_key": "annotations/123/snapshots/20260112_120000_snapshot.png",
    "success": true
  }'

# 응답: {"id": 123, "snapshot_status": "Completed", "snapshot_uploaded_at": "2026-01-12T10:00:00Z"}

# 5. 스냅샷 상태 조회
curl -X GET "http://localhost:8080/api/annotations/123/snapshot/status" \
  -H "X-User-ID: 1"

# 응답: {"annotation_id": 123, "status": "completed", "uploaded_at": "2026-01-12T10:00:00Z"}
```

### 6.2 Python 예시

```python
import requests

BASE_URL = "http://localhost:8080"
USER_ID = 1

# 1. 어노테이션 생성
response = requests.post(
    f"{BASE_URL}/api/annotations",
    json={
        "study_uid": "1.2.840.113619.2.55.3.604688119.868",
        "series_uid": "1.2.840.113619.2.55.3.604688119.868.1",
        "instance_uid": "1.2.840.113619.2.55.3.604688119.868.1.1",
        "tool_name": "Circle Tool",
        "label": "Tumor",
        "data": {"type": "circle", "radius": 10}
    },
    headers={"X-User-ID": str(USER_ID)}
)
annotation_id = response.json()["id"]

# 2. 스냅샷 업로드 URL 요청
response = requests.post(
    f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/upload-url",
    json={
        "filename": "snapshot.png",
        "mime_type": "image/png",
        "ttl_seconds": 600
    },
    headers={"X-User-ID": str(USER_ID)}
)
upload_data = response.json()

# 3. S3에 이미지 업로드
with open("snapshot.png", "rb") as f:
    requests.put(
        upload_data["upload_url"],
        data=f.read(),
        headers={"Content-Type": "image/png"}
    )

# 4. 업로드 완료 알림
response = requests.post(
    f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/complete-upload",
    json={
        "image_key": upload_data["image_key"],
        "success": True
    },
    headers={"X-User-ID": str(USER_ID)}
)

print(f"✅ 업로드 완료: {response.json()}")
```

---

## 7. 테스트

### 7.1 E2E 테스트 실행

**Python 스크립트**:
```bash
cd pacs-server
python3 e2e/test_annotation_snapshot_e2e.py
```

**웹 관리 페이지**:
1. http://localhost:3000 접속
2. 사이드바 → API 점검 클릭
3. Annotation Snapshot (📸) 클릭
4. "E2E 테스트 실행" 버튼 클릭

### 7.2 CRUD 테스트

웹 관리 페이지에서 제공하는 CRUD 인터페이스:
1. **어노테이션 생성**: 테스트용 어노테이션 자동 생성
2. **업로드 URL 요청**: S3 업로드 URL 생성
3. **업로드 완료 처리**: 업로드 완료 알림
4. **상태 조회**: 스냅샷 상태 조회

---

## 8. 주요 설계 결정

### 8.1 타임스탬프 생성 책임

**결정**: 서버가 `snapshot_uploaded_at`을 자동 생성

**이유**:
- 시간대 불일치 방지 (클라이언트 로컬 시간 vs 서버 UTC)
- 보안 (클라이언트가 시간 조작 불가)
- 데이터 일관성 (모든 타임스탬프는 서버 기준)

**구현**:
```rust
let now = chrono::Utc::now();  // 서버에서 자동 생성
```

### 8.2 S3 Key 규칙

**형식**: `annotations/{annotation_id}/snapshots/{timestamp}_{filename}`

**예시**: `annotations/123/snapshots/20260112_120000_snapshot.png`

### 8.3 상태 관리

**상태 전이**:
```
NULL → pending → uploading → completed
                     ↓
                  failed
```

---

## 9. 참고 문서

- [WORKLOG.md](./WORKLOG.md) - 구현 작업 로그
- [ARCHITECTURE.md](./ARCHITECTURE.md) - 아키텍처 설계
- [issues/ISSUE-001](./issues/ISSUE-001-timestamp-responsibility.md) - 타임스탬프 책임 소재
- [issues/ISSUE-002](./issues/ISSUE-002-no-update-annotation-entity.md) - 업데이트 패턴

---

**최종 업데이트**: 2026-01-12
**상태**: 구현 완료 ✅
