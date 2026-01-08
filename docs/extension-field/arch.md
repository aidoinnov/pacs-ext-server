# 📄 Study List View / Extension Metadata 구현 기술문서 (초안)

## 1. 목적 (Purpose)

본 문서는 **dcm4chee-arc 기반 PACS 환경**에서
Study 목록 조회 시 **DICOM 표준 메타데이터 + 비표준 확장 메타데이터**를
**DICOMweb 스타일을 유지한 채** 제공하기 위한
**View / Column 구성 및 Backend 구현 가이드**를 정의한다.

본 구현은 다음을 목표로 한다.

* DICOM 표준(JSON Model, QIDO-RS 스타일) 유지
* Private Tag 사용 없이 확장 메타 제공
* Study List 컬럼 구성을 서버에서 관리
* 확장 메타는 **저장하지 않고**, 주변 시스템(DB/API)에서 조회
* UI는 View 선택만 수행 (컬럼 로직 비노출)

---

## 2. 범위 (Scope)

### 포함

* Study List View / Column 구성 관리
* DICOM + Extension Metadata 조합 응답
* View 기반 Study 목록 조회 API
* Config/Metadata 중심 DB 테이블 설계

### 제외 (초기 단계)

* 캐싱 전략
* 확장 메타 저장용 테이블
* Series / Instance 레벨 확장 (추후 확장 가능)

---

## 3. 전체 아키텍처 개요

```text
[Client(UI)]
  └─ View 선택 (view_id)

[Backend]
  ├─ Study List View Config (DB)
  ├─ DICOM Study 조회 (dcm4chee / QIDO-RS)
  ├─ Extension Metadata 조회 (CTMS / Annotation / Workflow DB)
  └─ 응답 조합 (dicom + extensions)

[DICOM Storage]
  └─ dcm4chee-arc (변경 없음)

[External Systems]
  ├─ CTMS DB
  ├─ Annotation DB
  └─ 기타 메타 DB
```

---

## 4. 핵심 개념 정의

### 4.1 Study List View

* 사용자가 선택하는 **컬럼 프리셋**
* “Default / Research / Clinical” 등의 논리적 묶음

### 4.2 Field

* Study List에 표시되는 하나의 컬럼
* 출처에 따라 구분:

  * `dicom` : DICOM 표준 메타
  * `extension` : 비표준 확장 메타

### 4.3 Extension Metadata

* DICOM Tag가 아닌 메타데이터
* Project, Subject No, Visit 정보 등
* **Virtual Extension Block**으로 응답

---

## 5. DB 설계 (Config / View 전용)

### 5.1 study_list_view

**역할**

* View(컬럼 프리셋)의 정체성 및 소유 정보 관리

```sql
CREATE TABLE study_list_view (
  view_id        VARCHAR PRIMARY KEY,
  view_name      VARCHAR NOT NULL,
  is_system      BOOLEAN NOT NULL DEFAULT FALSE,
  owner_user_id  VARCHAR NULL,
  scope_type     VARCHAR NULL,   -- optional: project
  scope_id       VARCHAR NULL,   -- optional: project_id
  created_at     TIMESTAMP NOT NULL DEFAULT NOW()
);
```

---

### 5.2 study_list_view_field

**역할**

* View에 포함되는 컬럼 정의
* 컬럼 순서 및 표시 여부 관리

```sql
CREATE TABLE study_list_view_field (
  view_id        VARCHAR NOT NULL,
  field_source   VARCHAR NOT NULL,  -- 'dicom' | 'extension'
  field_key      VARCHAR NOT NULL,
  display_order  INT NOT NULL,
  visible        BOOLEAN NOT NULL DEFAULT TRUE,
  pinned         BOOLEAN NOT NULL DEFAULT FALSE,
  width          INT NULL,
  PRIMARY KEY (view_id, field_source, field_key)
);
```

---

### 5.3 dicom_field_def (DICOM 필드 정의)

**역할**

* DICOM 표준 필드의 메타 정의
* UI 힌트 및 정렬/필터 가능 여부 관리

