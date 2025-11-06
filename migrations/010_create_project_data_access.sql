-- Migration: Project Data Access Management
-- Created: 2024-12-19
-- Description: Creates tables for managing project data (Study) access permissions

-- DICOM Study 데이터 접근 권한 상태
CREATE TYPE data_access_status_enum AS ENUM ('APPROVED', 'DENIED', 'PENDING');

-- 프로젝트 데이터(Study) 테이블
CREATE TABLE project_data (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    study_uid TEXT NOT NULL,
    study_description TEXT,
    patient_id TEXT,
    patient_name TEXT,
    study_date DATE,
    modality TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_id, study_uid)
);

-- 프로젝트 데이터 접근 권한 테이블
CREATE TABLE project_data_access (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_data_id INTEGER NOT NULL REFERENCES project_data(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    status data_access_status_enum NOT NULL DEFAULT 'APPROVED',
    requested_at TIMESTAMPTZ,
    requested_by INTEGER REFERENCES security_user(id),
    reviewed_at TIMESTAMPTZ,
    reviewed_by INTEGER REFERENCES security_user(id),
    review_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_data_id, user_id)
);

-- 인덱스
CREATE INDEX idx_project_data_project_id ON project_data(project_id);
CREATE INDEX idx_project_data_study_uid ON project_data(study_uid);
CREATE INDEX idx_project_data_access_user_id ON project_data_access(user_id);
CREATE INDEX idx_project_data_access_status ON project_data_access(status);
CREATE INDEX idx_project_data_access_project_data_id ON project_data_access(project_data_id);

-- 업데이트 트리거
CREATE OR REPLACE FUNCTION update_project_data_access_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_project_data_access_updated_at
BEFORE UPDATE ON project_data_access
FOR EACH ROW
EXECUTE FUNCTION update_project_data_access_updated_at();

-- 테이블 주석
COMMENT ON TABLE project_data IS '프로젝트별 DICOM Study 데이터 메타데이터';
COMMENT ON TABLE project_data_access IS '프로젝트 데이터 접근 권한 관리';
COMMENT ON COLUMN project_data.study_uid IS 'DICOM Study UID';
COMMENT ON COLUMN project_data.study_description IS 'Study 설명';
COMMENT ON COLUMN project_data.patient_id IS '환자 ID';
COMMENT ON COLUMN project_data.patient_name IS '환자 이름';
COMMENT ON COLUMN project_data.study_date IS 'Study 날짜';
COMMENT ON COLUMN project_data.modality IS '영상 모달리티';
COMMENT ON COLUMN project_data_access.status IS '접근 권한 상태 (APPROVED/DENIED/PENDING)';
COMMENT ON COLUMN project_data_access.requested_at IS '접근 요청 시각';
COMMENT ON COLUMN project_data_access.requested_by IS '접근 요청자 ID';
COMMENT ON COLUMN project_data_access.reviewed_at IS '승인/거부 시각';
COMMENT ON COLUMN project_data_access.reviewed_by IS '승인/거부자 ID';
COMMENT ON COLUMN project_data_access.review_note IS '승인/거부 사유';
