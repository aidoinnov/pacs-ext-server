알겠어.
아래는 **지금까지의 논의(프로젝트 vs 서브젝트, CTIMS 개념, fallback 단계, 기존 ERD 존중)**를 모두 반영해서 정리한
👉 **“TimePoint / Subject 설계 기술문서 – 최종본”**이야.

이 문서는:

* 팀 내부 공유
* 외주/커서 설명
* 아키텍처 리뷰
* 이후 ERD·DDL의 기준 문서

로 그대로 써도 되는 수준으로 작성했어.

---

# 📘 TimePoint / Subject 관리 기능

## 기술 설계 문서 (Fallback 단계, CTIMS 연동 대비)

---

## 1. 문서 목적

본 문서는 **PACS Extension Server 환경에서 RECIST Report 작성을 위해 필요한
Subject(환자) 및 TimePoint 관리 기능의 기술 설계**를 정의한다.

본 설계는 다음 조건을 전제로 한다.

* 기존 PACS 워크리스트 구조를 변경하지 않는다.
* CTIMS(Clinical Trial Information Management System)와 **아직 연동되지 않은 fallback 단계**를 지원한다.
* 향후 CTIMS 연동 시 **DB 구조 변경 없이 확장 가능**해야 한다.

---

## 2. 배경 및 문제 정의

### 2.1 RECIST Report의 구조적 요구사항

RECIST Report는 다음과 같은 계층 구조를 가진다.

```
Project (연구)
 └─ Subject (환자)
      └─ TimePoint (Baseline, TP1, TP2…)
           └─ Study (DICOM Study 1..N)
```

핵심 전제는 다음과 같다.

* **TimePoint는 프로젝트 단위가 아니라, 환자(Subject) 단위 개념**이다.
* 하나의 Subject는 정확히 하나의 Baseline TimePoint를 가진다.
* 하나의 TimePoint는 하나 이상의 Study로 구성될 수 있다.

---

### 2.2 기존 PACS 구조의 한계

현재 PACS Extension Server는:

* Project 개념은 존재 (`security_project`)
* DICOM Study는 존재 (`project_data_study`)
* 그러나 **Subject(환자 단위 컨텍스트)**와 **TimePoint 개념이 존재하지 않음**

또한 fallback 단계에서는:

* CTIMS로부터 Subject / TimePoint 정보를 받을 수 없음
* 자동 TimePoint 추론을 하지 않음
* 사용자가 직접 Study를 TimePoint에 배치해야 함

---

## 3. 설계 원칙

본 설계는 다음 원칙을 따른다.

1. **도메인 정합성**

   * Baseline은 Project가 아닌 Subject에 귀속된다.
2. **최소 침습**

   * 기존 `project_data_study`, `annotation_*` 스키마를 수정하지 않는다.
3. **명시적 사용자 제어**

   * 자동 추론 없이 사용자가 TimePoint를 설정한다.
4. **CTIMS 연동 대비**

   * CTIMS key를 저장할 수 있는 필드를 사전에 확보한다.
5. **보드형 UX와 1:1 대응**

   * “Unassigned → TimePoint 이동” 개념이 DB에서도 자연스럽게 표현된다.

---

## 4. 핵심 개념 정의

### 4.1 Project

* 임상시험 단위
* 기존 `security_project` 테이블 사용

---

### 4.2 Subject (환자)

* CTIMS에서의 Subject = 환자
* Project 내에서 환자를 식별하는 논리적 엔티티
* fallback 단계에서는 **내부적으로 관리**

---

### 4.3 TimePoint

* 특정 Subject의 평가 시점
* 예:

  * Baseline
  * TP1
  * TP2
* **Subject 단위로만 존재**

---

### 4.4 Study ↔ TimePoint 관계

* 하나의 Study는:

  * 하나의 Subject에 속함
  * 그 Subject의 TimePoint 중 하나에만 속할 수 있음
* TimePoint에 속하지 않은 Study는 **Unassigned 상태**

---

## 5. 데이터 모델 설계

### 5.1 `project_subject`

> Project 내 환자(Subject) 정의

```sql
CREATE TABLE project_subject (
    id                      SERIAL PRIMARY KEY,
    project_id              INT NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_code            VARCHAR(50) NOT NULL,     -- A001, B002 (CTIMS subject name)
    external_subject_key    VARCHAR(100),             -- CTIMS subject pk (nullable)
    patient_id              VARCHAR(64),              -- PACS patient_id
    patient_name            TEXT,
    patient_birth_date      DATE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 제약 조건
    CONSTRAINT uq_project_subject_code UNIQUE (project_id, subject_code),
    CONSTRAINT uq_project_patient_id UNIQUE (project_id, patient_id),
    CONSTRAINT uq_external_subject_key UNIQUE (external_subject_key)
        WHERE external_subject_key IS NOT NULL
);

-- 인덱스
CREATE INDEX idx_project_subject_project ON project_subject(project_id);
CREATE INDEX idx_project_subject_patient ON project_subject(patient_id);
CREATE INDEX idx_project_subject_external ON project_subject(external_subject_key)
    WHERE external_subject_key IS NOT NULL;
```

