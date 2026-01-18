-- Migration: Subject & TimePoint Management for Clinical Trials
-- Created: 2026-01-18
-- Description: Creates tables for managing Subjects and TimePoints in clinical trial projects
--              Supports RECIST Report workflow with Baseline/TP1/TP2... structure
--              Designed for CTIMS integration with fallback support

-- ========================================
-- ENUMS
-- ========================================

-- TimePoint visit type
CREATE TYPE timepoint_visit_type_enum AS ENUM ('Baseline', 'Visit', 'EOT', 'USV');

-- ========================================
-- TABLES
-- ========================================

-- 프로젝트별 환자(Subject) 관리
CREATE TABLE project_subject (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_code VARCHAR(50) NOT NULL,           -- A001, B002 (CTIMS subject name)
    external_subject_key VARCHAR(100),           -- CTIMS subject pk (nullable, for future integration)
    patient_id VARCHAR(64),                      -- PACS patient_id
    patient_name TEXT,
    patient_birth_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT uq_project_subject_code UNIQUE (project_id, subject_code),
    CONSTRAINT uq_project_patient_id UNIQUE (project_id, patient_id),
    CONSTRAINT uq_external_subject_key UNIQUE (external_subject_key) 
        WHERE external_subject_key IS NOT NULL
);

-- Subject별 TimePoint 관리
CREATE TABLE subject_timepoint (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,                   -- BL, TP1, TP2
    visit_type timepoint_visit_type_enum NOT NULL,
    visit_no INTEGER,                            -- CTIMS visit number (nullable)
    order_index INTEGER NOT NULL,
    external_key VARCHAR(100),                   -- CTIMS timepoint key (nullable, for future integration)
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT uq_subject_timepoint_name UNIQUE (subject_id, name),
    CONSTRAINT uq_external_timepoint_key UNIQUE (external_key) 
        WHERE external_key IS NOT NULL
);

-- TimePoint ↔ Study 매핑 (보드 UX 핵심 테이블)
CREATE TABLE subject_timepoint_study_map (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    timepoint_id INTEGER NOT NULL REFERENCES subject_timepoint(id) ON DELETE CASCADE,
    study_id INTEGER NOT NULL REFERENCES project_data_study(id) ON DELETE CASCADE,
    assigned_by INTEGER REFERENCES security_user(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 핵심 제약: Subject 내에서 Study는 하나의 TimePoint만 가질 수 있음
    CONSTRAINT uq_subject_study UNIQUE (subject_id, study_id)
);

-- ========================================
-- INDEXES
-- ========================================

-- project_subject 인덱스
CREATE INDEX idx_project_subject_project ON project_subject(project_id);
CREATE INDEX idx_project_subject_patient ON project_subject(patient_id);
CREATE INDEX idx_project_subject_external ON project_subject(external_subject_key) 
    WHERE external_subject_key IS NOT NULL;

-- subject_timepoint 인덱스
CREATE INDEX idx_timepoint_subject ON subject_timepoint(subject_id);
CREATE INDEX idx_timepoint_project ON subject_timepoint(project_id);
CREATE INDEX idx_timepoint_order ON subject_timepoint(subject_id, order_index);
CREATE INDEX idx_timepoint_external ON subject_timepoint(external_key) 
    WHERE external_key IS NOT NULL;

-- 핵심 제약: Subject당 Baseline은 정확히 1개
CREATE UNIQUE INDEX idx_subject_baseline 
ON subject_timepoint (subject_id) 
WHERE visit_type = 'Baseline';

-- subject_timepoint_study_map 인덱스
CREATE INDEX idx_study_map_timepoint ON subject_timepoint_study_map(timepoint_id);
CREATE INDEX idx_study_map_study ON subject_timepoint_study_map(study_id);
CREATE INDEX idx_study_map_subject ON subject_timepoint_study_map(subject_id);
CREATE INDEX idx_study_map_project ON subject_timepoint_study_map(project_id);

-- ========================================
-- TRIGGERS
-- ========================================

-- project_subject updated_at 트리거
CREATE OR REPLACE FUNCTION update_project_subject_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_project_subject_updated_at
BEFORE UPDATE ON project_subject
FOR EACH ROW
EXECUTE FUNCTION update_project_subject_updated_at();

-- subject_timepoint updated_at 트리거
CREATE OR REPLACE FUNCTION update_subject_timepoint_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_subject_timepoint_updated_at
BEFORE UPDATE ON subject_timepoint
FOR EACH ROW
EXECUTE FUNCTION update_subject_timepoint_updated_at();

-- ========================================
-- COMMENTS
-- ========================================

-- project_subject 주석
COMMENT ON TABLE project_subject IS '프로젝트별 환자(Subject) 관리 - CTIMS 연동 대비';
COMMENT ON COLUMN project_subject.subject_code IS 'Subject 코드 (A001, B002 등)';
COMMENT ON COLUMN project_subject.external_subject_key IS 'CTIMS Subject PK (연동 시 사용)';
COMMENT ON COLUMN project_subject.patient_id IS 'PACS Patient ID';
COMMENT ON COLUMN project_subject.patient_name IS '환자 이름';
COMMENT ON COLUMN project_subject.patient_birth_date IS '환자 생년월일';

-- subject_timepoint 주석
COMMENT ON TABLE subject_timepoint IS 'Subject별 평가 시점(TimePoint) 관리 - Baseline, TP1, TP2...';
COMMENT ON COLUMN subject_timepoint.name IS 'TimePoint 이름 (BL, TP1, TP2)';
COMMENT ON COLUMN subject_timepoint.visit_type IS 'Visit 타입 (Baseline/Visit/EOT/USV)';
COMMENT ON COLUMN subject_timepoint.visit_no IS 'CTIMS Visit Number (연동 시 사용)';
COMMENT ON COLUMN subject_timepoint.order_index IS 'TimePoint 정렬 순서';
COMMENT ON COLUMN subject_timepoint.external_key IS 'CTIMS TimePoint Key (연동 시 사용)';

-- subject_timepoint_study_map 주석
COMMENT ON TABLE subject_timepoint_study_map IS 'TimePoint ↔ Study 매핑 - 보드 UX 핵심 테이블';
COMMENT ON COLUMN subject_timepoint_study_map.assigned_by IS 'Study를 TimePoint에 할당한 사용자';
COMMENT ON COLUMN subject_timepoint_study_map.assigned_at IS 'Study 할당 시각';

-- ========================================
-- VALIDATION QUERIES (for testing)
-- ========================================

-- 1. Subject당 Baseline 1개 제약 확인
-- SELECT subject_id, COUNT(*)
-- FROM subject_timepoint
-- WHERE visit_type = 'Baseline'
-- GROUP BY subject_id
-- HAVING COUNT(*) > 1;

-- 2. Study 중복 할당 확인
-- SELECT subject_id, study_id, COUNT(*)
-- FROM subject_timepoint_study_map
-- GROUP BY subject_id, study_id
-- HAVING COUNT(*) > 1;

-- 3. Unassigned Studies 조회 (특정 Subject)
-- SELECT s.*
-- FROM project_data_study s
-- INNER JOIN project_data pd ON pd.study_id = s.id
-- LEFT JOIN subject_timepoint_study_map m
--   ON m.study_id = s.id AND m.subject_id = :subject_id
-- WHERE pd.project_id = :project_id
--   AND m.id IS NULL;

