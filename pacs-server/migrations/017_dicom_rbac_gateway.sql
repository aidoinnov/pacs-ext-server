-- Migration: DICOM RBAC Gateway Schema Extension
-- Created: 2025-01-27
-- Description: Extends existing schema for DICOM Web RBAC Gateway implementation
-- Based on: docs/technical/db-schema/dicom-rbac-gateway-schema.md

-- ==========================
-- INSTITUTION TABLES
-- ==========================

-- 사용자 소속 기관 (Security Schema)
CREATE TABLE security_institution (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    institution_code VARCHAR(128) UNIQUE NOT NULL,
    institution_name VARCHAR(255) NOT NULL,
    institution_type VARCHAR(50) DEFAULT 'HOSPITAL', -- HOSPITAL, CLINIC, RESEARCH
    address TEXT,
    phone VARCHAR(50),
    email VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 데이터 소속 기관 (Project Data Schema)
CREATE TABLE project_data_institution (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    institution_code VARCHAR(128) UNIQUE NOT NULL,
    institution_name VARCHAR(255) NOT NULL,
    institution_type VARCHAR(50) DEFAULT 'HOSPITAL',
    address TEXT,
    phone VARCHAR(50),
    email VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ==========================
-- PROJECT DATA EXTENSIONS
-- ==========================

-- project_data_study 테이블 확장
ALTER TABLE project_data_study 
ADD COLUMN data_institution_id INTEGER REFERENCES project_data_institution(id),
ADD COLUMN institution_code VARCHAR(128),
ADD COLUMN accession_no VARCHAR(255),
ADD COLUMN modality VARCHAR(64),
ADD COLUMN patient_sex CHAR(1),
ADD COLUMN study_time VARCHAR(32),
ADD COLUMN referring_physician VARCHAR(255),
ADD COLUMN performing_physician VARCHAR(255),
ADD COLUMN series_count INTEGER DEFAULT 0,
ADD COLUMN instance_count INTEGER DEFAULT 0,
ADD COLUMN is_active BOOLEAN DEFAULT true,
ADD COLUMN sync_status VARCHAR(20) DEFAULT 'PENDING';

-- project_data_series 테이블 확장
ALTER TABLE project_data_series 
ADD COLUMN body_part VARCHAR(128),
ADD COLUMN station_name VARCHAR(255),
ADD COLUMN series_time VARCHAR(32),
ADD COLUMN performing_physician VARCHAR(255),
ADD COLUMN protocol_name VARCHAR(255),
ADD COLUMN instance_count INTEGER DEFAULT 0,
ADD COLUMN is_active BOOLEAN DEFAULT true,
ADD COLUMN sync_status VARCHAR(20) DEFAULT 'PENDING';

-- project_data_instance 테이블 생성
CREATE TABLE project_data_instance (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_uid VARCHAR(255) NOT NULL,
    sop_class_uid VARCHAR(255),
    instance_number INTEGER,
    content_date VARCHAR(32),
    content_time VARCHAR(32),
    is_active BOOLEAN DEFAULT true,
    sync_status VARCHAR(20) DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (series_id, instance_uid)
);

-- ==========================
-- ACCESS CONTROL EXTENSIONS
-- ==========================

-- project_data_access 테이블 확장
ALTER TABLE project_data_access 
ADD COLUMN instance_id INTEGER REFERENCES project_data_instance(id) ON DELETE CASCADE,
ADD COLUMN user_institution_id INTEGER REFERENCES security_institution(id),
ADD COLUMN data_institution_id INTEGER REFERENCES project_data_institution(id),
ADD COLUMN access_scope VARCHAR(50) DEFAULT 'FULL', -- FULL, LIMITED, READ_ONLY
ADD COLUMN expires_at TIMESTAMPTZ,
ADD COLUMN granted_by INTEGER REFERENCES security_user(id),
ADD COLUMN granted_at TIMESTAMPTZ;

-- security_access_condition 테이블 확장
ALTER TABLE security_access_condition 
ADD COLUMN institution_id INTEGER REFERENCES project_data_institution(id),
ADD COLUMN patient_id VARCHAR(255),
ADD COLUMN study_uid_pattern VARCHAR(255),
ADD COLUMN series_uid_pattern VARCHAR(255),
ADD COLUMN modality VARCHAR(64),
ADD COLUMN date_range_start DATE,
ADD COLUMN date_range_end DATE;

-- ==========================
-- INSTITUTION ACCESS CONTROL
-- ==========================

-- 기관 간 접근 권한 테이블
CREATE TABLE security_institution_data_access (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_institution_id INTEGER NOT NULL REFERENCES security_institution(id),
    data_institution_id INTEGER NOT NULL REFERENCES project_data_institution(id),
    access_level VARCHAR(20) DEFAULT 'READ', -- READ, WRITE, ADMIN
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_institution_id, data_institution_id)
);

-- Role별 DICOM 접근 조건
CREATE TABLE security_role_dicom_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    access_condition_id INTEGER NOT NULL REFERENCES security_access_condition(id) ON DELETE CASCADE,
    priority INTEGER DEFAULT 0, -- 우선순위 (높을수록 먼저 평가)
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (role_id, access_condition_id)
);

