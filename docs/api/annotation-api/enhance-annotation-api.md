좋아 👍 완벽해.
아래는 **1차 개발 (REST 기반 Annotation 시스템)** 에 대한 기술 설계 문서 초안이야.
이걸 그대로 `annotation_api_spec_v1.md` 같은 이름으로 백엔드 팀에 전달하면 돼.
형식은 Markdown 문서로 되어 있어서 GitHub, Notion, Confluence 어디에 붙여도 읽기 좋아.

---

```markdown
# 🩻 Annotation REST API v1 — Technical Specification

**Version:** 1.0  
**Scope:** DICOM Viewer 1차 개발 (REST 기반 Annotation 관리)  
**Author:** Frontend Team  
**Date:** 2025-11-07

---

## 📘 목적

본 문서는 DICOM Viewer의 1차 개발 범위인 **Annotation REST API 설계 명세**를 정의한다.  
1차에서는 **단일 사용자 편집 / 캐시 / 버전 관리 기반의 안정적인 CRUD** 기능을 목표로 한다.  
WebSocket 실시간 동기화는 2차 확장 범위로 제외한다.

---

## 🧩 System Overview

### 아키텍처 개요
```

Viewer (Client)
│
├── REST API (HTTP)
│     ├─ CRUD (Create, Read, Update, Delete)
│     └─ Version Control
│
└── Annotation Service (Backend)
├─ DB (Annotation Table)
└─ DICOM Metadata (Study / Series / Instance)

````

---

## ⚙️ API Endpoints

### 1. Study / Series 레벨