#### 특징

* CTIMS 연동 전: 내부 subject_code 사용
* CTIMS 연동 후: `external_subject_key` 매핑
* 프로젝트 내 subject_code 유일성 보장
* 프로젝트 내 patient_id 유일성 보장 (동일 환자 중복 방지)

---

### 5.2 `subject_timepoint`

> Subject 단위 TimePoint 정의

```sql
CREATE TABLE subject_timepoint (
    id              SERIAL PRIMARY KEY,
    project_id      INT NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_id      INT NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    name            VARCHAR(50) NOT NULL,         -- BL, TP1, TP2
    visit_type      VARCHAR(20) NOT NULL          -- Baseline, Visit, EOT, USV
        CHECK (visit_type IN ('Baseline', 'Visit', 'EOT', 'USV')),
    visit_no        INT,                          -- CTIMS visit number (nullable)
    order_index     INT NOT NULL,
    external_key    VARCHAR(100),                 -- CTIMS timepoint key (nullable)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 제약 조건
    CONSTRAINT uq_subject_timepoint_name UNIQUE (subject_id, name),
    CONSTRAINT uq_external_timepoint_key UNIQUE (external_key)
        WHERE external_key IS NOT NULL
);

-- 핵심 제약: Subject당 Baseline은 정확히 1개
CREATE UNIQUE INDEX idx_subject_baseline
ON subject_timepoint (subject_id)
WHERE visit_type = 'Baseline';

-- 인덱스
CREATE INDEX idx_timepoint_subject ON subject_timepoint(subject_id);
CREATE INDEX idx_timepoint_project ON subject_timepoint(project_id);
CREATE INDEX idx_timepoint_order ON subject_timepoint(subject_id, order_index);
CREATE INDEX idx_timepoint_external ON subject_timepoint(external_key)
    WHERE external_key IS NOT NULL;
```

#### 핵심 제약

* **Baseline 유일성**: Partial Unique Index로 Subject당 Baseline 1개 보장
* **TimePoint 이름 유일성**: Subject 내에서 name 중복 불가
* **순서 보장**: order_index로 TimePoint 정렬

---

### 5.3 `subject_timepoint_study_map`

> Study ↔ TimePoint 매핑 (보드 UX 핵심 테이블)

```sql
CREATE TABLE subject_timepoint_study_map (
    id              SERIAL PRIMARY KEY,
    project_id      INT NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_id      INT NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    timepoint_id    INT NOT NULL REFERENCES subject_timepoint(id) ON DELETE CASCADE,
    study_id        INT NOT NULL REFERENCES project_data_study(id) ON DELETE CASCADE,
    assigned_by     INT REFERENCES security_user(id),
    assigned_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 핵심 제약: Subject 내에서 Study는 하나의 TimePoint만 가질 수 있음
    CONSTRAINT uq_subject_study UNIQUE (subject_id, study_id)
);

-- 인덱스
CREATE INDEX idx_study_map_timepoint ON subject_timepoint_study_map(timepoint_id);
CREATE INDEX idx_study_map_study ON subject_timepoint_study_map(study_id);
CREATE INDEX idx_study_map_subject ON subject_timepoint_study_map(subject_id);
CREATE INDEX idx_study_map_project ON subject_timepoint_study_map(project_id);
```

#### 핵심 제약

* **Study 유일성**: `UNIQUE (subject_id, study_id)`로 중복 할당 방지

👉 이 제약으로 인해:

* Study 재할당 = 기존 row 삭제 + 신규 row 생성
* “assign” API가 실제로는 **move 동작**
* TimePoint 삭제 시 CASCADE로 자동 매핑 해제

---

## 6. Unassigned 상태의 표현

* `subject_timepoint_study_map`에 row가 없는 Study
* 별도 상태 컬럼 없이 **존재 여부로 판단**

```sql
-- Unassigned Studies 조회 (특정 Subject 기준)
SELECT s.*
FROM project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id
LEFT JOIN subject_timepoint_study_map m
  ON m.study_id = s.id AND m.subject_id = :subject_id
WHERE pd.project_id = :project_id
  AND m.id IS NULL;

-- Unassigned Studies 조회 (전체 프로젝트 기준)
SELECT s.*
FROM project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id
LEFT JOIN subject_timepoint_study_map m ON m.study_id = s.id
WHERE pd.project_id = :project_id
  AND m.id IS NULL;
```

