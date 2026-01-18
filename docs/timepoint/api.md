알겠어.
지금 네가 만든 명세를 **“팀/외주/미래의 우리를 설득할 수 있는 기술문서”** 형태로 정리해서 **한 번에 끝나는 문서**로 만들어줄게.
(👉 이후 ERD 문서의 상위 문서로 그대로 써도 되는 수준)

아래 문서는 **왜 이 구조인지(배경) → 무엇을 해결하는지 → 어떻게 동작하는지 → API 명세 요약** 순서로 구성했어.

---

# 📘 TimePoint 관리 기능 – 기술 설계 문서 (Fallback 단계)

## 1. 문서 목적

본 문서는 **웹 PACS 환경에서 RECIST Report 작성을 위해 필요한 TimePoint 관리 기능**의
**Fallback(CTIMS 미연동) 단계 구현을 위한 기술 설계 문서**이다.

본 기능은 다음을 목표로 한다.

* 자동 추론 없이, **사용자가 명시적으로 Study를 TimePoint에 배치**
* 기존 PACS **워크리스트 구조를 변경하지 않음**
* 향후 **CTIMS(TimePoint/Visit 정보) 연동 시 구조 변경 없이 전환 가능**

---

## 2. 배경 및 문제 정의

### 2.1 문제 배경

* RECIST Report는 **환자 단위 + TimePoint 단위**로 구성된다.
* 하나의 TimePoint는 **1개 이상의 Study**로 구성될 수 있다.
* 그러나 기존 PACS 환경에서는:

  * TimePoint 개념이 없음
  * Study 간 시점 관계가 정의되지 않음

또한 현재 단계에서는:

* CTIMS 연동 ❌
* Visit / TimePoint 자동 계산 ❌
* 사용자가 직접 시점을 지정해야 함 ⭕

---

### 2.2 설계 원칙

본 기능은 다음 원칙을 따른다.

1. **자동 추론을 하지 않는다**
2. **사용자가 직접 Study를 분류한다**
3. **실수 복구가 쉬운 UX를 제공한다**
4. **CTIMS 연동 시 read-only 전환이 가능해야 한다**

---

## 3. 핵심 개념 정의

### 3.1 TimePoint

* RECIST 평가에서의 **시점(순서)** 개념
* 예:

  * Baseline (order_index = 0)
  * TP1 (order_index = 1)
  * TP2 (order_index = 2)

> TimePoint는 **계산 기준**이며, Report는 항상 TimePoint 단위로 생성된다.

---

### 3.2 VisitType

* 임상시험 프로토콜 상의 방문 의미
* 예:

  * Baseline
  * Visit
  * EOT (End of Treatment)
  * USV (Unscheduled Visit)

> VisitType은 **임상적 의미**이며,
> fallback 단계에서는 보조 정보로만 사용된다.

---

### 3.3 Study ↔ TimePoint 관계

* 하나의 Study는 **하나의 TimePoint에만 속할 수 있음**
* TimePoint는 **여러 Study를 포함 가능**
* TimePoint에 속하지 않은 Study는 **Unassigned 상태**

---

## 4. UX 구조 요약 (보드형 모델)

본 기능은 “설정 화면”이 아니라
**Study를 TimePoint에 배치하는 작업 보드** 형태로 제공된다.

### 4.1 구조 개념

* 좌측: **Unassigned Studies**
* 우측: **TimePoint 그룹(Baseline, TP1, TP2 …)**
* 이동:

  * Unassigned → TimePoint (할당)
  * TimePoint → Unassigned (되돌리기)
  * TimePoint 간 이동 (재할당)

### 4.2 UX 특징

* 드래그 앤 드롭 없이 버튼 기반 이동
* 실수 시 언제든 되돌릴 수 있음
* TimePoint / VisitType은 **Study 단위가 아닌 그룹 단위**

---

## 5. API 설계 개요

### 5.1 Base 정보

* Base URL: `/api/timepoints`
* 인증: RBAC 기반 인증 필수
* Scope: **Project 단위**

---

## 6. API 목록 요약

### 6.1 TimePoint 목록 조회

```
GET /api/timepoints?project_id={projectId}
```

* 프로젝트에 속한 TimePoint 목록 반환
* `order_index` 기준 정렬

---

### 6.2 TimePoint 생성

```
POST /api/timepoints
```

* 새로운 TimePoint 생성
* 주요 규칙:

  * 프로젝트당 Baseline TimePoint는 1개만 허용
  * `order_index` 자동 증가
  * VisitType은 추후 변경 가능

---

### 6.3 TimePoint 수정

```
PUT /api/timepoints/{timepointId}
```

* 이름, VisitType, 순서 수정 가능
* Baseline 중복 방지

---

### 6.4 TimePoint 삭제

```
DELETE /api/timepoints/{timepointId}
```