```sql
CREATE TABLE dicom_field_def (
  field_key        VARCHAR PRIMARY KEY,  -- 'StudyDate', 'PatientName' 등
  tag              VARCHAR NOT NULL,     -- '00080020', '00100010' 등
  vr               VARCHAR NOT NULL,     -- 'DA', 'PN' 등
  label            VARCHAR NOT NULL,
  level            VARCHAR NOT NULL,     -- 'study' | 'series' | 'instance'
  value_type       VARCHAR NOT NULL,     -- string | number | date
  description      TEXT,
  sortable         BOOLEAN DEFAULT FALSE,
  filterable       BOOLEAN DEFAULT FALSE,
  default_visible  BOOLEAN DEFAULT FALSE,
  default_order    INTEGER,
  created_at       TIMESTAMP DEFAULT NOW()
);
```

---

### 5.4 ext_field_def (확장 필드 정의)

**역할**

* 확장 메타 필드의 의미/타입/UI 힌트 정의
* 데이터 소스 정보 포함 (어느 시스템에서 조회할지)
* 실제 값은 저장하지 않음

```sql
CREATE TABLE ext_field_def (
  field_key        VARCHAR PRIMARY KEY,
  label            VARCHAR NOT NULL,
  level            VARCHAR NOT NULL,     -- study | series | instance
  value_type       VARCHAR NOT NULL,     -- string | number | date | enum
  description      TEXT,
  source_system    VARCHAR NOT NULL,     -- 'ctms' | 'annotation' | 'workflow' | 'internal'
  source_config    JSONB,                -- 조회 설정 (테이블명, 컬럼명, API 경로 등)
  sortable         BOOLEAN DEFAULT FALSE,
  filterable       BOOLEAN DEFAULT FALSE,
  default_visible  BOOLEAN DEFAULT FALSE,
  default_order    INTEGER,
  created_at       TIMESTAMP DEFAULT NOW()
);
```

**source_config 예시**

```json
// internal DB 조회
{
  "type": "db",
  "table": "project_data",
  "column": "subject_no",
  "join_key": "study_instance_uid"
}

// 외부 API 조회
{
  "type": "api",
  "endpoint": "/ctms/studies/{study_uid}/metadata",
  "field_path": "subjectNo"
}
```

---

## 6. API 설계

> **Base Path**: `/api/v1`

### 6.1 View 목록 조회

```http
GET /api/v1/study-list-views
```

**Response**

```json
{
  "items": [
    {
      "viewId": "default",
      "viewName": "Default",
      "isSystem": true,
      "scopeType": null,
      "scopeId": null
    },
    {
      "viewId": "research",
      "viewName": "Research",
      "isSystem": true,
      "scopeType": "project",
      "scopeId": "LUNG_CANCER_01"
    }
  ]
}
```

---

### 6.2 View별 컬럼 구성 조회

```http
GET /api/v1/study-list-views/{viewId}
```

**Response**

```json
{
  "viewId": "research",
  "viewName": "Research",
  "isSystem": true,
  "fields": [
    { "source": "dicom", "key": "StudyDate", "label": "Study Date", "order": 0, "visible": true },
    { "source": "dicom", "key": "PatientName", "label": "Patient Name", "order": 1, "visible": true },
    { "source": "extension", "key": "subjectNo", "label": "Subject No", "order": 2, "visible": true },
    { "source": "extension", "key": "timePoint", "label": "Time Point", "order": 3, "visible": true }
  ]
}
```

---

### 6.3 필드 정의 조회

```http
GET /api/v1/study-list-fields
```

**Query Parameters**

* `source` (optional): `dicom` | `extension` - 필터링

**Response**

```json
{
  "items": [
    {
      "source": "dicom",
      "key": "StudyDate",
      "tag": "00080020",
      "label": "Study Date",
      "valueType": "date",
      "sortable": true,
      "filterable": true
    },
    {
      "source": "extension",
      "key": "subjectNo",
      "label": "Subject No",
      "valueType": "string",
      "sourceSystem": "internal",
      "sortable": true,
      "filterable": true
    }
  ]
}
```

---

### 6.4 Study List 조회 (View 기반)

