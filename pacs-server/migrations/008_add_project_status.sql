-- Migration: Add project status enum and column
-- Created: 2025-01-27
-- Description: Adds project status enum and status column to security_project table
-- This enables more granular project lifecycle management

-- Create project status enum
CREATE TYPE project_status AS ENUM (
    'PREPARING',    -- 준비중
    'IN_PROGRESS',  -- 진행중
    'COMPLETED',    -- 완료
    'ON_HOLD',      -- 보류
    'CANCELLED'     -- 취소
);

-- Add status column to security_project
ALTER TABLE security_project 
ADD COLUMN status project_status NOT NULL DEFAULT 'PREPARING';

-- Migrate existing data based on is_active
UPDATE security_project 
SET status = CASE 
    WHEN is_active = true THEN 'IN_PROGRESS'::project_status
    ELSE 'ON_HOLD'::project_status
END;

-- Create index for status filtering
CREATE INDEX idx_project_status ON security_project(status);

-- Add comments
COMMENT ON COLUMN security_project.status IS 'Project status: PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED';
COMMENT ON TYPE project_status IS 'Project lifecycle status';

-- Note: Keep is_active for backward compatibility, or consider deprecating it