* 삭제 시:

  * 해당 TimePoint에 속한 모든 Study는 `unassigned` 상태로 변경

---

### 6.5 Study → TimePoint 할당 (이동)

```
POST /api/timepoints/assign-study
```

* 여러 Study를 한 번에 특정 TimePoint로 이동
* 이미 다른 TimePoint에 속한 Study도 **자동 재할당**

> 본 API는 “할당”이 아닌 **이동(move)** 개념을 가진다.

---

### 6.6 Study → Unassigned 제거

```
POST /api/timepoints/remove-study
```

* TimePoint에서 Study 제거
* 제거된 Study는 `timepoint_id = null`

---

## 7. 데이터 모델 개요

### 7.1 TimePoint

```ts
interface TimePoint {
  id: string;
  project_id: string;
  name: string;                // Baseline, TP1, TP2
  visit_type: "Baseline" | "Visit" | "EOT" | "USV";
  visit_no?: number;           // CTIMS 연동 대비
  order_index: number;
}
```

---

### 7.2 Study-TimePoint 관계

```ts
interface StudyWithTimePoint {
  studyInstanceUID: string;
  project_id: string;
  timepoint_id: string | null; // null = unassigned
}
```

---

## 8. CTIMS 연동을 고려한 확장성

본 설계는 다음을 전제로 확장 가능하다.

* CTIMS에서:

  * TimePoint
  * VisitType
  * Visit No
    를 내려받는 경우

### 연동 이후 정책

* TimePoint / VisitType → read-only
* 내부 DB는 **CTIMS key를 저장**
* 기존 UI / API 구조 변경 없음

---

## 9. 설계 요약 (한 문장)

> **본 TimePoint 기능은
> 자동화 없는 fallback 단계에서
> 사용자가 Study를 명시적으로 시점에 배치하기 위한
> 작업 보드 기반 관리 기능이다.**

---

이 문서는 이제:

* ✅ 팀 내부 공유
* ✅ 외주 커뮤니케이션
* ✅ 다음 단계 **ERD 설계의 상위 문서**

로 바로 써도 된다.

---

👉 이제 다음 단계로 자연스럽게 갈 수 있어:
**“이 API를 기준으로 ERD를 그려달라”**

그거 던지면,
이번엔 **테이블 + FK + 제약조건까지 포함해서** 정리해줄게.

---

## 10. REST API 상세 스펙

### 10.1 Subject 관리 API

#### 10.1.1 Subject 목록 조회
```http
GET /api/projects/{project_id}/subjects
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "subjects": [
    {
      "id": 1,
      "project_id": 1,
      "subject_code": "A001",
      "patient_id": "P12345",
      "patient_name": "홍길동",
      "patient_birth_date": "1980-01-01",
      "created_at": "2026-01-18T10:00:00Z"
    }
  ]
}
```

#### 10.1.2 Subject 생성
```http
POST /api/projects/{project_id}/subjects
Authorization: Bearer {token}
Content-Type: application/json

{
  "subject_code": "A001",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1980-01-01"
}
```

**Response 201 Created:**
```json
{
  "id": 1,
  "project_id": 1,
  "subject_code": "A001",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1980-01-01",
  "created_at": "2026-01-18T10:00:00Z"
}
```

**Error 409 Conflict:**
```json
{
  "error": "SUBJECT_CODE_DUPLICATE",
  "message": "Subject code 'A001' already exists in this project"
}
```

#### 10.1.3 Subject 상세 조회
```http
GET /api/subjects/{subject_id}
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "id": 1,
  "project_id": 1,
  "subject_code": "A001",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1980-01-01",
  "timepoint_count": 3,
  "study_count": 5,
  "created_at": "2026-01-18T10:00:00Z"
}
```

---

### 10.2 TimePoint 관리 API

#### 10.2.1 Subject의 TimePoint 목록 조회
```http
GET /api/subjects/{subject_id}/timepoints
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
[
  {
    "id": 1,
    "subject_id": 1,
    "project_id": 1,
    "name": "Baseline",
    "visit_type": "Baseline",
    "visit_no": null,
    "order_index": 0,
    "external_key": null,
    "created_at": "2026-01-18T10:00:00Z",
    "updated_at": "2026-01-18T10:00:00Z"
  },
  {
    "id": 2,
    "subject_id": 1,
    "project_id": 1,
    "name": "TP1",
    "visit_type": "Visit",
    "visit_no": 1,
    "order_index": 1,
    "external_key": null,
    "created_at": "2026-01-18T11:00:00Z",
    "updated_at": "2026-01-18T11:00:00Z"
  }
]
```

