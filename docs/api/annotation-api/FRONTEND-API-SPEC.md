# 📡 Annotation API - 프론트엔드 API 명세

## 🎯 개요

프론트엔드에서 사용할 Annotation API의 완전한 명세입니다.

---

## 1️⃣ Study/Series 레벨 Annotation 조회

### 요청

```http
GET /api/annotations?study_instance_uid={uid}&level=study,series&project_id={id}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `study_instance_uid` | string | ✅ | Study Instance UID |
| `level` | string | ✅ | `study,series` (쉼표로 구분) |
| `project_id` | number | ❌ | 프로젝트 ID (권한 필터링) |

### 응답 (200 OK)

```json
{
  "annotations": [
    {
      "id": 1,
      "project_id": 1,
      "user_id": 1,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "",
      "sop_instance_uid": "",
      "annotation_data": {
        "type": "rectangle",
        "coordinates": [100, 100, 200, 200],
        "label": "Tumor",
        "color": "#FF0000"
      },
      "version": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    },
    {
      "id": 2,
      "project_id": 1,
      "user_id": 1,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "",
      "annotation_data": {
        "type": "polygon",
        "points": [[100, 100], [200, 100], [200, 200], [100, 200]],
        "label": "Lesion",
        "color": "#00FF00"
      },
      "version": 2,
      "created_at": "2024-01-01T00:01:00Z",
      "updated_at": "2024-01-01T00:01:00Z"
    }
  ],
  "total": 2
}
```

### 응답 헤더

```
ETag: "1"
Last-Modified: Mon, 01 Jan 2024 00:00:00 +0000
Cache-Control: public, max-age=5
Content-Type: application/json
```

### 에러 응답

#### 404 Not Found
```json
{
  "error": "Not Found",
  "message": "Study not found"
}
```

#### 403 Forbidden
```json
{
  "error": "Forbidden",
  "message": "You don't have permission to access this study"
}
```

---

## 2️⃣ Instance 레벨 Annotation 조회

### 요청

```http
GET /api/annotations?series_instance_uid={uid}&level=instance&project_id={id}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `series_instance_uid` | string | ✅ | Series Instance UID |
| `level` | string | ✅ | `instance` |
| `project_id` | number | ❌ | 프로젝트 ID (권한 필터링) |

### 응답 (200 OK)

```json
{
  "annotations": [
    {
      "id": 3,
      "project_id": 1,
      "user_id": 1,
      "study_instance_uid": "1.2.3.4.5",
      "series_instance_uid": "1.2.3.4.5.6",
      "sop_instance_uid": "1.2.3.4.5.6.7",
      "annotation_data": {
        "type": "circle",
        "center": [150, 150],
        "radius": 50,
        "label": "Nodule",
        "color": "#0000FF"
      },
      "version": 1,
      "created_at": "2024-01-01T00:02:00Z",
      "updated_at": "2024-01-01T00:02:00Z"
    }
  ],
  "total": 1
}
```

### 응답 헤더

```
ETag: "1"
Last-Modified: Mon, 01 Jan 2024 00:02:00 +0000
Cache-Control: public, max-age=5
Content-Type: application/json
```

---

## 3️⃣ 캐시 검증 (HEAD 요청)

### 요청

```http
HEAD /api/annotations/{annotation_id}
If-None-Match: "{version}"
```

### 파라미터

| 파라미터 | 위치 | 타입 | 필수 | 설명 |
|---------|------|------|------|------|
| `annotation_id` | URL | number | ✅ | Annotation ID |
| `If-None-Match` | Header | string | ❌ | 캐시된 버전 (ETag) |

### 응답 (304 Not Modified - 캐시 유효)

```
HTTP/1.1 304 Not Modified
ETag: "1"
Last-Modified: Mon, 01 Jan 2024 00:00:00 +0000
Cache-Control: public, max-age=5
(본문 없음)
```

### 응답 (200 OK - 새로운 버전)

```
HTTP/1.1 200 OK
ETag: "2"
Last-Modified: Mon, 01 Jan 2024 00:01:00 +0000
Cache-Control: public, max-age=5
(본문 없음)
```

### 응답 (404 Not Found)

```
HTTP/1.1 404 Not Found
(본문 없음)
```

---

## 4️⃣ Annotation 수정

### 요청