#### `GET /studies/{studyUID}/series`
- **설명:** Study에 포함된 시리즈 목록 조회
- **Query Params:** `include=annotationSummary` (optional)
- **응답 예시:**
```json
[
  {
    "seriesUID": "1.2.840.113619.2.55.3.604688.908.1675744222.467",
    "seriesDescription": "T1 Axial",
    "annotationCount": 3,
    "latestVersion": 14
  }
]
````

---

### 2. Series 내 Annotation 목록

#### `GET /api/annotations?series_instance_uid={seriesUID}&project_id={projectID}`

* **설명:** 해당 시리즈의 annotation 목록 조회 (Query Parameter 기반)
* **Query Parameters:**
  - `series_instance_uid` (required): Series UID
  - `project_id` (optional): Project ID (권한 기반 필터링 시 필요)
  - `user_id` (optional): User ID (권한 기반 필터링 활성화)
  - `level` (optional): 필터링 레벨 (`study`, `series`, `instance`)
  - `viewer_software` (optional): Viewer 소프트웨어 필터링
* **응답 예시:**

```json
{
  "annotations": [
    {
      "id": 243,
      "project_id": 2,
      "user_id": 1,
      "study_instance_uid": "1.2.410.200022.500.200612201921171.113378644",
      "series_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
      "sop_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013813",
      "tool_name": "ROI",
      "tool_version": "1.0",
      "data": {...},
      "is_shared": false,
      "created_at": "2025-11-07T10:22:00Z",
      "updated_at": "2025-11-07T10:22:00Z",
      "viewer_software": "TI-DicomViewer",
      "description": "Lesion_01",
      "measurement_values": {...},
      "user_name": "John Doe"
    }
  ],
  "total": 1
}
```

#### `GET /api/annotations?study_instance_uid={studyUID}&project_id={projectID}`

* **설명:** 해당 Study의 annotation 목록 조회 (Query Parameter 기반)
* **Query Parameters:**
  - `study_instance_uid` (required): Study UID
  - `project_id` (optional): Project ID
  - `user_id` (optional): User ID (권한 기반 필터링 활성화)
  - `level` (optional): 필터링 레벨
  - `viewer_software` (optional): Viewer 소프트웨어 필터링
* **응답:** 위와 동일한 형식

---

### 3. Instance 단위 Annotation

#### `GET /api/annotations?sop_instance_uid={instanceUID}`

* **설명:** 특정 instance(프레임)의 annotation 전체 데이터 조회 (Query Parameter 기반)
* **Query Parameters:**
  - `sop_instance_uid` (required): SOP Instance UID
  - `level` (optional): 필터링 레벨 (`study`, `series`, `instance`)
  - `viewer_software` (optional): Viewer 소프트웨어 필터링
  - `user_id` (optional): User ID (필터링)
* **응답 예시:**

```json
{
  "annotations": [
    {
      "id": 243,
      "project_id": 2,
      "user_id": 1,
      "study_instance_uid": "1.2.410.200022.500.200612201921171.113378644",
      "series_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
      "sop_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013813",
      "tool_name": "ROI",
      "tool_version": "1.0",
      "data": {...},
      "is_shared": false,
      "created_at": "2025-11-07T10:22:00Z",
      "updated_at": "2025-11-07T10:22:00Z",
      "viewer_software": "TI-DicomViewer",
      "description": "Lesion_01",
      "measurement_values": {...},
      "user_name": "John Doe"
    }
  ],
  "total": 1
}
```

---

### 4. Annotation 생성 / 수정 / 삭제

#### `POST /api/annotations`

* **설명:** 새로운 annotation 생성
* **Request Body:**

```json
{
  "project_id": 2,
  "user_id": 1,
  "study_instance_uid": "1.2.410.200022.500.200612201921171.113378644",
  "series_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
  "sop_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013813",
  "tool_name": "ROI",
  "tool_version": "1.0",
  "data": {...},
  "is_shared": false,
  "viewer_software": "TI-DicomViewer",
  "description": "Lesion_01",
  "measurement_values": {...}
}
```

* **응답 예시 (201 Created):**

```json
{
  "id": 245,
  "project_id": 2,
  "user_id": 1,
  "study_instance_uid": "1.2.410.200022.500.200612201921171.113378644",
  "series_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
  "sop_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013813",
  "tool_name": "ROI",
  "tool_version": "1.0",
  "data": {...},
  "is_shared": false,
  "created_at": "2025-11-07T11:30:00Z",
  "updated_at": "2025-11-07T11:30:00Z",
  "viewer_software": "TI-DicomViewer",
  "description": "Lesion_01",
  "measurement_values": {...},
  "user_name": "John Doe"
}
```

---

#### `PUT /api/annotations/{annotation_id}`

* **설명:** annotation 수정
* **Request Body:**

```json
{
  "tool_name": "ROI",
  "tool_version": "1.0",
  "data": {...},
  "is_shared": false,
  "viewer_software": "TI-DicomViewer",
  "description": "Lesion_01_v2",
  "measurement_values": {...}
}
```

* **응답 (200 OK):**

```json
{
  "id": 245,
  "project_id": 2,
  "user_id": 1,
  "study_instance_uid": "1.2.410.200022.500.200612201921171.113378644",
  "series_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013771",
  "sop_instance_uid": "1.3.12.2.1107.5.1.4.51698.30000006122005083573400013813",
  "tool_name": "ROI",
  "tool_version": "1.0",
  "data": {...},
  "is_shared": false,
  "created_at": "2025-11-07T11:30:00Z",
  "updated_at": "2025-11-07T11:32:00Z",
  "viewer_software": "TI-DicomViewer",
  "description": "Lesion_01_v2",
  "measurement_values": {...},
  "user_name": "John Doe"
}
```

---

#### `DELETE /api/annotations/{annotation_id}`

* **설명:** annotation 삭제
* **응답:**

```
204 No Content
```

---

## 🧠 Version Control Logic

| 항목              | 설명                                      |
| --------------- | --------------------------------------- |
| **created_at**  | annotation 생성 시간 (서버에서 자동 설정)        |
| **updated_at**  | annotation 마지막 수정 시간 (서버에서 자동 업데이트) |
| **충돌 처리**       | 현재 구현에서는 Last-Write-Wins 방식 사용      |
| **캐시 관리**       | 클라이언트는 updated_at 기반으로 캐시 유효성 검증   |
| **정합성 유지**      | 모든 수정 요청은 최신 데이터로 덮어쓰기              |

> **주의:** 현재 구현은 버전 충돌 처리가 없습니다. 향후 2차 개발에서 optimistic locking 또는 conflict resolution 추가 예정

---

## 🗄️ 데이터 모델 (현재 구현)

| 필드                  | 타입       | 설명                           |
| ------------------- | -------- | ----------------------------- |
| id                  | integer  | 고유 ID (Primary Key)          |
| project_id          | integer  | 프로젝트 ID                      |
| user_id             | integer  | 생성자 User ID                  |
| study_instance_uid  | string   | Study UID                     |
| series_instance_uid | string   | Series UID                    |
| sop_instance_uid    | string   | SOP Instance UID (Instance)   |
| tool_name           | string   | 도구명 (ROI, Mask, Note 등)     |
| tool_version        | string   | 도구 버전                        |
| data                | JSON     | annotation 데이터 (geometry 등) |
| is_shared           | boolean  | 공유 여부                        |
| created_at          | datetime | 생성 시간                        |
| updated_at          | datetime | 마지막 수정 시간                   |
| viewer_software     | string   | Viewer 소프트웨어명               |
| description         | string   | 설명/라벨                        |
| measurement_values  | JSON     | 측정값 (선택사항)                  |
| user_name           | string   | 생성자 이름 (응답에만 포함)          |

---

## 🔒 에러 코드 표준화

| 코드                        | 의미            | 설명                |
| ------------------------- | ------------- | ----------------- |
| `400 BadRequest`          | 요청 데이터 오류     | geometry 포맷 불일치 등 |
| `404 NotFound`            | annotation 없음 | 잘못된 ID 요청         |
| `409 Conflict`            | 버전 충돌         | baseVersion 불일치   |
| `500 InternalServerError` | 서버 오류         | 예외 상황             |

---

## 📅 향후 확장 계획 (2차 이후)

| 범위                    | 설명                                            |
| --------------------- | --------------------------------------------- |
| WebSocket Event       | annotation_created / updated / deleted 실시간 전파 |
| Collaborative Lock    | 동일 annotation 동시 수정 방지                        |
| Presence              | 편집자 실시간 표시                                    |
| History / Audit Trail | annotation 변경 이력 저장                           |

---

## ✅ 요약

1️⃣ REST API는 모든 annotation CRUD의 **단일 진실 소스 (Source of Truth)**
2️⃣ 모든 수정/삭제는 **baseVersion 필수**로 version 정합성 유지
3️⃣ WebSocket은 차후 “실시간 알림” 용도로 REST 위에 추가

---

> **Frontend Note:**
> 클라이언트는 캐시된 annotation의 version을 관리하고,
> 각 instance 로드 시 `HEAD` 요청으로 최신 여부를 확인해야 한다.

```