#### 10.2.2 TimePoint 단일 조회
```http
GET /api/timepoints/{timepoint_id}
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "id": 1,
  "subject_id": 1,
  "project_id": 1,
  "name": "Baseline",
  "visit_type": "Baseline",
  "visit_no": null,
  "order_index": 0,
  "external_key": null,
  "created_at": "2026-01-18T10:00:00Z",
  "updated_at": "2026-01-18T10:00:00Z"
}
```

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

#### 10.2.3 TimePoint 생성
```http
POST /api/subjects/{subject_id}/timepoints
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "TP1",
  "visit_type": "Visit",
  "visit_no": 1,
  "order_index": 1
}
```

**Request Body:**
- `name` (required): TimePoint 이름 (예: "Baseline", "TP1", "TP2")
- `visit_type` (required): Visit 타입 ("Baseline" | "Visit" | "EOT" | "USV")
- `visit_no` (optional): CTIMS Visit Number
- `order_index` (required): 정렬 순서 (0부터 시작)
- `subject_id`: URL 경로에서 자동 설정됨

**Response 201 Created:**
```json
{
  "id": 2,
  "subject_id": 1,
  "project_id": 1,
  "name": "TP1",
  "visit_type": "Visit",
  "visit_no": 1,
  "order_index": 1,
  "external_key": null,
  "created_at": "2026-01-18T11:00:00Z",
  "updated_at": "2026-01-18T11:00:00Z"
}
```

**Error 400 Bad Request (유효성 검증 실패):**
```json
{
  "error": "Invalid visit_type: InvalidType"
}
```

**Error 404 Not Found (Subject 없음):**
```json
{
  "error": "Subject not found"
}
```

**Error 409 Conflict (Baseline 중복):**
```json
{
  "error": "Subject already has a Baseline timepoint"
}
```

**Error 409 Conflict (이름 중복):**
```json
{
  "error": "TimePoint name 'TP1' already exists for this subject"
}
```

#### 10.2.4 TimePoint 수정
```http
PUT /api/timepoints/{timepoint_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "TP1-Updated",
  "visit_type": "Visit",
  "order_index": 1
}
```

**Request Body (모든 필드 optional):**
- `name`: TimePoint 이름
- `visit_type`: Visit 타입
- `visit_no`: CTIMS Visit Number
- `order_index`: 정렬 순서

**Response 200 OK:**
```json
{
  "id": 2,
  "subject_id": 1,
  "project_id": 1,
  "name": "TP1-Updated",
  "visit_type": "Visit",
  "visit_no": 1,
  "order_index": 1,
  "external_key": null,
  "created_at": "2026-01-18T11:00:00Z",
  "updated_at": "2026-01-18T12:00:00Z"
}
```

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

**Error 409 Conflict (Baseline 중복):**
```json
{
  "error": "Subject already has a Baseline timepoint"
}
```

#### 10.2.5 TimePoint 삭제
```http
DELETE /api/timepoints/{timepoint_id}
Authorization: Bearer {token}
```

**Response 204 No Content**

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

**Note:**
- TimePoint 삭제 시 매핑된 모든 Study는 자동으로 할당 해제됨 (project_timepoint_study 레코드 삭제)
- Subject는 삭제되지 않음

---

### 10.3 Study 할당 관리 API

#### 10.3.1 Subject의 미할당 Study 목록 조회
```http
GET /api/subjects/{subject_id}/studies/unassigned
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
[
  {
    "study_instance_uid": "1.2.840.113619.2.55.3.123456789.1",
    "study_date": "20260115",
    "study_time": "143000",
    "study_description": "CT CHEST",
    "modality": "CT",
    "patient_id": "P12345",
    "patient_name": "홍길동"
  },
  {
    "study_instance_uid": "1.2.840.113619.2.55.3.123456789.2",
    "study_date": "20260116",
    "study_time": "100000",
    "study_description": "CT ABDOMEN",
    "modality": "CT",
    "patient_id": "P12345",
    "patient_name": "홍길동"
  }
]
```

**Error 404 Not Found:**
```json
{
  "error": "Subject not found"
}
```

#### 10.3.2 TimePoint에 할당된 Study 목록 조회
```http
GET /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
[
  {
    "study_instance_uid": "1.2.840.113619.2.55.3.123456789.3",
    "study_date": "20260117",
    "study_time": "090000",
    "study_description": "CT CHEST",
    "modality": "CT",
    "patient_id": "P12345",
    "patient_name": "홍길동"
  }
]
```

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

#### 10.3.3 Study를 TimePoint에 할당
```http
POST /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
Content-Type: application/json

{
  "study_instance_uids": [
    "1.2.840.113619.2.55.3.123456789.1",
    "1.2.840.113619.2.55.3.123456789.2"
  ]
}
```

**Request Body:**
- `study_instance_uids` (required): 할당할 Study UID 배열

