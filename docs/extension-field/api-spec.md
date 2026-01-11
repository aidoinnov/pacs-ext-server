# 📄 Study List View / Extension Field API 명세서

> **Version**: 1.0.0
> **Base URL**: `/api/v1`
> **인증**: Bearer Token (Keycloak)

---

## 목차

1. [View 관리 API](#1-view-관리-api)
2. [Field 정의 API](#2-field-정의-api)
3. [Study List 조회 API](#3-study-list-조회-api)
4. [공통 응답 형식](#4-공통-응답-형식)
5. [에러 코드](#5-에러-코드)

---

## 1. View 관리 API

### 1.1 View 목록 조회

사용 가능한 Study List View 목록을 조회한다.

```http
GET /api/v1/study-list-views
```

**Query Parameters**

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `scopeType` | string | ❌ | 범위 타입 필터 (`project`, `user`) |
| `scopeId` | string | ❌ | 범위 ID 필터 (project_id 등) |

**Response** `200 OK`

```json
{
  "items": [
    {
      "viewId": "default",
      "viewName": "Default",
      "isSystem": true,
      "ownerUserId": null,
      "scopeType": null,
      "scopeId": null,
      "createdAt": "2024-01-01T00:00:00Z"
    },
    {
      "viewId": "research-lung",
      "viewName": "Lung Cancer Research",
      "isSystem": false,
      "ownerUserId": null,
      "scopeType": "project",
      "scopeId": "LUNG_CANCER_01",
      "createdAt": "2024-06-15T10:30:00Z"
    }
  ],
  "total": 2
}
```

---

### 1.2 View 상세 조회

특정 View의 상세 정보 및 필드 구성을 조회한다.

```http
GET /api/v1/study-list-views/{viewId}
```

**Path Parameters**

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `viewId` | string | ✅ | View ID |

**Response** `200 OK`

```json
{
  "viewId": "research-lung",
  "viewName": "Lung Cancer Research",
  "isSystem": false,
  "ownerUserId": null,
  "scopeType": "project",
  "scopeId": "LUNG_CANCER_01",
  "createdAt": "2024-06-15T10:30:00Z",
  "fields": [
    {
      "source": "dicom",
      "key": "PatientName",
      "label": "Patient Name",
      "displayOrder": 0,
      "visible": true,
      "pinned": true,
      "width": 150
    },
    {
      "source": "dicom",
      "key": "StudyDate",
      "label": "Study Date",
      "displayOrder": 1,
      "visible": true,
      "pinned": false,
      "width": 100
    },
    {
      "source": "extension",
      "key": "subjectNo",
      "label": "Subject No",
      "displayOrder": 2,
      "visible": true,
      "pinned": false,
      "width": 120
    },
    {
      "source": "extension",
      "key": "timePoint",
      "label": "Time Point",
      "displayOrder": 3,
      "visible": true,
      "pinned": false,
      "width": 100
    }
  ]
}
```

**Response** `404 Not Found`

```json
{
  "error": "NOT_FOUND",
  "message": "View not found: invalid-view-id"
}
```

---

### 1.3 View 생성

새로운 View를 생성한다.

```http
POST /api/v1/study-list-views
```

**Request Body**

```json
{
  "viewId": "my-custom-view",
  "viewName": "My Custom View",
  "scopeType": "project",
  "scopeId": "LUNG_CANCER_01",
  "fields": [
    { "source": "dicom", "key": "PatientName", "displayOrder": 0, "visible": true, "pinned": true },
    { "source": "dicom", "key": "StudyDate", "displayOrder": 1, "visible": true },
    { "source": "extension", "key": "subjectNo", "displayOrder": 2, "visible": true }
  ]
}
```

**Response** `201 Created`

```json
{
  "viewId": "my-custom-view",
  "viewName": "My Custom View",
  "isSystem": false,
  "ownerUserId": "user-123",
  "scopeType": "project",
  "scopeId": "LUNG_CANCER_01",
  "createdAt": "2024-12-01T15:00:00Z"
}
```

---

### 1.4 View 수정

기존 View를 수정한다. (시스템 View는 수정 불가)

```http
PUT /api/v1/study-list-views/{viewId}
```

**Request Body**

```json
{
  "viewName": "Updated View Name",
  "fields": [
    { "source": "dicom", "key": "PatientName", "displayOrder": 0, "visible": true, "pinned": true },
    { "source": "dicom", "key": "StudyDate", "displayOrder": 1, "visible": true },
    { "source": "dicom", "key": "Modality", "displayOrder": 2, "visible": true },
    { "source": "extension", "key": "subjectNo", "displayOrder": 3, "visible": true }
  ]
}
```

**Response** `200 OK`

```json
{
  "viewId": "my-custom-view",
  "viewName": "Updated View Name",
  "updatedAt": "2024-12-01T16:00:00Z"
}
```

---

### 1.5 View 삭제

View를 삭제한다. (시스템 View는 삭제 불가)

```http
DELETE /api/v1/study-list-views/{viewId}
```

**Response** `204 No Content`

---

## 2. Field 정의 API

### 2.1 전체 필드 정의 조회

사용 가능한 모든 필드 정의를 조회한다.

```http
GET /api/v1/study-list-fields
```

**Query Parameters**

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `source` | string | ❌ | 필드 소스 필터 (`dicom`, `extension`) |
| `level` | string | ❌ | 레벨 필터 (`study`, `series`, `instance`) |
| `sortable` | boolean | ❌ | 정렬 가능 여부 필터 |
| `filterable` | boolean | ❌ | 필터 가능 여부 필터 |

**Response** `200 OK`

```json
{
  "items": [
    {
      "source": "dicom",
      "key": "PatientName",
      "tag": "00100010",
      "vr": "PN",
      "label": "Patient Name",
      "level": "study",
      "valueType": "string",
      "description": "환자 이름",
      "sortable": true,
      "filterable": true,
      "defaultVisible": true,
      "defaultOrder": 0
    },
    {
      "source": "dicom",
      "key": "StudyDate",
      "tag": "00080020",
      "vr": "DA",
      "label": "Study Date",
      "level": "study",
      "valueType": "date",
      "description": "검사 날짜",
      "sortable": true,
      "filterable": true,
      "defaultVisible": true,
      "defaultOrder": 1
    },
    {
      "source": "extension",
      "key": "subjectNo",
      "label": "Subject No",
      "level": "study",
      "valueType": "string",
      "description": "연구 대상자 번호",
      "sourceSystem": "internal",
      "sortable": true,
      "filterable": true,
      "defaultVisible": false,
      "defaultOrder": 10
    },
    {
      "source": "extension",
      "key": "annotationCount",
      "label": "Annotation Count",
      "level": "study",
      "valueType": "number",
      "description": "어노테이션 개수",
      "sourceSystem": "annotation",
      "sortable": true,
      "filterable": false,
      "defaultVisible": false,
      "defaultOrder": 20
    }
  ],
  "total": 4
}
```

---

### 3.1 Study List 조회 (GET)

View 기반으로 Study 목록을 조회한다.

```http
GET /api/v1/studies
```

**Query Parameters**

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `viewId` | string | ❌ | View ID (기본값: `default`) |
| `offset` | integer | ❌ | 페이지네이션 시작 위치 (기본값: 0) |
| `limit` | integer | ❌ | 페이지 크기 (기본값: 50, 최대: 200) |
| `sortField` | string | ❌ | 정렬 필드 (예: `StudyDate`, `subjectNo`) |
| `sortOrder` | string | ❌ | 정렬 순서 (`asc`, `desc`) |
| `PatientName` | string | ❌ | 환자 이름 필터 (DICOM wildcard 지원: `*`) |
| `StudyDate` | string | ❌ | 검사 날짜 필터 (범위: `20240101-20241231`) |
| `Modality` | string | ❌ | 모달리티 필터 |
| `{extensionField}` | string | ❌ | 확장 필드 필터 (예: `subjectNo=SUBJ-001`) |

**Response** `200 OK`

```json
{
  "items": [
    {
      "dicom": {
        "0020000D": { "vr": "UI", "Value": ["1.2.840.113619.2.388.10201606.1234567"] },
        "00100010": { "vr": "PN", "Value": [{ "Alphabetic": "Hong^Gildong" }] },
        "00080020": { "vr": "DA", "Value": ["20241201"] },
        "00081030": { "vr": "LO", "Value": ["Chest CT"] },
        "00080061": { "vr": "CS", "Value": ["CT"] }
      },
      "extensions": {
        "subjectNo": "SUBJ-0007",
        "timePoint": "Baseline",
        "visitType": "Screening",
        "annotationCount": 5,
        "reviewStatus": "Completed"
      }
    },
    {
      "dicom": {
        "0020000D": { "vr": "UI", "Value": ["1.2.840.113619.2.388.10201606.7654321"] },
        "00100010": { "vr": "PN", "Value": [{ "Alphabetic": "Kim^Cheolsu" }] },
        "00080020": { "vr": "DA", "Value": ["20241130"] },
        "00081030": { "vr": "LO", "Value": ["Brain MRI"] },
        "00080061": { "vr": "CS", "Value": ["MR"] }
      },
      "extensions": {
        "subjectNo": "SUBJ-0012",
        "timePoint": "Week 4",
        "visitType": "Follow-up",
        "annotationCount": null,
        "reviewStatus": "Pending"
      }
    }
  ],
  "pagination": {
    "offset": 0,
    "limit": 50,
    "total": 128,
    "hasMore": true
  },
  "meta": {
    "viewId": "research-lung",
    "requestedFields": {
      "dicom": ["PatientName", "StudyDate", "StudyDescription", "Modality"],
      "extension": ["subjectNo", "timePoint", "visitType", "annotationCount", "reviewStatus"]
    }
  }
}
```

---

### 3.2 Study List 검색 (POST)

복잡한 필터링이 필요한 경우 사용한다.

```http
POST /api/v1/studies/search
```

**Request Body**

```json
{
  "viewId": "research-lung",
  "filters": {
    "dicom": {
      "PatientName": "*Hong*",
      "StudyDate": {
        "from": "20240101",
        "to": "20241231"
      },
      "Modality": ["CT", "MR"]
    },
    "extension": {
      "subjectNo": "SUBJ-001",
      "reviewStatus": ["Pending", "InProgress"]
    }
  },
  "sort": [
    { "field": "StudyDate", "order": "desc" },
    { "field": "subjectNo", "order": "asc" }
  ],
  "pagination": {
    "offset": 0,
    "limit": 50
  }
}
```

**Response** `200 OK`

(3.1과 동일한 형식)

---

## 4. 공통 응답 형식

### 4.1 성공 응답

```json
{
  "items": [...],
  "total": 100,
  "pagination": {
    "offset": 0,
    "limit": 50,
    "hasMore": true
  }
}
```

### 4.2 에러 응답

```json
{
  "error": "ERROR_CODE",
  "message": "Human readable error message",
  "details": {
    "field": "additional info"
  }
}
```

---

## 5. 에러 코드

| HTTP Status | Error Code | 설명 |
|-------------|------------|------|
| 400 | `INVALID_REQUEST` | 잘못된 요청 파라미터 |
| 400 | `INVALID_VIEW_ID` | 잘못된 View ID 형식 |
| 400 | `INVALID_FIELD_KEY` | 존재하지 않는 필드 키 |
| 401 | `UNAUTHORIZED` | 인증 필요 |
| 403 | `FORBIDDEN` | 권한 없음 |
| 403 | `SYSTEM_VIEW_READONLY` | 시스템 View는 수정/삭제 불가 |
| 404 | `VIEW_NOT_FOUND` | View를 찾을 수 없음 |
| 409 | `VIEW_ALREADY_EXISTS` | 동일한 viewId가 이미 존재 |
| 500 | `INTERNAL_ERROR` | 서버 내부 오류 |
| 502 | `EXTENSION_SOURCE_ERROR` | Extension 소스 조회 실패 |
| 504 | `EXTENSION_TIMEOUT` | Extension 소스 타임아웃 |

---

## 6. 타입 정의 (TypeScript 참고용)

```typescript
// View 관련
interface StudyListView {
  viewId: string;
  viewName: string;
  isSystem: boolean;
  ownerUserId: string | null;
  scopeType: 'project' | 'user' | null;
  scopeId: string | null;
  createdAt: string;
  fields?: ViewField[];
}

interface ViewField {
  source: 'dicom' | 'extension';
  key: string;
  label: string;
  displayOrder: number;
  visible: boolean;
  pinned?: boolean;
  width?: number;
}

// Field 정의
interface DicomFieldDef {
  source: 'dicom';
  key: string;
  tag: string;
  vr: string;
  label: string;
  level: 'study' | 'series' | 'instance';
  valueType: 'string' | 'number' | 'date';
  description?: string;
  sortable: boolean;
  filterable: boolean;
  defaultVisible: boolean;
  defaultOrder: number;
}

interface ExtFieldDef {
  source: 'extension';
  key: string;
  label: string;
  level: 'study' | 'series' | 'instance';
  valueType: 'string' | 'number' | 'date' | 'enum';
  description?: string;
  sourceSystem: 'internal' | 'annotation' | 'workflow' | 'ctms' | 'ai';
  sortable: boolean;
  filterable: boolean;
  defaultVisible: boolean;
  defaultOrder: number;
}

// Study 응답
interface StudyItem {
  dicom: Record<string, DicomValue>;
  extensions: Record<string, any>;
}

interface DicomValue {
  vr: string;
  Value?: any[];
}

// 페이지네이션
interface Pagination {
  offset: number;
  limit: number;
  total: number;
  hasMore: boolean;
}
```
