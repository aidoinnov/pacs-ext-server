-- Migration: Add permission category comment
-- Created: 2025-01-20
-- Description: Add comment to clarify that resource_type serves as permission category

-- Add comment for clarity
COMMENT ON COLUMN security_permission.resource_type IS 'Permission category (USER, PROJECT, ANNOTATION, etc.)';
