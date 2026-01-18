# 📘 Subject & TimePoint 관리 API - 구현 문서

> **작성일**: 2026-01-18
> **버전**: 2.0.0
> **상태**: ✅ 구현 완료 + Subject 자동 생성 기능 추가

---

## 목차

1. [개요](#1-개요)
2. [구현 상태](#2-구현-상태)
3. [API 엔드포인트](#3-api-엔드포인트)
4. [데이터 모델](#4-데이터-모델)
5. [에러 처리](#5-에러-처리)
6. [테스트](#6-테스트)
7. [향후 계획](#7-향후-계획)

---

## 1. 개요

### 1.1 목적

본 문서는 **웹 PACS 환경에서 RECIST Report 작성을 위한 Subject & TimePoint 관리 기능**의 실제 구현 내용을 기술합니다.

### 1.2 핵심 기능

- **Subject 관리**: 임상시험 피험자 정보 관리
- **Subject 자동 생성**: Study 할당 시 Subject 자동 생성 (v2.0 신규)
- **TimePoint 관리**: 평가 시점(Baseline, TP1, TP2 등) 관리
- **Study 할당**: DICOM Study를 프로젝트 및 TimePoint에 매핑

### 1.3 설계 원칙

1. **명시적 할당**: 자동 추론 없이 사용자가 직접 Study를 TimePoint에 할당
2. **PACS 비침투**: 기존 PACS 워크리스트 구조 변경 없음
3. **확장 가능**: CTIMS 연동 시 구조 변경 없이 전환 가능
4. **실수 복구**: 언제든 할당 해제 및 재할당 가능

---

## 2. 구현 상태

### 2.1 완료된 기능 ✅

#### Subject 관리
- [x] Subject 생성 (프로젝트별)
- [x] Subject 조회 (단일/목록/상세)
- [x] Subject 수정
- [x] Subject 삭제 (TimePoint 보호)
- [x] Subject Code 중복 방지 (프로젝트 내)
- [x] Patient ID 중복 방지 (프로젝트 내)
- [x] **Subject 자동 생성** (Study 할당 시, v2.0 신규)
- [x] **Subject Code 자동 생성** (A-001, A-002, ..., v2.0 신규)

#### TimePoint 관리
- [x] TimePoint 생성 (Subject별)
- [x] TimePoint 조회 (단일/목록)
- [x] TimePoint 수정
- [x] TimePoint 삭제 (Study 자동 할당 해제)
- [x] Baseline 중복 방지 (Subject당 1개)
- [x] TimePoint 이름 중복 방지 (Subject 내)

#### Study 할당
- [x] 미할당 Study 목록 조회
- [x] TimePoint별 Study 목록 조회
- [x] Study 할당 (단일/다중)
- [x] Study 재할당 (TimePoint 간 이동)
- [x] Study 할당 해제

### 2.2 테스트 현황

- **총 테스트**: 22개
- **통과**: 18개 ✅
- **스킵**: 4개 (의도적)
- **실패**: 0개

**테스트 분류:**
- Subject 관리: 6개
- TimePoint 관리: 5개
- Study 할당: 5개
- Cascade 보호: 3개
- 에러 케이스: 3개

---

## 3. API 엔드포인트

### 3.1 Subject API

| Method | Endpoint | 설명 |
|--------|----------|------|
| `POST` | `/api/projects/{project_id}/subjects` | Subject 생성 |
| `GET` | `/api/projects/{project_id}/subjects` | Subject 목록 조회 |
| `GET` | `/api/subjects/{id}` | Subject 조회 |
| `GET` | `/api/subjects/{id}/detail` | Subject 상세 조회 |
| `PUT` | `/api/subjects/{id}` | Subject 수정 |
| `DELETE` | `/api/subjects/{id}` | Subject 삭제 |

### 3.2 TimePoint API

| Method | Endpoint | 설명 |
|--------|----------|------|
| `POST` | `/api/subjects/{subject_id}/timepoints` | TimePoint 생성 |
| `GET` | `/api/subjects/{subject_id}/timepoints` | Subject의 TimePoint 목록 |
| `GET` | `/api/timepoints/{id}` | TimePoint 조회 |
| `PUT` | `/api/timepoints/{id}` | TimePoint 수정 |
| `DELETE` | `/api/timepoints/{id}` | TimePoint 삭제 |

### 3.3 Study 할당 API

| Method | Endpoint | 설명 |
|--------|----------|------|
| `GET` | `/api/subjects/{subject_id}/studies/unassigned` | 미할당 Study 목록 |
| `GET` | `/api/timepoints/{id}/studies` | TimePoint의 Study 목록 |
| `POST` | `/api/timepoints/{id}/studies` | Study 할당 |
| `DELETE` | `/api/timepoints/{id}/studies` | Study 할당 해제 |

---

## 4. 데이터 모델

### 4.1 Subject

```typescript
interface Subject {
  id: number;
  project_id: number;
  subject_code: string;           // 프로젝트 내 고유
  patient_id: string;             // 프로젝트 내 고유
  patient_name: string;
  patient_birth_date: string;     // YYYY-MM-DD
  external_subject_key?: string;  // CTIMS 연동용
  created_at: string;             // ISO 8601
  updated_at: string;
}
```

### 4.2 TimePoint

```typescript
interface TimePoint {
  id: number;
  project_id: number;
  subject_id: number;
  name: string;                   // Subject 내 고유
  visit_type: VisitType;
  visit_no?: number;              // CTIMS Visit Number
  order_index: number;            // 정렬 순서
  external_key?: string;          // CTIMS 연동용
  created_at: string;
  updated_at: string;
}

type VisitType = "Baseline" | "Visit" | "EOT" | "USV";
```

### 4.3 Study 정보

```typescript
interface StudyInfo {
  study_instance_uid: string;
  study_date: string;             // YYYYMMDD
  study_time: string;             // HHMMSS
  study_description: string;
  modality: string;
  patient_id: string;
  patient_name: string;
}
```

---

## 5. API 상세 명세

### 5.1 Subject 생성

```http
POST /api/projects/{project_id}/subjects
Authorization: Bearer {token}
Content-Type: application/json

{
  "subject_code": "A001",
  "patient_id": "P12345",
  "patient_name": "홍길동",
  "patient_birth_date": "1990-01-01"
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
  "patient_birth_date": "1990-01-01",
  "external_subject_key": null,
  "created_at": "2026-01-18T10:00:00Z",
  "updated_at": "2026-01-18T10:00:00Z"
}
```

**Error 404 Not Found:**
```json
{
  "error": "Project not found"
}
```

**Error 409 Conflict:**
```json
{
  "error": "Subject code 'A001' already exists in this project"
}
```

---

### 5.2 Subject 상세 조회

```http
GET /api/subjects/{id}/detail
Authorization: Bearer {token}
```

**Response 200 OK:**
```json
{
  "subject": {
    "id": 1,
    "project_id": 1,
    "subject_code": "A001",
    "patient_id": "P12345",
    "patient_name": "홍길동",
    "patient_birth_date": "1990-01-01",
    "external_subject_key": null,
    "created_at": "2026-01-18T10:00:00Z",
    "updated_at": "2026-01-18T10:00:00Z"
  },
  "timepoint_count": 3,
  "study_count": 5
}
```

---

### 5.3 TimePoint 생성

```http
POST /api/subjects/{subject_id}/timepoints
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Baseline",
  "visit_type": "Baseline",
  "order_index": 0
}
```

**Request Body:**
- `name` (required): TimePoint 이름
- `visit_type` (required): "Baseline" | "Visit" | "EOT" | "USV"
- `visit_no` (optional): CTIMS Visit Number
- `order_index` (required): 정렬 순서 (0부터 시작)

**Response 201 Created:**
```json
{
  "id": 1,
  "project_id": 1,
  "subject_id": 1,
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
  "error": "TimePoint name 'Baseline' already exists for this subject"
}
```

---

### 5.4 Study 할당

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

**Response 200 OK:**
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

**응답 필드 설명:**
- `assigned`: 새로 할당된 Study 목록
- `reassigned`: 다른 TimePoint에서 이동된 Study 목록
- `failed`: 할당 실패한 Study 목록

---

### 5.5 미할당 Study 조회

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
  }
]
```

---

## 6. 에러 처리

### 6.1 HTTP 상태 코드

| 코드 | 의미 | 사용 예시 |
|------|------|-----------|
| 200 | OK | 조회/수정 성공 |
| 201 | Created | 생성 성공 |
| 204 | No Content | 삭제 성공 |
| 400 | Bad Request | 유효성 검증 실패 |
| 404 | Not Found | 리소스 없음 |
| 409 | Conflict | 중복/제약조건 위반 |
| 500 | Internal Server Error | 서버 오류 |

### 6.2 에러 응답 형식

```json
{
  "error": "에러 메시지"
}
```

### 6.3 주요 에러 케이스

#### Subject 관련
- Subject Code 중복 (409)
- Patient ID 중복 (409)
- TimePoint 존재 시 삭제 불가 (409)
- Project 없음 (404)

#### TimePoint 관련
- Baseline 중복 (409)
- TimePoint 이름 중복 (409)
- Subject 없음 (404)
- 잘못된 VisitType (400)

#### Study 할당 관련
- TimePoint 없음 (404)
- Subject 없음 (404)
- 빈 Study 목록 (400)

---

## 7. 테스트

### 7.1 E2E 테스트 구조

```
tests/e2e/test_05_subject_timepoint.py
├── TestSubjectManagement (6 tests)
│   ├── test_01_create_subject
│   ├── test_02_get_subject
│   ├── test_03_get_subject_detail
│   ├── test_04_list_subjects_by_project
│   ├── test_05_update_subject
│   └── test_06_duplicate_subject_code
├── TestTimePointManagement (5 tests)
│   ├── test_01_create_baseline_timepoint
│   ├── test_02_create_visit_timepoint
│   ├── test_03_duplicate_baseline
│   ├── test_04_list_timepoints_by_subject
│   └── test_05_update_timepoint
├── TestStudyAssignment (5 tests)
│   ├── test_01_get_unassigned_studies
│   ├── test_02_assign_study_to_timepoint
│   ├── test_03_get_assigned_studies
│   ├── test_04_move_study_to_another_timepoint
│   └── test_05_unassign_study
├── TestCascadeProtection (3 tests)
│   ├── test_01_cannot_delete_subject_with_timepoints
│   ├── test_02_delete_timepoints_first
│   └── test_03_delete_subject_after_timepoints_removed
└── TestErrorCases (3 tests)
    ├── test_01_get_nonexistent_subject
    ├── test_02_create_subject_invalid_data
    └── test_03_create_timepoint_invalid_visit_type
```

### 7.2 테스트 실행

```bash
# 전체 테스트 실행
pytest tests/e2e/test_05_subject_timepoint.py -v

# 특정 테스트 클래스 실행
pytest tests/e2e/test_05_subject_timepoint.py::TestSubjectManagement -v

# 특정 테스트 실행
pytest tests/e2e/test_05_subject_timepoint.py::TestSubjectManagement::test_01_create_subject -v
```

### 7.3 테스트 결과

```
=================== 18 passed, 4 skipped, 1 warning in 2.17s ===================
```

---

## 8. 향후 계획

### 8.1 CTIMS 연동 🔮

- [ ] TimePoint/Visit 정보 동기화
- [ ] External Key 매핑
- [ ] Read-only 모드 전환
- [ ] 자동 Study 할당 (CTIMS 메타데이터 기반)

### 8.2 UI 개선 🎨

- [ ] 드래그 앤 드롭 지원
- [ ] 대량 할당 UI
- [ ] TimePoint 순서 변경 UI
- [ ] Study 미리보기

### 8.3 성능 최적화 ⚡

- [ ] Study 목록 페이지네이션
- [ ] 캐싱 전략
- [ ] 인덱스 최적화
- [ ] 배치 작업 최적화

---

## 9. 기술 스택

- **Backend**: Rust + Actix-web
- **Database**: PostgreSQL
- **ORM**: SQLx (compile-time checked queries)
- **API Documentation**: OpenAPI 3.0 (utoipa)
- **Testing**: pytest (E2E)
- **Authentication**: JWT + RBAC

---

## 10. 참고 문서

- [설계 문서](./api.md)
- [ERD 문서](./erd.md)
- [데이터베이스 마이그레이션](../../pacs-server/migrations/023_refactor_project_data_hierarchy.sql)
- [E2E 테스트 코드](../../tests/e2e/test_05_subject_timepoint.py)
- [Subject Controller](../../pacs-server/src/presentation/controllers/subject_controller.rs)
- [TimePoint Controller](../../pacs-server/src/presentation/controllers/timepoint_controller.rs)

---

## 11. 변경 이력

| 날짜 | 버전 | 변경 내용 |
|------|------|-----------|
| 2026-01-18 | 1.0.0 | 초기 구현 완료 (18개 테스트 통과) |

---

**문서 작성자**: AI Assistant
**최종 업데이트**: 2026-01-18