-- Project별 DICOM 접근 조건
CREATE TABLE security_project_dicom_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    access_condition_id INTEGER NOT NULL REFERENCES security_access_condition(id) ON DELETE CASCADE,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_id, access_condition_id)
);

-- ==========================
-- USER INSTITUTION CONNECTION
-- ==========================

-- security_user에 기관 연결
ALTER TABLE security_user 
ADD COLUMN institution_id INTEGER REFERENCES security_institution(id);

-- ==========================
-- INDEXES FOR PERFORMANCE
-- ==========================

-- 기관 관련 인덱스
CREATE INDEX idx_security_user_institution ON security_user(institution_id);
CREATE INDEX idx_project_data_study_institution ON project_data_study(data_institution_id);
CREATE INDEX idx_project_data_study_institution_code ON project_data_study(institution_code);

-- 접근 제어 인덱스
CREATE INDEX idx_project_data_access_user_resource ON project_data_access(user_id, resource_level, study_id, series_id, instance_id);
CREATE INDEX idx_project_data_access_institution ON project_data_access(user_institution_id, data_institution_id);
CREATE INDEX idx_project_data_access_expires ON project_data_access(expires_at);

-- 룰 기반 인덱스
CREATE INDEX idx_security_role_dicom_condition_role ON security_role_dicom_condition(role_id);
CREATE INDEX idx_security_project_dicom_condition_project ON security_project_dicom_condition(project_id);
CREATE INDEX idx_security_access_condition_institution ON security_access_condition(institution_id);
CREATE INDEX idx_security_access_condition_modality ON security_access_condition(modality);

-- Instance 관련 인덱스
CREATE INDEX idx_project_data_instance_series ON project_data_instance(series_id);
CREATE INDEX idx_project_data_instance_uid ON project_data_instance(instance_uid);

-- 기관 간 접근 권한 인덱스
CREATE INDEX idx_security_institution_data_access_user ON security_institution_data_access(user_institution_id);
CREATE INDEX idx_security_institution_data_access_data ON security_institution_data_access(data_institution_id);

-- ==========================
-- COMMENTS
-- ==========================

COMMENT ON TABLE security_institution IS '사용자 소속 의료기관 정보';
COMMENT ON TABLE project_data_institution IS 'DICOM 데이터 소속 의료기관 정보';
COMMENT ON TABLE project_data_instance IS 'DICOM Instance 레벨 메타데이터';
COMMENT ON TABLE security_institution_data_access IS '기관 간 데이터 접근 권한';
COMMENT ON TABLE security_role_dicom_condition IS '역할별 DICOM 접근 조건';
COMMENT ON TABLE security_project_dicom_condition IS '프로젝트별 DICOM 접근 조건';

