-- Migration 045: Add updated_at column to security_role table
-- Purpose: Track role changes for ETag-based caching in Role-Capability matrix API

-- Add updated_at column to security_role
ALTER TABLE security_role 
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create trigger function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_security_role_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to call the function before UPDATE
CREATE TRIGGER trigger_update_security_role_updated_at
    BEFORE UPDATE ON security_role
    FOR EACH ROW
    EXECUTE FUNCTION update_security_role_updated_at();

-- Initialize updated_at with created_at for existing rows
UPDATE security_role SET updated_at = created_at;

