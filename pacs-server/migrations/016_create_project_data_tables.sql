-- Migration: Create project data tables (Study, Series, Access)
-- Created: 2025-01-27
-- Description: Creates tables for DICOM hierarchical data (Study → Series) and access control
-- This supports fine-grained access control at STUDY, SERIES, and INSTANCE levels

-- Create data_access_status_enum if not exists (resource_level_enum is already created in 001_initial_schema.sql)
DO $$ BEGIN
    CREATE TYPE data_access_status_enum AS ENUM ('APPROVED', 'DENIED', 'PENDING');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- 프로젝트 데이터 (Study 레벨)
CREATE TABLE IF NOT EXISTS project_data_study (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    study_uid TEXT NOT NULL,
    study_description TEXT,
    patient_id TEXT,
    patient_name TEXT,
    patient_birth_date DATE,
    study_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_id, study_uid)
);

-- 프로젝트 데이터 (Series 레벨)
CREATE TABLE IF NOT EXISTS project_data_series (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    study_id INTEGER NOT NULL REFERENCES project_data_study(id) ON DELETE CASCADE,
    series_uid TEXT NOT NULL,
    series_description TEXT,
    modality TEXT,
    series_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (study_id, series_uid)
);

-- Modify existing project_data_access table to support hierarchical access control
-- Add new columns for resource_level, study_id, series_id
DO $$ 
BEGIN
    -- Add resource_level column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'project_data_access' AND column_name = 'resource_level') THEN
        ALTER TABLE project_data_access ADD COLUMN resource_level resource_level_enum DEFAULT 'STUDY';
    END IF;

    -- Add study_id column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'project_data_access' AND column_name = 'study_id') THEN
        ALTER TABLE project_data_access ADD COLUMN study_id INTEGER REFERENCES project_data_study(id) ON DELETE CASCADE;
    END IF;

    -- Add series_id column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'project_data_access' AND column_name = 'series_id') THEN
        ALTER TABLE project_data_access ADD COLUMN series_id INTEGER REFERENCES project_data_series(id) ON DELETE CASCADE;
    END IF;

    -- Add project_id column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'project_data_access' AND column_name = 'project_id') THEN
        ALTER TABLE project_data_access ADD COLUMN project_id INTEGER REFERENCES security_project(id) ON DELETE CASCADE;
        
        -- Update existing rows to set project_id based on project_data_id
        -- This requires a migration script to be run separately as it depends on project_data table structure
    END IF;

    -- Change status default to PENDING
    ALTER TABLE project_data_access ALTER COLUMN status SET DEFAULT 'PENDING'::data_access_status_enum;
END $$;

-- 인덱스: Study 테이블
CREATE INDEX IF NOT EXISTS idx_project_data_study_project ON project_data_study(project_id);
CREATE INDEX IF NOT EXISTS idx_project_data_study_uid ON project_data_study(study_uid);
CREATE INDEX IF NOT EXISTS idx_project_data_study_patient ON project_data_study(patient_id);
CREATE INDEX IF NOT EXISTS idx_project_data_study_date ON project_data_study(study_date);

-- 인덱스: Series 테이블
CREATE INDEX IF NOT EXISTS idx_project_data_series_study ON project_data_series(study_id);
CREATE INDEX IF NOT EXISTS idx_project_data_series_uid ON project_data_series(series_uid);
CREATE INDEX IF NOT EXISTS idx_project_data_series_modality ON project_data_series(modality);

-- 인덱스: Access 테이블 (new columns)
CREATE INDEX IF NOT EXISTS idx_project_data_access_project ON project_data_access(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_access_study ON project_data_access(study_id) WHERE study_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_access_series ON project_data_access(series_id) WHERE series_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_access_resource ON project_data_access(resource_level, study_id, series_id) WHERE resource_level IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_access_project_user ON project_data_access(project_id, user_id) WHERE project_id IS NOT NULL AND user_id IS NOT NULL;

-- 코멘트 추가
COMMENT ON TABLE project_data_study IS '프로젝트별 DICOM Study 데이터';
COMMENT ON TABLE project_data_series IS '프로젝트별 DICOM Series 데이터';
COMMENT ON TABLE project_data_access IS '사용자별 DICOM 데이터 접근 권한 관리';
COMMENT ON COLUMN project_data_access.resource_level IS '접근 권한 레벨: STUDY, SERIES, INSTANCE';
COMMENT ON COLUMN project_data_access.status IS '접근 상태: APPROVED(승인), DENIED(거부), PENDING(대기)';
