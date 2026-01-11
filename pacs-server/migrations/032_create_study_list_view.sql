-- ============================================================================
-- Migration: 032_create_study_list_view
-- Description: Study List View / Extension Field 기능을 위한 테이블 생성
-- Author: System
-- Date: 2026-01-08
-- ============================================================================

-- ============================================================================
-- 1. study_list_view: View(컬럼 프리셋) 정의
-- ============================================================================
CREATE TABLE IF NOT EXISTS study_list_view (
    view_id         VARCHAR(100) PRIMARY KEY,
    view_name       VARCHAR(255) NOT NULL,
    is_system       BOOLEAN NOT NULL DEFAULT FALSE,
    owner_user_id   VARCHAR(100) NULL,
    scope_type      VARCHAR(50) NULL,      -- 'project' | 'user' | NULL (global)
    scope_id        VARCHAR(100) NULL,     -- project_id 또는 user_id
    description     TEXT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 인덱스
CREATE INDEX IF NOT EXISTS idx_study_list_view_owner ON study_list_view(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_study_list_view_scope ON study_list_view(scope_type, scope_id);

COMMENT ON TABLE study_list_view IS 'Study List View 정의 (컬럼 프리셋)';
COMMENT ON COLUMN study_list_view.view_id IS 'View 고유 식별자';
COMMENT ON COLUMN study_list_view.view_name IS 'View 표시 이름';
COMMENT ON COLUMN study_list_view.is_system IS '시스템 기본 View 여부 (true면 수정/삭제 불가)';
COMMENT ON COLUMN study_list_view.owner_user_id IS '소유자 사용자 ID (개인 View인 경우)';
COMMENT ON COLUMN study_list_view.scope_type IS '범위 타입 (project, user, null=global)';
COMMENT ON COLUMN study_list_view.scope_id IS '범위 ID (project_id 등)';

-- ============================================================================
-- 2. dicom_field_def: DICOM 표준 필드 정의
-- ============================================================================
CREATE TABLE IF NOT EXISTS dicom_field_def (
    field_key       VARCHAR(100) PRIMARY KEY,
    tag             VARCHAR(20) NOT NULL,      -- '00080020' 형식
    vr              VARCHAR(10) NOT NULL,      -- 'DA', 'PN' 등
    label           VARCHAR(255) NOT NULL,
    level           VARCHAR(20) NOT NULL DEFAULT 'study',  -- 'study' | 'series' | 'instance'
    value_type      VARCHAR(50) NOT NULL,      -- 'string' | 'number' | 'date'
    description     TEXT NULL,
    sortable        BOOLEAN NOT NULL DEFAULT FALSE,
    filterable      BOOLEAN NOT NULL DEFAULT FALSE,
    default_visible BOOLEAN NOT NULL DEFAULT FALSE,
    default_order   INTEGER NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dicom_field_def_level ON dicom_field_def(level);
CREATE INDEX IF NOT EXISTS idx_dicom_field_def_tag ON dicom_field_def(tag);

COMMENT ON TABLE dicom_field_def IS 'DICOM 표준 필드 정의';
COMMENT ON COLUMN dicom_field_def.field_key IS '필드 키 (예: PatientName, StudyDate)';
COMMENT ON COLUMN dicom_field_def.tag IS 'DICOM 태그 (예: 00100010)';
COMMENT ON COLUMN dicom_field_def.vr IS 'DICOM Value Representation';

-- ============================================================================
-- 3. ext_field_def: 확장 필드 정의
-- ============================================================================
CREATE TABLE IF NOT EXISTS ext_field_def (
    field_key       VARCHAR(100) PRIMARY KEY,
    label           VARCHAR(255) NOT NULL,
    level           VARCHAR(20) NOT NULL DEFAULT 'study',  -- 'study' | 'series' | 'instance'
    value_type      VARCHAR(50) NOT NULL,      -- 'string' | 'number' | 'date' | 'enum'
    description     TEXT NULL,
    source_system   VARCHAR(50) NOT NULL,      -- 'internal' | 'annotation' | 'workflow' | 'ctms' | 'ai'
    source_config   JSONB NULL,                -- 조회 설정 (테이블명, 컬럼명, API 경로 등)
    sortable        BOOLEAN NOT NULL DEFAULT FALSE,
    filterable      BOOLEAN NOT NULL DEFAULT FALSE,
    default_visible BOOLEAN NOT NULL DEFAULT FALSE,
    default_order   INTEGER NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ext_field_def_level ON ext_field_def(level);
CREATE INDEX IF NOT EXISTS idx_ext_field_def_source ON ext_field_def(source_system);

COMMENT ON TABLE ext_field_def IS '확장 필드 정의 (비표준 메타데이터)';
COMMENT ON COLUMN ext_field_def.source_system IS '데이터 소스 시스템';
COMMENT ON COLUMN ext_field_def.source_config IS '소스 조회 설정 (JSON)';

-- ============================================================================
-- 4. study_list_view_field: View에 포함되는 필드 구성
-- ============================================================================
CREATE TABLE IF NOT EXISTS study_list_view_field (
    view_id         VARCHAR(100) NOT NULL,
    field_source    VARCHAR(20) NOT NULL,      -- 'dicom' | 'extension'
    field_key       VARCHAR(100) NOT NULL,
    display_order   INTEGER NOT NULL DEFAULT 0,
    visible         BOOLEAN NOT NULL DEFAULT TRUE,
    pinned          BOOLEAN NOT NULL DEFAULT FALSE,
    width           INTEGER NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (view_id, field_source, field_key),
    FOREIGN KEY (view_id) REFERENCES study_list_view(view_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_study_list_view_field_view ON study_list_view_field(view_id);

COMMENT ON TABLE study_list_view_field IS 'View별 필드 구성';
COMMENT ON COLUMN study_list_view_field.field_source IS '필드 소스 (dicom 또는 extension)';
COMMENT ON COLUMN study_list_view_field.display_order IS '표시 순서';
COMMENT ON COLUMN study_list_view_field.pinned IS '고정 컬럼 여부';

-- ============================================================================
-- 5. 기본 데이터 삽입: DICOM 필드 정의
-- ============================================================================
INSERT INTO dicom_field_def (field_key, tag, vr, label, level, value_type, sortable, filterable, default_visible, default_order) VALUES
    -- Study Level 필드
    ('StudyInstanceUID', '0020000D', 'UI', 'Study Instance UID', 'study', 'string', FALSE, TRUE, FALSE, 100),
    ('StudyDate', '00080020', 'DA', 'Study Date', 'study', 'date', TRUE, TRUE, TRUE, 1),
    ('StudyTime', '00080030', 'TM', 'Study Time', 'study', 'string', TRUE, FALSE, FALSE, 2),
    ('StudyDescription', '00081030', 'LO', 'Study Description', 'study', 'string', TRUE, TRUE, TRUE, 5),
    ('AccessionNumber', '00080050', 'SH', 'Accession Number', 'study', 'string', TRUE, TRUE, TRUE, 6),
    ('PatientName', '00100010', 'PN', 'Patient Name', 'study', 'string', TRUE, TRUE, TRUE, 0),
    ('PatientID', '00100020', 'LO', 'Patient ID', 'study', 'string', TRUE, TRUE, TRUE, 3),
    ('PatientBirthDate', '00100030', 'DA', 'Patient Birth Date', 'study', 'date', TRUE, TRUE, FALSE, 10),
    ('PatientSex', '00100040', 'CS', 'Patient Sex', 'study', 'string', FALSE, TRUE, FALSE, 11),
    ('Modality', '00080060', 'CS', 'Modality', 'study', 'string', TRUE, TRUE, TRUE, 4),
    ('ModalitiesInStudy', '00080061', 'CS', 'Modalities in Study', 'study', 'string', FALSE, TRUE, TRUE, 4),
    ('ReferringPhysicianName', '00080090', 'PN', 'Referring Physician', 'study', 'string', TRUE, TRUE, FALSE, 12),
    ('InstitutionName', '00080080', 'LO', 'Institution Name', 'study', 'string', TRUE, TRUE, FALSE, 13),
    ('NumberOfStudyRelatedSeries', '00201206', 'IS', 'Number of Series', 'study', 'number', TRUE, FALSE, TRUE, 7),
    ('NumberOfStudyRelatedInstances', '00201208', 'IS', 'Number of Instances', 'study', 'number', TRUE, FALSE, FALSE, 8)
ON CONFLICT (field_key) DO NOTHING;

-- ============================================================================
-- 6. 기본 데이터 삽입: 확장 필드 정의
-- ============================================================================
INSERT INTO ext_field_def (field_key, label, level, value_type, source_system, source_config, sortable, filterable, default_visible, default_order) VALUES
    ('projectId', 'Project ID', 'study', 'string', 'internal', 
     '{"type": "db", "table": "project_data", "column": "project_id", "join_key": "study_instance_uid"}', 
     TRUE, TRUE, FALSE, 50),
    ('subjectNo', 'Subject No', 'study', 'string', 'internal',
     '{"type": "db", "table": "project_data", "column": "subject_no", "join_key": "study_instance_uid"}',
     TRUE, TRUE, FALSE, 51),
    ('visitName', 'Visit Name', 'study', 'string', 'internal',
     '{"type": "db", "table": "project_data", "column": "visit_name", "join_key": "study_instance_uid"}',
     TRUE, TRUE, FALSE, 52)
ON CONFLICT (field_key) DO NOTHING;

-- ============================================================================
-- 7. 기본 데이터 삽입: 시스템 기본 View
-- ============================================================================
INSERT INTO study_list_view (view_id, view_name, is_system, description) VALUES
    ('default', 'Default', TRUE, '기본 Study List View')
ON CONFLICT (view_id) DO NOTHING;

-- Default View 필드 구성
INSERT INTO study_list_view_field (view_id, field_source, field_key, display_order, visible, pinned) VALUES
    ('default', 'dicom', 'PatientName', 0, TRUE, TRUE),
    ('default', 'dicom', 'PatientID', 1, TRUE, FALSE),
    ('default', 'dicom', 'StudyDate', 2, TRUE, FALSE),
    ('default', 'dicom', 'StudyDescription', 3, TRUE, FALSE),
    ('default', 'dicom', 'ModalitiesInStudy', 4, TRUE, FALSE),
    ('default', 'dicom', 'AccessionNumber', 5, TRUE, FALSE),
    ('default', 'dicom', 'NumberOfStudyRelatedSeries', 6, TRUE, FALSE)
ON CONFLICT (view_id, field_source, field_key) DO NOTHING;

