-- Migration: Add user profile fields
-- Created: 2025-01-23
-- Description: Adds additional user profile fields to security_user table

-- Add user profile fields to security_user table
ALTER TABLE security_user
ADD COLUMN full_name TEXT,
ADD COLUMN organization TEXT,
ADD COLUMN department TEXT,
ADD COLUMN phone TEXT,
ADD COLUMN updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Create index for name search
CREATE INDEX idx_user_full_name ON security_user(full_name);
CREATE INDEX idx_user_organization ON security_user(organization);

-- Add trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_user_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_user_updated_at
BEFORE UPDATE ON security_user
FOR EACH ROW
EXECUTE FUNCTION update_user_updated_at();

-- Add column comments
COMMENT ON COLUMN security_user.full_name IS '사용자 실명';
COMMENT ON COLUMN security_user.organization IS '소속 기관';
COMMENT ON COLUMN security_user.department IS '소속 부서/그룹';
COMMENT ON COLUMN security_user.phone IS '연락처';
COMMENT ON COLUMN security_user.updated_at IS '마지막 업데이트 시각';
