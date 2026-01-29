-- Migration 046: Add updated_at column to security_role_capability table
-- Purpose: Track role-capability assignment changes for ETag-based caching

-- Add updated_at column to security_role_capability
ALTER TABLE security_role_capability 
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create trigger function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_role_capability_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to call the function before UPDATE
CREATE TRIGGER trigger_update_role_capability_updated_at
    BEFORE UPDATE ON security_role_capability
    FOR EACH ROW
    EXECUTE FUNCTION update_role_capability_updated_at();

-- Initialize updated_at with created_at for existing rows
UPDATE security_role_capability SET updated_at = created_at;

