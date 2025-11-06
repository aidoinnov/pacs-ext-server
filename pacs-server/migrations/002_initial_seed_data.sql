-- Migration: Initial seed data
-- Created: 2025-10-16
-- Description: Inserts initial seed data for roles and permissions

-- ==========================
-- INITIAL ROLES
-- ==========================

INSERT INTO security_role (name, description, scope) VALUES
    ('SUPER_ADMIN', '시스템 전체 관리자', 'GLOBAL'),
    ('PROJECT_ADMIN', '프로젝트 관리자', 'PROJECT'),
    ('RESEARCHER', '연구원 (읽기/쓰기)', 'PROJECT'),
    ('VIEWER', '뷰어 (읽기 전용)', 'PROJECT'),
    ('ANNOTATOR', '어노테이터 (주석 작성)', 'PROJECT')
ON CONFLICT (name) DO NOTHING;

-- ==========================
-- INITIAL PERMISSIONS
-- ==========================

-- User management permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('사용자 및 권한 관리', 'USER', 'CREATE'),
    ('사용자 및 권한 관리', 'USER', 'READ'),
    ('사용자 및 권한 관리', 'USER', 'UPDATE'),
    ('사용자 및 권한 관리', 'USER', 'DELETE')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Project management permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('프로젝트 관리', 'PROJECT', 'CREATE'),
    ('프로젝트 관리', 'PROJECT', 'READ'),
    ('프로젝트 관리', 'PROJECT', 'UPDATE'),
    ('프로젝트 관리', 'PROJECT', 'DELETE')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Study permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('DICOM 데이터 관리', 'STUDY', 'READ'),
    ('DICOM 데이터 관리', 'STUDY', 'DOWNLOAD'),
    ('DICOM 데이터 관리', 'STUDY', 'SHARE')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Series permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('DICOM 데이터 관리', 'SERIES', 'READ'),
    ('DICOM 데이터 관리', 'SERIES', 'DOWNLOAD')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Instance permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('DICOM 데이터 관리', 'INSTANCE', 'READ'),
    ('DICOM 데이터 관리', 'INSTANCE', 'DOWNLOAD')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Annotation permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('어노테이션 관리', 'ANNOTATION', 'CREATE'),
    ('어노테이션 관리', 'ANNOTATION', 'READ'),
    ('어노테이션 관리', 'ANNOTATION', 'UPDATE'),
    ('어노테이션 관리', 'ANNOTATION', 'DELETE'),
    ('어노테이션 관리', 'ANNOTATION', 'SHARE')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Mask permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('어노테이션 관리', 'MASK', 'CREATE'),
    ('어노테이션 관리', 'MASK', 'READ'),
    ('어노테이션 관리', 'MASK', 'UPDATE'),
    ('어노테이션 관리', 'MASK', 'DELETE'),
    ('어노테이션 관리', 'MASK', 'DOWNLOAD')
ON CONFLICT (resource_type, action) DO NOTHING;

-- Hanging Protocol permissions
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('어노테이션 관리', 'HANGING_PROTOCOL', 'CREATE'),
    ('어노테이션 관리', 'HANGING_PROTOCOL', 'READ'),
    ('어노테이션 관리', 'HANGING_PROTOCOL', 'UPDATE'),
    ('어노테이션 관리', 'HANGING_PROTOCOL', 'DELETE')
ON CONFLICT (resource_type, action) DO NOTHING;

-- ==========================
-- ROLE-PERMISSION MAPPINGS
-- ==========================

-- SUPER_ADMIN: 모든 권한
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'SUPER_ADMIN'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- PROJECT_ADMIN: 프로젝트 내 모든 권한 (USER 제외)
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'PROJECT_ADMIN'
  AND p.resource_type IN ('PROJECT', 'STUDY', 'SERIES', 'INSTANCE', 'ANNOTATION', 'MASK', 'HANGING_PROTOCOL')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- RESEARCHER: 연구원 권한 (읽기, 쓰기, 어노테이션)
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'RESEARCHER'
  AND (
    (p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action IN ('READ', 'DOWNLOAD'))
    OR (p.resource_type = 'ANNOTATION' AND p.action IN ('CREATE', 'READ', 'UPDATE', 'DELETE'))
    OR (p.resource_type = 'MASK' AND p.action IN ('CREATE', 'READ', 'UPDATE', 'DELETE', 'DOWNLOAD'))
    OR (p.resource_type = 'HANGING_PROTOCOL' AND p.action IN ('CREATE', 'READ', 'UPDATE', 'DELETE'))
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ANNOTATOR: 어노테이션 작성 권한
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'ANNOTATOR'
  AND (
    (p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action = 'READ')
    OR (p.resource_type = 'ANNOTATION' AND p.action IN ('CREATE', 'READ', 'UPDATE'))
    OR (p.resource_type = 'MASK' AND p.action IN ('CREATE', 'READ', 'UPDATE', 'DOWNLOAD'))
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- VIEWER: 읽기 전용 권한
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r
CROSS JOIN security_permission p
WHERE r.name = 'VIEWER'
  AND (
    (p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE', 'ANNOTATION', 'MASK') AND p.action = 'READ')
    OR (p.resource_type = 'HANGING_PROTOCOL' AND p.action = 'READ')
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

