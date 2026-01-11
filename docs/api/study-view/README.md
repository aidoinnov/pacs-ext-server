# 📋 Study List View API

> **Version**: 1.0.0  
> **Base URL**: `http://{server}:8080/api`  
> **인증**: Bearer Token (Keycloak)

## 개요

Study List View API는 PACS 뷰어의 Study 목록에 표시할 컬럼(필드) 구성을 관리합니다.

### 핵심 개념

| 개념 | 설명 |
|------|------|
| **View** | 컬럼 구성 프리셋. 여러 필드의 조합 |
| **Field** | 개별 컬럼. DICOM 필드 또는 확장 필드 |
| **System View** | 기본 제공 View (default). 삭제 불가, 필드만 수정 가능 |
| **User View** | 사용자 생성 View. 완전한 CRUD 가능 |

---

## 인증

모든 수정 API에 Bearer 토큰이 필요합니다:

```
Authorization: Bearer {access_token}
```

**토큰 획득:**
```bash
curl -X POST "http://localhost:8080/api/auth/keycloak-token" \
  -H "Content-Type: application/json" \
  -d '{"username":"your-username","password":"your-password"}'
```

---

## API 엔드포인트

### 1. View 목록 조회

```
GET /api/study-list-views
```

**응답:** 200 OK
```json
{
  "items": [
    {
      "viewId": "default",
      "viewName": "Default",
      "isSystem": true,
      "ownerUserId": null,
      "description": "기본 Study List View",
      "createdAt": "2026-01-08T05:45:02.375220Z"
    }
  ],
  "total": 1
}
```

---

### 2. View 상세 조회 (필드 포함)

```
GET /api/study-list-views/{viewId}
```

**응답:** 200 OK
```json
{
  "viewId": "default",
  "viewName": "Default",
  "isSystem": true,
  "fields": [
    {
      "source": "dicom",
      "key": "PatientName",
      "label": "Patient Name",
      "displayLabel": null,
      "displayOrder": 1,
      "visible": true,
      "pinned": false,
      "width": null
    },
    {
      "source": "dicom",
      "key": "StudyDate",
      "label": "검사일",
      "displayLabel": "검사일",
      "displayOrder": 2,
      "visible": true,
      "pinned": false,
      "width": 120
    }
  ]
}
```

---

### 3. View 생성

```
POST /api/study-list-views
Authorization: Bearer {token}
Content-Type: application/json
```

**요청:**
```json
{
  "viewId": "my-custom-view",
  "viewName": "My Custom View",
  "description": "프로젝트 전용 뷰",
  "fields": [
    {"source": "dicom", "key": "PatientName", "displayOrder": 1, "visible": true, "width": 200},
    {"source": "dicom", "key": "StudyDate", "displayOrder": 2, "visible": true, "displayLabel": "검사일"},
    {"source": "extension", "key": "status", "displayOrder": 3, "visible": true, "width": 100}
  ]
}
```

**응답:** 201 Created

---

### 4. View 수정 (필드 순서 변경)

```
PUT /api/study-list-views/{viewId}
Authorization: Bearer {token}
Content-Type: application/json
```

**요청:**
```json
{
  "viewName": "Updated Name",
  "description": "수정된 설명",
  "fields": [
    {"source": "dicom", "key": "StudyDate", "displayOrder": 1, "visible": true},
    {"source": "dicom", "key": "PatientName", "displayOrder": 2, "visible": true}
  ]
}
```

> ⚠️ **시스템 View**: viewName, description은 무시되고 fields만 수정됩니다.

**응답:** 200 OK

---

### 5. View 삭제

```
DELETE /api/study-list-views/{viewId}
Authorization: Bearer {token}
```

**응답:** 204 No Content

> ⚠️ 시스템 View는 삭제 불가 (403 Forbidden)

---

### 6. 필드 정의 조회

```
GET /api/study-list-views/field-defs
GET /api/study-list-views/field-defs?source=dicom
GET /api/study-list-views/field-defs?source=extension
```

**응답:** 200 OK
```json
{
  "items": [
    {"source": "dicom", "key": "PatientName", "label": "Patient Name", "tag": "00100010"},
    {"source": "extension", "key": "status", "label": "Status", "sourceSystem": "pacs-ext"}
  ],
  "total": 2
}
```

---

## 데이터 타입

### ViewFieldInput (요청 시)

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `source` | string | ✅ | `dicom` 또는 `extension` |
| `key` | string | ✅ | 필드 키 (예: `PatientName`, `status`) |
| `displayOrder` | integer | ✅ | 표시 순서 (1부터 시작) |
| `visible` | boolean | ❌ | 표시 여부 (기본: true) |
| `pinned` | boolean | ❌ | 고정 여부 (기본: false) |
| `width` | integer | ❌ | 컬럼 너비 (px) |
| `displayLabel` | string | ❌ | 사용자 정의 표시명 (설정 시 원본 label 대신 사용) |

### ViewFieldResponse (응답 시)

| 필드 | 타입 | 설명 |
|------|------|------|
| `source` | string | `dicom` 또는 `extension` |
| `key` | string | 필드 키 |
| `label` | string | 표시 라벨 (`displayLabel`이 있으면 그 값, 없으면 원본 label) |
| `displayLabel` | string/null | 사용자 정의 표시명 (null이면 원본 label 사용) |
| `displayOrder` | integer | 표시 순서 |
| `visible` | boolean | 표시 여부 |
| `pinned` | boolean | 고정 여부 |
| `width` | integer/null | 컬럼 너비 |

---

## 에러 응답

| 상태 코드 | 설명 | 예시 |
|-----------|------|------|
| `400` | 잘못된 요청 | 필수 필드 누락 |
| `401` | 인증 실패 | 토큰 없음/만료 |
| `403` | 권한 없음 | 시스템 View 삭제 시도 |
| `404` | 리소스 없음 | 존재하지 않는 viewId |
| `409` | 충돌 | 중복 viewId |
| `500` | 서버 오류 | - |

**에러 응답 형식:**
```json
{
  "error": "Forbidden",
  "message": "Cannot delete system view"
}
```

---

## 사용 예시 (cURL)

```bash
# 토큰 획득
TOKEN=$(curl -s -X POST "http://localhost:8080/api/auth/keycloak-token" \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"xxx"}' | jq -r '.access_token')

# View 목록 조회
curl "http://localhost:8080/api/study-list-views"

# View 상세 조회
curl "http://localhost:8080/api/study-list-views/default"

# View 생성
curl -X POST "http://localhost:8080/api/study-list-views" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"viewId":"my-view","viewName":"My View","fields":[{"source":"dicom","key":"PatientName","displayOrder":1,"visible":true}]}'

# 필드 순서 변경
curl -X PUT "http://localhost:8080/api/study-list-views/default" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"fields":[{"source":"dicom","key":"StudyDate","displayOrder":1,"visible":true},{"source":"dicom","key":"PatientName","displayOrder":2,"visible":true}]}'

# View 삭제
curl -X DELETE "http://localhost:8080/api/study-list-views/my-view" \
  -H "Authorization: Bearer $TOKEN"
```
