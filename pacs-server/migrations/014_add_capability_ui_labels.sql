-- Migration: Add UI Label Fields to Capability Table
-- Created: 2025-10-25
-- Description: Add display_label and category_label fields for UI display

-- Capability 테이블에 UI 레이블 필드 추가
ALTER TABLE security_capability 
ADD COLUMN display_label VARCHAR(50) NOT NULL DEFAULT '',
ADD COLUMN category_label VARCHAR(50) NOT NULL DEFAULT '';

COMMENT ON COLUMN security_capability.display_label IS 'UI 표시용 짧은 레이블 (예: Admin, User)';
COMMENT ON COLUMN security_capability.category_label IS 'UI 카테고리 짧은 레이블 (예: MANAGE, PROJECT)';

-- 기존 데이터에 레이블 추가
-- MANAGE 카테고리
UPDATE security_capability SET display_label = 'Admin', category_label = 'MANAGE' WHERE name = 'SYSTEM_ADMIN';
UPDATE security_capability SET display_label = 'Users', category_label = 'MANAGE' WHERE name = 'USER_MANAGEMENT';
UPDATE security_capability SET display_label = 'Roles', category_label = 'MANAGE' WHERE name = 'ROLE_MANAGEMENT';
UPDATE security_capability SET display_label = 'Projects', category_label = 'MANAGE' WHERE name = 'PROJECT_MANAGEMENT';

-- PROJECT 카테고리
UPDATE security_capability SET display_label = 'CREATE', category_label = 'PROJECT' WHERE name = 'PROJECT_CREATE';
UPDATE security_capability SET display_label = 'ASSIGN', category_label = 'PROJECT' WHERE name = 'PROJECT_ASSIGN';
UPDATE security_capability SET display_label = 'EDIT', category_label = 'PROJECT' WHERE name = 'PROJECT_EDIT';

-- DICOM 카테고리
UPDATE security_capability SET display_label = 'READ', category_label = 'DICOM' WHERE name = 'DICOM_READ_ACCESS';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'DICOM' WHERE name = 'DICOM_WRITE_ACCESS';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'DICOM' WHERE name = 'DICOM_DELETE_ACCESS';
UPDATE security_capability SET display_label = 'SHARE', category_label = 'DICOM' WHERE name = 'DICOM_SHARE_ACCESS';

-- ANNOTATION 카테고리
UPDATE security_capability SET display_label = 'READ OWN', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_READ_OWN';
UPDATE security_capability SET display_label = 'READ ALL', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_READ_ALL';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_WRITE';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_DELETE';
UPDATE security_capability SET display_label = 'SHARE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_SHARE';

-- MASK 카테고리
UPDATE security_capability SET display_label = 'READ', category_label = 'MASK' WHERE name = 'MASK_READ';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'MASK' WHERE name = 'MASK_WRITE';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'MASK' WHERE name = 'MASK_DELETE';

-- HANGING_PROTOCOL 카테고리
UPDATE security_capability SET display_label = 'MANAGE', category_label = 'HANGING_PROTOCOL' WHERE name = 'HANGING_PROTOCOL_MANAGEMENT';

-- 인덱스 추가 (검색 성능 향상)
CREATE INDEX idx_capability_category_label ON security_capability(category_label);
