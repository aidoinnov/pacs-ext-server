-- Migration: Refactor project_data to support hierarchical DICOM resources
-- Created: 2025-01-30
-- Description: 
--   1. Remove project_id from project_data_study (make it global)
--   2. Refactor project_data to reference study/series/instance hierarchically
--   3. Support STUDY, SERIES, INSTANCE level project data inclusion

-- ============================================================================
-- STEP 1: Create project_data_instance table (if not exists)
-- ============================================================================
CREATE TABLE IF NOT EXISTS project_data_instance (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_uid TEXT NOT NULL,
    sop_class_uid TEXT,
    instance_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (series_id, instance_uid)
);

CREATE INDEX IF NOT EXISTS idx_project_data_instance_series ON project_data_instance(series_id);
CREATE INDEX IF NOT EXISTS idx_project_data_instance_uid ON project_data_instance(instance_uid);

-- ============================================================================
-- STEP 2: Backup existing project_data_study data
-- ============================================================================
-- Create temporary backup table
CREATE TABLE IF NOT EXISTS _backup_project_data_study_with_project AS
SELECT * FROM project_data_study;

-- ============================================================================
-- STEP 3: Remove project_id from project_data_study
-- ============================================================================
-- Drop the unique constraint that includes project_id
ALTER TABLE project_data_study DROP CONSTRAINT IF EXISTS project_data_study_project_id_study_uid_key;
ALTER TABLE project_data_study DROP CONSTRAINT IF EXISTS uq_pds_project_studyuid;

-- Drop indexes that reference project_id
DROP INDEX IF EXISTS idx_project_data_study_project;
DROP INDEX IF EXISTS idx_pds_project_studydate;
DROP INDEX IF EXISTS idx_pds_project_patient;

-- Remove project_id column (only if it exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'project_data_study' AND column_name = 'project_id') THEN
        ALTER TABLE project_data_study DROP COLUMN project_id;
    END IF;
END $$;

-- Add new unique constraint on study_uid only (global uniqueness)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'uq_study_uid') THEN
        ALTER TABLE project_data_study ADD CONSTRAINT uq_study_uid UNIQUE (study_uid);
    END IF;
END $$;

-- ============================================================================
-- STEP 4: Migrate data to new structure
-- ============================================================================
-- Insert unique studies from backup (deduplication by study_uid)
-- Only if backup table exists and has data
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_backup_project_data_study_with_project') THEN
        INSERT INTO project_data_study (
            study_uid,
            study_description,
            patient_id,
            patient_name,
            patient_birth_date,
            study_date,
            created_at,
            updated_at
        )
        SELECT DISTINCT ON (study_uid)
            study_uid,
            study_description,
            patient_id,
            patient_name,
            patient_birth_date,
            study_date,
            created_at,
            updated_at
        FROM _backup_project_data_study_with_project
        ORDER BY study_uid, created_at
        ON CONFLICT (study_uid) DO NOTHING;
    END IF;
END $$;

-- ============================================================================
-- STEP 5: Refactor project_data table to support hierarchical resources
-- ============================================================================

-- Rename old project_data to backup (if exists from migration 010)
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'project_data') THEN
        ALTER TABLE project_data RENAME TO _backup_project_data_old;
    END IF;
END $$;

-- Create new project_data table with hierarchical resource support
CREATE TABLE IF NOT EXISTS project_data (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    
    -- Resource level: STUDY, SERIES, or INSTANCE
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    
    -- Hierarchical references (based on resource_level)
    study_id INTEGER REFERENCES project_data_study(id) ON DELETE CASCADE,
    series_id INTEGER REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_id INTEGER REFERENCES project_data_instance(id) ON DELETE CASCADE,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints:
    -- 1. At least study_id must be set
    -- 2. If resource_level = STUDY, only study_id should be set
    -- 3. If resource_level = SERIES, study_id and series_id should be set
    -- 4. If resource_level = INSTANCE, all three should be set
    CONSTRAINT chk_project_data_study_required CHECK (study_id IS NOT NULL),
    CONSTRAINT chk_project_data_study_level CHECK (
        (resource_level = 'STUDY' AND series_id IS NULL AND instance_id IS NULL) OR
        (resource_level = 'SERIES' AND series_id IS NOT NULL AND instance_id IS NULL) OR
        (resource_level = 'INSTANCE' AND series_id IS NOT NULL AND instance_id IS NOT NULL)
    ),
    
    -- Unique constraint: one entry per project + resource combination
    UNIQUE (project_id, study_id, series_id, instance_id)
);