```http
PUT /api/annotations/{annotation_id}
Content-Type: application/json

{
  "base_version": 1,
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 250, 250],
    "label": "Updated Tumor",
    "color": "#FF0000"
  }
}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `annotation_id` | number | ✅ | Annotation ID (URL) |
| `base_version` | number | ✅ | 클라이언트가 가진 버전 |
| `annotation_data` | object | ✅ | 수정할 Annotation 데이터 |

### 응답 (200 OK - 수정 성공)

```json
{
  "id": 1,
  "project_id": 1,
  "user_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "",
  "sop_instance_uid": "",
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 250, 250],
    "label": "Updated Tumor",
    "color": "#FF0000"
  },
  "version": 2,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:05:00Z"
}
```

### 응답 (409 Conflict - 버전 충돌)

```json
{
  "error": "Version Conflict",
  "message": "Version conflict: current version is 2, but client version is 1",
  "current_version": 2,
  "client_version": 1
}
```

**처리 방법:**
1. 최신 버전 조회: `GET /api/annotations/{annotation_id}`
2. 사용자에게 충돌 알림
3. 사용자가 선택:
   - 최신 버전 유지
   - 자신의 변경사항 다시 적용

### 응답 (404 Not Found)

```json
{
  "error": "Not Found",
  "message": "Annotation not found"
}
```

### 응답 (403 Forbidden)

```json
{
  "error": "Forbidden",
  "message": "You don't have permission to update this annotation"
}
```

---

## 5️⃣ Annotation 생성

### 요청

```http
POST /api/annotations
Content-Type: application/json

{
  "project_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 200, 200],
    "label": "New Annotation",
    "color": "#FF0000"
  }
}
```

### 응답 (201 Created)

```json
{
  "id": 4,
  "project_id": 1,
  "user_id": 1,
  "study_instance_uid": "1.2.3.4.5",
  "series_instance_uid": "1.2.3.4.5.6",
  "sop_instance_uid": "1.2.3.4.5.6.7",
  "annotation_data": {
    "type": "rectangle",
    "coordinates": [100, 100, 200, 200],
    "label": "New Annotation",
    "color": "#FF0000"
  },
  "version": 1,
  "created_at": "2024-01-01T00:10:00Z",
  "updated_at": "2024-01-01T00:10:00Z"
}
```

---

## 6️⃣ Annotation 삭제

### 요청

```http
DELETE /api/annotations/{annotation_id}
```

### 응답 (204 No Content)

```
HTTP/1.1 204 No Content
(본문 없음)
```

### 응답 (404 Not Found)

```json
{
  "error": "Not Found",
  "message": "Annotation not found"
}
```

---

## 📊 응답 필드 설명

### Annotation 객체

| 필드 | 타입 | 설명 |
|------|------|------|
| `id` | number | Annotation ID |
| `project_id` | number | 프로젝트 ID |
| `user_id` | number | 생성한 사용자 ID |
| `study_instance_uid` | string | Study Instance UID |
| `series_instance_uid` | string | Series Instance UID (선택사항) |
| `sop_instance_uid` | string | SOP Instance UID (선택사항) |
| `annotation_data` | object | Annotation 데이터 (자유 형식) |
| `version` | number | 현재 버전 (낙관적 잠금용) |
| `created_at` | string | 생성 시간 (ISO 8601) |
| `updated_at` | string | 수정 시간 (ISO 8601) |

---

## 🔄 HTTP 상태 코드

| 코드 | 설명 |
|------|------|
| `200 OK` | 요청 성공 |
| `201 Created` | 리소스 생성 성공 |
| `204 No Content` | 삭제 성공 |
| `304 Not Modified` | 캐시 유효 (HEAD 요청) |
| `400 Bad Request` | 잘못된 요청 |
| `403 Forbidden` | 권한 없음 |
| `404 Not Found` | 리소스 없음 |
| `409 Conflict` | 버전 충돌 |
| `500 Internal Server Error` | 서버 오류 |

---

## 🔐 인증

모든 요청에 인증 토큰 필요:

```http
Authorization: Bearer {token}
```

---

## 📝 예제 시나리오

### 시나리오 1: Study 선택 후 Annotation 표시

```bash
# 1. Study/Series 레벨 Annotation 조회
GET /api/annotations?study_instance_uid=1.2.3.4.5&level=study,series

# 응답: Study 레벨 + Series 레벨 Annotation 목록
# 캐시에 저장 (version: 1)
```

### 시나리오 2: Instance 선택 후 캐시 검증

```bash
# 1. 캐시 검증 (HEAD 요청)
HEAD /api/annotations/2
If-None-Match: "2"

# 응답: 304 Not Modified (캐시 유효)
# → 캐시된 데이터 사용

# 또는

# 응답: 200 OK (새로운 버전)
# → 전체 데이터 조회

# 2. Instance 레벨 Annotation 조회
GET /api/annotations?series_instance_uid=1.2.3.4.5.6&level=instance

# 응답: Instance 레벨 Annotation 목록
# 캐시 업데이트 (version: 1)
```

### 시나리오 3: Annotation 수정

```bash
# 1. Annotation 수정
PUT /api/annotations/1
{
  "base_version": 1,
  "annotation_data": {...}
}

# 응답 (성공): 200 OK + 새로운 version (2)
# 캐시 업데이트

# 또는

# 응답 (충돌): 409 Conflict
# → 사용자에게 알림
# → 최신 버전 조회
```

---

## 💡 팁

1. **캐시 활용**: 항상 캐시를 먼저 확인하세요
2. **HEAD 요청**: 메타데이터만 필요할 때 HEAD 요청 사용
3. **버전 관리**: 수정 시 항상 `base_version` 포함
4. **에러 처리**: 409 Conflict 시 최신 버전 조회
5. **성능**: 병렬 요청으로 성능 향상