```http
GET /api/v1/studies?viewId=research
```

또는 복잡한 필터링이 필요한 경우:

```http
POST /api/v1/studies/search
{
  "viewId": "research",
  "filters": {
    "StudyDate": { "from": "20240101", "to": "20241231" },
    "subjectNo": "SUBJ-001"
  },
  "sort": {
    "field": "StudyDate",
    "order": "desc"
  },
  "pagination": {
    "offset": 0,
    "limit": 50
  }
}
```

---

## 7. Study List 응답 포맷

### 7.1 응답 구조

```json
{
  "items": [
    {
      "dicom": {
        "0020000D": { "vr": "UI", "Value": ["1.2.840..."] },
        "00081030": { "vr": "LO", "Value": ["Chest CT"] }
      },
      "extensions": {
        "project": "LUNG_CANCER_01",
        "subjectNo": "SUBJ-0007",
        "timePoint": "Baseline",
        "visitType": "Screening",
        "visitNumber": 1,
        "annotationCount": 5,
        "status": "Reviewed"
      }
    }
  ]
}
```

### 7.2 설계 원칙

* `dicom` 블록: **순수 DICOM JSON Model**
* `extensions` 블록: 비표준 메타 (Tag 아님)
* 클라이언트는 `source + key` 기준으로 접근
* DICOM / Extension 파싱 책임은 서버

---

## 8. 서버 내부 처리 흐름

### 8.1 Study List 조회 시퀀스

```text
1. Request: GET /api/v1/studies?viewId=research

2. View 조회
   └─ study_list_view + study_list_view_field JOIN
   └─ 결과: [{ source: dicom, key: StudyDate }, { source: extension, key: subjectNo }, ...]

3. 필드 분류
   ├─ DICOM 필드: [StudyDate, PatientName, StudyDescription]
   └─ Extension 필드: [subjectNo, timePoint, annotationCount]

4. DICOM 데이터 조회
   └─ QIDO-RS: GET /dcm4chee/aets/DCM4CHEE/rs/studies?includefield=00080020,00100010,...
   └─ 결과: Study[] (DICOM JSON)

5. Extension 데이터 조회 (source_system별 그룹핑)
   ├─ internal: SELECT subject_no, time_point FROM project_data WHERE study_instance_uid IN (...)
   ├─ annotation: SELECT count(*) FROM annotations WHERE study_uid IN (...) GROUP BY study_uid
   └─ ctms: GET /ctms/studies/batch?uids=1.2.840...,1.2.840...

6. 데이터 병합 (study_instance_uid 기준)
   └─ { dicom: {...}, extensions: {...} }

7. 정렬/필터링 적용
   ├─ DICOM 필드 정렬: QIDO-RS orderby 사용 (가능한 경우)
   └─ Extension 필드 정렬: 메모리 정렬

8. Response 반환
```

### 8.2 Extension 조회 전략

| source_system | 조회 방식 | 비고 |
|---------------|----------|------|
| `internal` | Extension DB 직접 쿼리 | project_data, user_note 등 |
| `annotation` | Extension DB 직접 쿼리 | annotation 테이블 |
| `workflow` | Extension DB 직접 쿼리 | workflow_status 등 |
| `ctms` | 외부 API 호출 | Batch API 사용 권장 |
| `ai` | 외부 API 호출 | AI 분석 결과 등 |

### 8.3 에러 처리

* Extension 조회 실패 시: 해당 필드는 `null` 반환 (전체 실패 X)
* 외부 API 타임아웃: 기본값 또는 `null` 반환
* 로깅: 실패한 source_system과 study_uid 기록

---

## 9. 비결정 사항 / 추후 확장

* Extension Metadata 캐싱 전략
* Series / Instance 레벨 확장
* 사용자별 View 커스터마이징
* Private Tag / FHIR 변환 여부

---

## 10. 결론

본 설계는 다음을 보장한다.

* DICOMweb 철학 유지
* Private Tag 충돌 리스크 제거
* View/컬럼 로직의 서버 집중화
* CTMS / AI / Viewer 확장 용이성
* 초기 구현 난이도 최소화
