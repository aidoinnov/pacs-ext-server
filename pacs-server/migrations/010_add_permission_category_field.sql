-- Migration: Add category field to security_permission
-- Created: 2025-10-25
-- Description: Add separate category field for better permission organization

-- Add category column
ALTER TABLE security_permission 
ADD COLUMN category TEXT;

-- Set default categories based on existing resource_type
UPDATE security_permission 
SET category = CASE 
    WHEN resource_type IN ('USER', 'ROLE', 'PERMISSION') THEN '사용자 및 권한 관리'
    WHEN resource_type IN ('PROJECT', 'PROJECT_DATA') THEN '프로젝트 관리'
    WHEN resource_type IN ('ANNOTATION', 'MASK', 'MASK_GROUP') THEN '어노테이션 관리'
    WHEN resource_type IN ('STUDY', 'SERIES', 'INSTANCE') THEN 'DICOM 데이터 관리'
    ELSE '기타'
END;

-- Make category NOT NULL after setting defaults
ALTER TABLE security_permission 
ALTER COLUMN category SET NOT NULL;

-- Add comment
COMMENT ON COLUMN security_permission.category IS 'Permission category for UI grouping (사용자 및 권한 관리, 프로젝트 관리, etc.)';
