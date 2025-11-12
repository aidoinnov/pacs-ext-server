-- Migration: Create project_data_access table
-- Created: 2025-11-12
-- Description: Creates project_data_access table for managing user access to project data
-- This table was missing and causing "relation does not exist" errors

-- Create data_access_status_enum if not exists
DO $$ BEGIN
    CREATE TYPE data_access_status_enum AS ENUM ('APPROVED', 'DENIED', 'PENDING');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Create project_data_access table
CREATE TABLE IF NOT EXISTS project_data_access (
    -- Primary key
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    
    -- User reference
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    
    -- Project reference
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    
    -- Hierarchical access control (Study → Series → Instance)
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    study_id INTEGER REFERENCES project_data_study(id) ON DELETE CASCADE,
    series_id INTEGER REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_id INTEGER REFERENCES project_data_instance(id) ON DELETE CASCADE,
    
    -- Legacy field for backward compatibility
    project_data_id INTEGER,
    
    -- Access status
    status data_access_status_enum NOT NULL DEFAULT 'PENDING',
    
    -- Request information
    requested_at TIMESTAMPTZ,
    requested_by INTEGER REFERENCES security_user(id),
    
    -- Review information
    reviewed_at TIMESTAMPTZ,
    reviewed_by INTEGER REFERENCES security_user(id),
    review_note TEXT,
    
    -- Grant information
    granted_by INTEGER REFERENCES security_user(id),
    granted_at TIMESTAMPTZ,
    
    -- Institution-based access control
    user_institution_id INTEGER REFERENCES security_institution(id),
    data_institution_id INTEGER REFERENCES project_data_institution(id),
    
    -- Access scope
    access_scope VARCHAR(50) DEFAULT 'FULL', -- FULL, LIMITED, READ_ONLY
    
    -- Expiration
    expires_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint: one access record per user per resource
    -- NULL values are not considered equal in UNIQUE constraints (PostgreSQL behavior)
    UNIQUE (project_id, user_id, study_id, series_id, instance_id)
);

-- Indexes for performance optimization

-- Single column indexes
CREATE INDEX IF NOT EXISTS idx_project_data_access_project 
    ON project_data_access(project_id);

CREATE INDEX IF NOT EXISTS idx_project_data_access_user 
    ON project_data_access(user_id);

CREATE INDEX IF NOT EXISTS idx_project_data_access_status 
    ON project_data_access(status);

-- Partial indexes (only index non-NULL values)
CREATE INDEX IF NOT EXISTS idx_project_data_access_study 
    ON project_data_access(study_id) 
    WHERE study_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_project_data_access_series 
    ON project_data_access(series_id) 
    WHERE series_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_project_data_access_instance 
    ON project_data_access(instance_id) 
    WHERE instance_id IS NOT NULL;

-- Composite indexes
CREATE INDEX IF NOT EXISTS idx_project_data_access_project_user 
    ON project_data_access(project_id, user_id);

CREATE INDEX IF NOT EXISTS idx_project_data_access_resource 
    ON project_data_access(resource_level, study_id, series_id, instance_id) 
    WHERE resource_level IS NOT NULL;

-- Index for expiration queries
CREATE INDEX IF NOT EXISTS idx_project_data_access_expires 
    ON project_data_access(expires_at) 
    WHERE expires_at IS NOT NULL;

-- Comments
COMMENT ON TABLE project_data_access IS '프로젝트 데이터에 대한 사용자별 접근 권한 관리';
COMMENT ON COLUMN project_data_access.resource_level IS '접근 권한 레벨: STUDY, SERIES, INSTANCE';
COMMENT ON COLUMN project_data_access.status IS '접근 상태: APPROVED(승인), DENIED(거부), PENDING(대기)';
COMMENT ON COLUMN project_data_access.access_scope IS '접근 범위: FULL(전체), LIMITED(제한), READ_ONLY(읽기전용)';
COMMENT ON COLUMN project_data_access.project_data_id IS '하위 호환성을 위한 레거시 필드';
COMMENT ON COLUMN project_data_access.study_id IS 'Study 레벨 접근 제어 (NULL 허용)';
COMMENT ON COLUMN project_data_access.series_id IS 'Series 레벨 접근 제어 (NULL 허용)';
COMMENT ON COLUMN project_data_access.instance_id IS 'Instance 레벨 접근 제어 (NULL 허용)';

-- Trigger for updated_at
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