---

## 7. UX 설계와 DB의 대응 관계

| UX 개념            | DB 동작                         | SQL 예시 |
| ---------------- | ----------------------------- | ------- |
| Unassigned 목록    | map 테이블에 없는 Study             | LEFT JOIN + IS NULL |
| Baseline으로 이동    | INSERT map                    | INSERT INTO map |
| TP1 → Unassigned | DELETE map                    | DELETE FROM map |
| TP1 → TP2 이동     | DELETE + INSERT (트랜잭션)        | BEGIN; DELETE; INSERT; COMMIT; |
| TimePoint 삭제     | CASCADE DELETE                | DELETE FROM timepoint |

### 주요 트랜잭션 예시

#### Study를 TimePoint에 할당
```sql
-- Unassigned → Baseline
INSERT INTO subject_timepoint_study_map
  (project_id, subject_id, timepoint_id, study_id, assigned_by)
VALUES
  (:project_id, :subject_id, :timepoint_id, :study_id, :user_id);
```

#### Study를 다른 TimePoint로 이동
```sql
-- TP1 → TP2 (원자적 처리)
BEGIN;
  DELETE FROM subject_timepoint_study_map
  WHERE subject_id = :subject_id AND study_id = :study_id;

  INSERT INTO subject_timepoint_study_map
    (project_id, subject_id, timepoint_id, study_id, assigned_by)
  VALUES
    (:project_id, :subject_id, :new_timepoint_id, :study_id, :user_id);
COMMIT;
```

#### Study를 Unassigned로 이동
```sql
-- TP1 → Unassigned
DELETE FROM subject_timepoint_study_map
WHERE subject_id = :subject_id AND study_id = :study_id;
```

#### TimePoint 삭제 (모든 Study Unassigned 처리)
```sql
-- CASCADE로 자동 처리됨
DELETE FROM subject_timepoint
WHERE id = :timepoint_id;
-- subject_timepoint_study_map의 관련 row들이 자동 삭제됨
```

---

## 8. CTIMS 연동 확장 전략

CTIMS 연동 시:

* `project_subject.external_subject_key` ← CTIMS subject PK
* `subject_timepoint.external_key` ← CTIMS timepoint PK
* `visit_no`, `visit_type`는 CTIMS 값을 authoritative source로 사용

정책:

* UI: read-only 전환
* DB: 구조 변경 없음
* 기존 fallback 데이터 유지 가능

---

## 9. 설계 요약

> **Baseline은 프로젝트의 속성이 아니라
> 환자(Subject)의 첫 평가 시점이다.
> 따라서 TimePoint는 반드시 Subject 단위로 모델링되어야 한다.**

본 설계는:

* 임상 도메인 정합성
* 기존 PACS 구조 보존
* fallback 단계 요구사항
* CTIMS 연동 확장성

을 모두 만족한다.

---

## 10. 성능 최적화 전략

### 10.1 인덱스 전략 요약

| 테이블 | 인덱스 | 목적 |
|--------|--------|------|
| `project_subject` | `(project_id)` | 프로젝트별 Subject 조회 |
| | `(patient_id)` | PACS 환자 ID 검색 |
| | `(external_subject_key)` | CTIMS 연동 조회 |
| `subject_timepoint` | `(subject_id)` | Subject별 TimePoint 조회 |
| | `(subject_id, order_index)` | TimePoint 정렬 |
| | `(subject_id) WHERE visit_type='Baseline'` | Baseline 유일성 |
| `subject_timepoint_study_map` | `(timepoint_id)` | TimePoint별 Study 조회 |
| | `(study_id)` | Study 할당 상태 확인 |
| | `(subject_id, study_id)` | 중복 할당 방지 (UNIQUE) |

### 10.2 쿼리 최적화 포인트

* **Unassigned 조회**: LEFT JOIN + IS NULL 패턴 사용
* **TimePoint별 Study 조회**: timepoint_id 인덱스 활용
* **Study 재할당**: 트랜잭션 내 DELETE + INSERT (UPSERT 불가)

---

## 11. 관련 문서

- [전체 데이터베이스 ERD](../database/ERD.md)
- [기존 Project Data 스키마](../database/ERD.md#project-data-schema)
- [마이그레이션 스크립트](../../migrations/040_create_subject_timepoint.sql)

---

## 12. 다음 단계

본 문서를 기준으로 다음 작업이 가능하다.

1. ✅ ERD 최종 확정 (전체 스키마 반영)
2. ✅ PostgreSQL DDL 작성
3. ⏳ assign/remove 트랜잭션 구현
4. ⏳ Report 진입 시 Subject/TimePoint 해석 로직 정의
5. ⏳ API 엔드포인트 설계 및 구현