-- ============================================================================
-- STEP 6: Migrate old project_data to new structure
-- ============================================================================
-- Migrate from backup table (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_backup_project_data_old') THEN
        INSERT INTO project_data (project_id, resource_level, study_id, created_at)
        SELECT 
            old.project_id,
            'STUDY'::resource_level_enum,
            pds.id,
            old.created_at
        FROM _backup_project_data_old old
        INNER JOIN project_data_study pds ON pds.study_uid = old.study_uid
        ON CONFLICT (project_id, study_id, series_id, instance_id) DO NOTHING;
    END IF;
END $$;

-- Migrate from backup_project_data_study_with_project
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_backup_project_data_study_with_project') THEN
        INSERT INTO project_data (project_id, resource_level, study_id, created_at)
        SELECT
            backup.project_id,
            'STUDY'::resource_level_enum,
            pds.id,
            backup.created_at
        FROM _backup_project_data_study_with_project backup
        INNER JOIN project_data_study pds ON pds.study_uid = backup.study_uid
        ON CONFLICT (project_id, study_id, series_id, instance_id) DO NOTHING;
    END IF;
END $$;

-- ============================================================================
-- STEP 7: Create indexes for performance
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_project_data_project ON project_data(project_id);
CREATE INDEX IF NOT EXISTS idx_project_data_study ON project_data(study_id) WHERE study_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_series ON project_data(series_id) WHERE series_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_instance ON project_data(instance_id) WHERE instance_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_data_level ON project_data(resource_level);
CREATE INDEX IF NOT EXISTS idx_project_data_project_level ON project_data(project_id, resource_level);

-- Study table indexes (without project_id)
CREATE INDEX IF NOT EXISTS idx_study_uid ON project_data_study(study_uid);
CREATE INDEX IF NOT EXISTS idx_study_patient ON project_data_study(patient_id);
CREATE INDEX IF NOT EXISTS idx_study_date ON project_data_study(study_date);

-- ============================================================================
-- STEP 8: Update project_data_access to reference new structure
-- ============================================================================
-- Add project_data_id reference (if not exists)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'project_data_access' AND column_name = 'project_data_id') THEN
        ALTER TABLE project_data_access ADD COLUMN project_data_id INTEGER REFERENCES project_data(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Create index
CREATE INDEX IF NOT EXISTS idx_project_data_access_project_data ON project_data_access(project_data_id) WHERE project_data_id IS NOT NULL;

-- ============================================================================
-- STEP 9: Add update trigger for project_data
-- ============================================================================
CREATE OR REPLACE FUNCTION update_project_data_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_project_data_updated_at
BEFORE UPDATE ON project_data
FOR EACH ROW
EXECUTE FUNCTION update_project_data_updated_at();

-- ============================================================================
-- STEP 10: Add comments
-- ============================================================================
COMMENT ON TABLE project_data IS '프로젝트에 포함되는 DICOM 데이터 (Study/Series/Instance 레벨 지원)';
COMMENT ON TABLE project_data_study IS '전역 DICOM Study 메타데이터 (프로젝트 독립적)';
COMMENT ON TABLE project_data_series IS 'DICOM Series 메타데이터';
COMMENT ON TABLE project_data_instance IS 'DICOM Instance 메타데이터';

COMMENT ON COLUMN project_data.resource_level IS '리소스 레벨: STUDY(전체 Study), SERIES(특정 Series), INSTANCE(특정 Instance)';
COMMENT ON COLUMN project_data.study_id IS 'Study 참조 (필수)';
COMMENT ON COLUMN project_data.series_id IS 'Series 참조 (SERIES/INSTANCE 레벨일 때)';
COMMENT ON COLUMN project_data.instance_id IS 'Instance 참조 (INSTANCE 레벨일 때)';

-- ============================================================================
-- STEP 11: Create helper view for backward compatibility
-- ============================================================================
CREATE OR REPLACE VIEW v_project_data_study_legacy AS
SELECT 
    pd.project_id,
    pds.id,
    pds.study_uid,
    pds.study_description,
    pds.patient_id,
    pds.patient_name,
    pds.patient_birth_date,
    pds.study_date,
    pds.created_at,
    pds.updated_at
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
WHERE pd.resource_level = 'STUDY';

COMMENT ON VIEW v_project_data_study_legacy IS '하위 호환성을 위한 뷰: 기존 project_data_study + project_id 구조 에뮬레이션';