COMMENT ON COLUMN project_data_study.data_institution_id IS '데이터가 생성된 기관 ID';
COMMENT ON COLUMN project_data_study.institution_code IS '기관 코드 (예: HSP001)';
COMMENT ON COLUMN project_data_study.accession_no IS 'Accession Number';
COMMENT ON COLUMN project_data_study.modality IS 'ModalitiesInStudy';
COMMENT ON COLUMN project_data_study.patient_sex IS '환자 성별 (M/F/O)';
COMMENT ON COLUMN project_data_study.study_time IS '검사 시간';
COMMENT ON COLUMN project_data_study.referring_physician IS '의뢰의';
COMMENT ON COLUMN project_data_study.performing_physician IS '시행의';
COMMENT ON COLUMN project_data_study.series_count IS 'Series 개수';
COMMENT ON COLUMN project_data_study.instance_count IS 'Instance 개수';
COMMENT ON COLUMN project_data_study.is_active IS '활성 상태';
COMMENT ON COLUMN project_data_study.sync_status IS '동기화 상태 (PENDING/SYNCING/SYNCED/ERROR)';

COMMENT ON COLUMN project_data_series.body_part IS '촬영 부위';
COMMENT ON COLUMN project_data_series.station_name IS '스테이션명';
COMMENT ON COLUMN project_data_series.series_time IS 'Series 시간';
COMMENT ON COLUMN project_data_series.performing_physician IS '시행의';
COMMENT ON COLUMN project_data_series.protocol_name IS '프로토콜명';
COMMENT ON COLUMN project_data_series.instance_count IS 'Instance 개수';
COMMENT ON COLUMN project_data_series.is_active IS '활성 상태';
COMMENT ON COLUMN project_data_series.sync_status IS '동기화 상태';

COMMENT ON COLUMN project_data_instance.instance_uid IS 'SOPInstanceUID';
COMMENT ON COLUMN project_data_instance.sop_class_uid IS 'SOP Class UID';
COMMENT ON COLUMN project_data_instance.instance_number IS 'Instance Number';
COMMENT ON COLUMN project_data_instance.content_date IS '생성일';
COMMENT ON COLUMN project_data_instance.content_time IS '생성시간';
COMMENT ON COLUMN project_data_instance.is_active IS '활성 상태';
COMMENT ON COLUMN project_data_instance.sync_status IS '동기화 상태';

COMMENT ON COLUMN project_data_access.instance_id IS 'Instance 레벨 접근 제어용';
COMMENT ON COLUMN project_data_access.user_institution_id IS '사용자 소속 기관';
COMMENT ON COLUMN project_data_access.data_institution_id IS '데이터 소속 기관';
COMMENT ON COLUMN project_data_access.access_scope IS '접근 범위 (FULL/LIMITED/READ_ONLY)';
COMMENT ON COLUMN project_data_access.expires_at IS '접근 만료 시간';
COMMENT ON COLUMN project_data_access.granted_by IS '승인자';
COMMENT ON COLUMN project_data_access.granted_at IS '승인 시간';

COMMENT ON COLUMN security_access_condition.institution_id IS '기관 필터링';
COMMENT ON COLUMN security_access_condition.patient_id IS '환자 필터링';
COMMENT ON COLUMN security_access_condition.study_uid_pattern IS 'Study UID 패턴';
COMMENT ON COLUMN security_access_condition.series_uid_pattern IS 'Series UID 패턴';
COMMENT ON COLUMN security_access_condition.modality IS 'Modality 필터링';
COMMENT ON COLUMN security_access_condition.date_range_start IS '날짜 범위 시작';
COMMENT ON COLUMN security_access_condition.date_range_end IS '날짜 범위 종료';

COMMENT ON COLUMN security_role_dicom_condition.priority IS '룰 평가 우선순위 (높을수록 먼저 평가)';
COMMENT ON COLUMN security_project_dicom_condition.priority IS '룰 평가 우선순위 (높을수록 먼저 평가)';

