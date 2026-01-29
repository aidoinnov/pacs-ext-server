-- Add updated_at column to security_user_project table
-- This allows ETag-based caching for role assignment APIs

-- Add updated_at column
ALTER TABLE security_user_project 
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_security_user_project_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_security_user_project_updated_at
    BEFORE UPDATE ON security_user_project
    FOR EACH ROW
    EXECUTE FUNCTION update_security_user_project_updated_at();

-- Add comment
COMMENT ON COLUMN security_user_project.updated_at IS 'Last updated timestamp for role assignment (used for ETag caching)';