**Response 200 OK:**
```json
{
  "assigned": [
    "1.2.840.113619.2.55.3.123456789.1",
    "1.2.840.113619.2.55.3.123456789.2"
  ],
  "reassigned": [],
  "failed": []
}
```

**Response 200 OK (일부 재할당):**
```json
{
  "assigned": [
    "1.2.840.113619.2.55.3.123456789.1"
  ],
  "reassigned": [
    "1.2.840.113619.2.55.3.123456789.2"
  ],
  "failed": []
}
```

**설명:**
- `assigned`: 새로 할당된 Study 목록
- `reassigned`: 다른 TimePoint에서 이동된 Study 목록
- `failed`: 할당 실패한 Study 목록 (존재하지 않거나 권한 없음)

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

**Error 400 Bad Request:**
```json
{
  "error": "study_instance_uids is required and must be a non-empty array"
}
```

#### 10.3.4 Study 할당 해제
```http
DELETE /api/timepoints/{timepoint_id}/studies
Authorization: Bearer {token}
Content-Type: application/json

{
  "study_instance_uids": [
    "1.2.840.113619.2.55.3.123456789.1"
  ]
}
```

**Request Body:**
- `study_instance_uids` (required): 할당 해제할 Study UID 배열

**Response 200 OK:**
```json
{
  "unassigned": [
    "1.2.840.113619.2.55.3.123456789.1"
  ],
  "not_found": []
}
```

**설명:**
- `unassigned`: 할당 해제된 Study 목록
- `not_found`: 해당 TimePoint에 할당되지 않았던 Study 목록

**Error 404 Not Found:**
```json
{
  "error": "TimePoint not found"
}
```

**Error 400 Bad Request:**
```json
{
  "error": "study_instance_uids is required and must be a non-empty array"
}
```

---

### 10.4 Subject 삭제 보호

Subject를 삭제하려면 먼저 모든 TimePoint를 삭제해야 합니다.

```http
DELETE /api/subjects/{subject_id}
Authorization: Bearer {token}
```

**Error 409 Conflict (TimePoint 존재):**
```json
{
  "error": "Cannot delete subject with existing timepoints. Delete all timepoints first."
}
```

**Response 204 No Content** (TimePoint가 없는 경우)

---

## 11. 구현 상태

### 11.1 완료된 기능 ✅

- [x] Subject CRUD 작업
  - [x] Subject 생성 (프로젝트별)
  - [x] Subject 조회 (단일/목록)
  - [x] Subject 상세 조회 (TimePoint/Study 카운트 포함)
  - [x] Subject 수정
  - [x] Subject 삭제 (TimePoint 보호)
  - [x] Subject Code 중복 방지 (프로젝트 내)
  - [x] Patient ID 중복 방지 (프로젝트 내)

- [x] TimePoint CRUD 작업
  - [x] TimePoint 생성 (Subject별)
  - [x] TimePoint 조회 (단일/목록)
  - [x] TimePoint 수정
  - [x] TimePoint 삭제 (Study 자동 할당 해제)
  - [x] Baseline 중복 방지 (Subject당 1개)
  - [x] TimePoint 이름 중복 방지 (Subject 내)

- [x] Study 할당 관리
  - [x] 미할당 Study 목록 조회
  - [x] TimePoint별 Study 목록 조회
  - [x] Study 할당 (단일/다중)
  - [x] Study 재할당 (TimePoint 간 이동)
  - [x] Study 할당 해제

- [x] E2E 테스트
  - [x] Subject 관리 테스트 (6개)
  - [x] TimePoint 관리 테스트 (5개)
  - [x] Study 할당 테스트 (5개)
  - [x] Cascade 보호 테스트 (3개)
  - [x] 에러 케이스 테스트 (3개)
  - [x] **총 18개 테스트 통과**

### 11.2 향후 확장 계획 🔮

- [ ] CTIMS 연동
  - [ ] TimePoint/Visit 정보 동기화
  - [ ] External Key 매핑
  - [ ] Read-only 모드 전환

- [ ] UI 개선
  - [ ] 드래그 앤 드롭 지원
  - [ ] 대량 할당 UI
  - [ ] TimePoint 순서 변경 UI

- [ ] 성능 최적화
  - [ ] Study 목록 페이지네이션
  - [ ] 캐싱 전략
  - [ ] 인덱스 최적화

---

## 12. 기술 스택

- **Backend**: Rust + Actix-web
- **Database**: PostgreSQL
- **ORM**: SQLx
- **API Documentation**: OpenAPI 3.0 (utoipa)
- **Testing**: pytest (E2E)

---

## 13. 참고 문서

- [ERD 설계 문서](./erd.md)
- [데이터베이스 스키마](../../pacs-server/migrations/023_refactor_project_data_hierarchy.sql)
- [E2E 테스트](../../tests/e2e/test_05_subject_timepoint.py)
