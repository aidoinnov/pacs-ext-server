-- Migration: Extend Project Fields
-- Created: 2025-01-27
-- Description: Adds sponsor, start_date, end_date, auto_complete fields to project table and extends project_status enum

-- ==========================
-- 1. Extend Project Status Enum
-- ==========================

-- Add new status values to project_status enum
ALTER TYPE project_status ADD VALUE IF NOT EXISTS 'PENDING_COMPLETION';
ALTER TYPE project_status ADD VALUE IF NOT EXISTS 'OVER_PLANNING';

-- ==========================
-- 2. Add New Columns to Project Table
-- ==========================

-- Add sponsor column
ALTER TABLE security_project
ADD COLUMN IF NOT EXISTS sponsor TEXT NOT NULL DEFAULT '미정';

-- Add start_date column
ALTER TABLE security_project
ADD COLUMN IF NOT EXISTS start_date DATE NOT NULL DEFAULT CURRENT_DATE;

-- Add end_date column (nullable)
ALTER TABLE security_project
ADD COLUMN IF NOT EXISTS end_date DATE;

-- Add auto_complete column
ALTER TABLE security_project
ADD COLUMN IF NOT EXISTS auto_complete BOOLEAN NOT NULL DEFAULT false;

-- ==========================
-- 3. Add Indexes for Performance
-- ==========================

CREATE INDEX IF NOT EXISTS idx_project_start_date ON security_project(start_date);
CREATE INDEX IF NOT EXISTS idx_project_end_date ON security_project(end_date);

-- ==========================
-- 4. Add Comments
-- ==========================

COMMENT ON COLUMN security_project.sponsor IS 'Project sponsor name';
COMMENT ON COLUMN security_project.start_date IS 'Project start date';
COMMENT ON COLUMN security_project.end_date IS 'Project end date or target date';
COMMENT ON COLUMN security_project.auto_complete IS 'Auto-complete project when end_date is reached';

-- ==========================
-- 5. Migrate Existing Data
-- ==========================

-- Update existing projects with default values
UPDATE security_project
SET 
  sponsor = COALESCE(sponsor, '미정'),
  start_date = COALESCE(start_date, created_at::date),
  auto_complete = COALESCE(auto_complete, false)
WHERE sponsor IS NULL OR start_date IS NULL OR auto_complete IS NULL;

-- ==========================
-- Migration Complete
-- ==========================

