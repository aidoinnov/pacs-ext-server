-- Migration: Add updated_at column to security_project table
-- Purpose: Enable ETag-based HTTP caching for project APIs
-- Date: 2026-01-24

-- Add updated_at column to security_project
ALTER TABLE security_project
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create trigger function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_security_project_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on security_project
CREATE TRIGGER trigger_update_security_project_updated_at
BEFORE UPDATE ON security_project
FOR EACH ROW
EXECUTE FUNCTION update_security_project_updated_at();

-- Initialize updated_at with created_at for existing rows
UPDATE security_project
SET updated_at = created_at
WHERE updated_at IS NULL OR updated_at = CURRENT_TIMESTAMP;

-- Add comment
COMMENT ON COLUMN security_project.updated_at IS 'Timestamp of last update, used for ETag generation';
COMMENT ON TRIGGER trigger_update_security_project_updated_at ON security_project IS 'Automatically updates updated_at on row modification';

