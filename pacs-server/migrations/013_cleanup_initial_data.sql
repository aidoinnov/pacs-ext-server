-- Migration: Cleanup Initial Data
-- Created: 2025-01-27
-- Description: 정리된 초기 데이터로 재설정

-- 1. 기존 데이터 정리
DELETE FROM security_role_capability;
DELETE FROM security_capability_mapping;
DELETE FROM security_capability;
DELETE FROM security_role_permission;
DELETE FROM security_permission;
DELETE FROM security_role;

-- 2. Role 데이터 재생성
INSERT INTO security_role (name, description, scope) VALUES
    ('SUPER_ADMIN', '시스템 전체 관리자', 'GLOBAL'),
    ('ADMIN', '관리자', 'GLOBAL'),
    ('PROJECT_ADMIN', '프로젝트 관리자', 'PROJECT'),
    ('USER', '일반 사용자', 'GLOBAL'),
    ('VIEWER', '조회 전용 사용자', 'GLOBAL');

-- 3. Permission 데이터 재생성 (정리된 버전)
INSERT INTO security_permission (category, resource_type, action) VALUES
    -- 사용자 및 권한 관리
    ('사용자 및 권한 관리', 'USER', 'CREATE'),
    ('사용자 및 권한 관리', 'USER', 'READ'),
    ('사용자 및 권한 관리', 'USER', 'UPDATE'),
    ('사용자 및 권한 관리', 'USER', 'DELETE'),
    ('사용자 및 권한 관리', 'ROLE', 'CREATE'),
    ('사용자 및 권한 관리', 'ROLE', 'READ'),
    ('사용자 및 권한 관리', 'ROLE', 'UPDATE'),
    ('사용자 및 권한 관리', 'ROLE', 'DELETE'),
    
    -- 프로젝트 관리
    ('프로젝트 관리', 'PROJECT', 'CREATE'),
    ('프로젝트 관리', 'PROJECT', 'READ'),
    ('프로젝트 관리', 'PROJECT', 'UPDATE'),
    ('프로젝트 관리', 'PROJECT', 'DELETE'),
    ('프로젝트 관리', 'PROJECT', 'ASSIGN'),
    
    -- DICOM 데이터 관리
    ('DICOM 데이터 관리', 'STUDY', 'READ'),
    ('DICOM 데이터 관리', 'STUDY', 'DOWNLOAD'),
    ('DICOM 데이터 관리', 'STUDY', 'SHARE'),
    ('DICOM 데이터 관리', 'SERIES', 'READ'),
    ('DICOM 데이터 관리', 'SERIES', 'DOWNLOAD'),
    ('DICOM 데이터 관리', 'INSTANCE', 'READ'),
    ('DICOM 데이터 관리', 'INSTANCE', 'DOWNLOAD'),
    ('DICOM 데이터 관리', 'STUDY', 'CREATE'),
    ('DICOM 데이터 관리', 'STUDY', 'UPDATE'),
    ('DICOM 데이터 관리', 'STUDY', 'DELETE'),
    ('DICOM 데이터 관리', 'SERIES', 'CREATE'),
    ('DICOM 데이터 관리', 'SERIES', 'UPDATE'),
    ('DICOM 데이터 관리', 'SERIES', 'DELETE'),
    ('DICOM 데이터 관리', 'INSTANCE', 'CREATE'),
    ('DICOM 데이터 관리', 'INSTANCE', 'UPDATE'),
    ('DICOM 데이터 관리', 'INSTANCE', 'DELETE'),
    
    -- 어노테이션 관리
    ('어노테이션 관리', 'ANNOTATION', 'CREATE'),
    ('어노테이션 관리', 'ANNOTATION', 'READ'),
    ('어노테이션 관리', 'ANNOTATION', 'UPDATE'),
    ('어노테이션 관리', 'ANNOTATION', 'DELETE'),
    ('어노테이션 관리', 'ANNOTATION', 'SHARE'),
    
    -- 마스크 관리
    ('마스크 관리', 'MASK', 'CREATE'),
    ('마스크 관리', 'MASK', 'READ'),
    ('마스크 관리', 'MASK', 'UPDATE'),
    ('마스크 관리', 'MASK', 'DELETE'),
    ('마스크 관리', 'MASK', 'DOWNLOAD'),
    
    -- 행잉 프로토콜 관리
    ('행잉 프로토콜 관리', 'HANGING_PROTOCOL', 'CREATE'),
    ('행잉 프로토콜 관리', 'HANGING_PROTOCOL', 'READ'),
    ('행잉 프로토콜 관리', 'HANGING_PROTOCOL', 'UPDATE'),
    ('행잉 프로토콜 관리', 'HANGING_PROTOCOL', 'DELETE');

-- 4. Capability 데이터 재생성 (정리된 버전)
INSERT INTO security_capability (name, display_name, description, category) VALUES
    -- 관리 카테고리
    ('SYSTEM_ADMIN', '시스템 관리', '시스템 전체 관리 권한', '관리'),
    ('USER_MANAGEMENT', '사용자 관리', '사용자 계정 생성, 조회, 수정, 삭제 권한', '관리'),
    ('ROLE_MANAGEMENT', '역할 관리', '역할 생성, 조회, 수정, 삭제 권한', '관리'),
    ('PROJECT_MANAGEMENT', '프로젝트 관리', '프로젝트 생성, 조회, 수정, 삭제 권한', '관리'),
    
    -- 프로젝트 카테고리
    ('PROJECT_CREATE', '프로젝트 생성', '새 프로젝트 생성 권한', '프로젝트'),
    ('PROJECT_EDIT', '프로젝트 편집', '프로젝트 정보 수정 권한', '프로젝트'),
    ('PROJECT_ASSIGN', '프로젝트 할당', '프로젝트에 사용자 할당 권한', '프로젝트'),
    
    -- DICOM 데이터 카테고리
    ('DICOM_READ_ACCESS', 'DICOM 읽기 접근', 'DICOM 스터디, 시리즈, 인스턴스 조회 및 다운로드 권한', 'DICOM 데이터 관리'),
    ('DICOM_WRITE_ACCESS', 'DICOM 쓰기 접근', 'DICOM 데이터 업로드 및 수정 권한', 'DICOM 데이터 관리'),
    ('DICOM_DELETE_ACCESS', 'DICOM 삭제 접근', 'DICOM 데이터 삭제 권한', 'DICOM 데이터 관리'),
    ('DICOM_SHARE_ACCESS', 'DICOM 공유 접근', 'DICOM 데이터 공유 권한', 'DICOM 데이터 관리'),
    
    -- 어노테이션 카테고리
    ('ANNOTATION_READ_OWN', '본인 어노테이션 읽기', '자신이 작성한 어노테이션 조회 권한', '어노테이션 관리'),
    ('ANNOTATION_READ_ALL', '모든 어노테이션 읽기', '모든 사용자의 어노테이션 조회 권한', '어노테이션 관리'),
    ('ANNOTATION_WRITE', '어노테이션 작성', '어노테이션 생성 및 수정 권한', '어노테이션 관리'),
    ('ANNOTATION_DELETE', '어노테이션 삭제', '어노테이션 삭제 권한', '어노테이션 관리'),
    ('ANNOTATION_SHARE', '어노테이션 공유', '어노테이션 공유 권한', '어노테이션 관리'),
    
    -- 마스크 카테고리
    ('MASK_READ', '마스크 읽기', '마스크 조회 및 다운로드 권한', '마스크 관리'),
    ('MASK_WRITE', '마스크 작성', '마스크 생성 및 수정 권한', '마스크 관리'),
    ('MASK_DELETE', '마스크 삭제', '마스크 삭제 권한', '마스크 관리'),
    
    -- 행잉 프로토콜 카테고리
    ('HANGING_PROTOCOL_MANAGEMENT', '행잉 프로토콜 관리', '행잉 프로토콜 생성, 조회, 수정, 삭제 권한', '행잉 프로토콜 관리');

-- 5. Capability-Permission 매핑 설정
-- SYSTEM_ADMIN → 모든 권한
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'SYSTEM_ADMIN';

-- USER_MANAGEMENT → USER 관련 권한
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'USER_MANAGEMENT' AND p.resource_type = 'USER';

-- ROLE_MANAGEMENT → ROLE 관련 권한
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ROLE_MANAGEMENT' AND p.resource_type = 'ROLE';

-- PROJECT_MANAGEMENT → PROJECT 관련 권한
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_MANAGEMENT' AND p.resource_type = 'PROJECT';

-- PROJECT_CREATE → PROJECT:CREATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_CREATE' AND p.resource_type = 'PROJECT' AND p.action = 'CREATE';

-- PROJECT_EDIT → PROJECT:UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_EDIT' AND p.resource_type = 'PROJECT' AND p.action = 'UPDATE';

-- PROJECT_ASSIGN → PROJECT:ASSIGN
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_ASSIGN' AND p.resource_type = 'PROJECT' AND p.action = 'ASSIGN';

-- DICOM_READ_ACCESS → STUDY/SERIES/INSTANCE:READ, DOWNLOAD
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'DICOM_READ_ACCESS' 
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') 
  AND p.action IN ('READ', 'DOWNLOAD');

-- DICOM_WRITE_ACCESS → STUDY/SERIES/INSTANCE:CREATE, UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'DICOM_WRITE_ACCESS' 
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') 
  AND p.action IN ('CREATE', 'UPDATE');

-- DICOM_DELETE_ACCESS → STUDY/SERIES/INSTANCE:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'DICOM_DELETE_ACCESS' 
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') 
  AND p.action = 'DELETE';

-- DICOM_SHARE_ACCESS → STUDY:SHARE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'DICOM_SHARE_ACCESS' 
  AND p.resource_type = 'STUDY' 
  AND p.action = 'SHARE';

-- ANNOTATION_READ_OWN → ANNOTATION:READ
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_READ_OWN' 
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'READ';

-- ANNOTATION_READ_ALL → ANNOTATION:READ
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_READ_ALL' 
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'READ';

-- ANNOTATION_WRITE → ANNOTATION:CREATE, UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_WRITE' 
  AND p.resource_type = 'ANNOTATION' 
  AND p.action IN ('CREATE', 'UPDATE');

-- ANNOTATION_DELETE → ANNOTATION:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_DELETE' 
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'DELETE';

-- ANNOTATION_SHARE → ANNOTATION:SHARE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_SHARE' 
  AND p.resource_type = 'ANNOTATION' 
  AND p.action = 'SHARE';

-- MASK_READ → MASK:READ, DOWNLOAD
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'MASK_READ' 
  AND p.resource_type = 'MASK' 
  AND p.action IN ('READ', 'DOWNLOAD');

-- MASK_WRITE → MASK:CREATE, UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'MASK_WRITE' 
  AND p.resource_type = 'MASK' 
  AND p.action IN ('CREATE', 'UPDATE');

-- MASK_DELETE → MASK:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'MASK_DELETE' 
  AND p.resource_type = 'MASK' 
  AND p.action = 'DELETE';

-- HANGING_PROTOCOL_MANAGEMENT → HANGING_PROTOCOL:*
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'HANGING_PROTOCOL_MANAGEMENT' 
  AND p.resource_type = 'HANGING_PROTOCOL';

-- 6. Role-Capability 매핑 설정
-- SUPER_ADMIN → 모든 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'SUPER_ADMIN';

-- ADMIN → 관리 관련 Capability + 기본 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'ADMIN' 
  AND c.name IN (
    'USER_MANAGEMENT', 'ROLE_MANAGEMENT', 'PROJECT_MANAGEMENT',
    'DICOM_READ_ACCESS', 'DICOM_WRITE_ACCESS', 'DICOM_DELETE_ACCESS', 'DICOM_SHARE_ACCESS',
    'ANNOTATION_READ_ALL', 'ANNOTATION_WRITE', 'ANNOTATION_DELETE', 'ANNOTATION_SHARE',
    'MASK_READ', 'MASK_WRITE', 'MASK_DELETE',
    'HANGING_PROTOCOL_MANAGEMENT'
  );

-- PROJECT_ADMIN → 프로젝트 관련 Capability + 기본 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'PROJECT_ADMIN' 
  AND c.name IN (
    'PROJECT_MANAGEMENT', 'PROJECT_CREATE', 'PROJECT_EDIT', 'PROJECT_ASSIGN',
    'DICOM_READ_ACCESS', 'DICOM_WRITE_ACCESS', 'DICOM_SHARE_ACCESS',
    'ANNOTATION_READ_ALL', 'ANNOTATION_WRITE', 'ANNOTATION_DELETE', 'ANNOTATION_SHARE',
    'MASK_READ', 'MASK_WRITE', 'MASK_DELETE'
  );

-- USER → 기본 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'USER' 
  AND c.name IN (
    'DICOM_READ_ACCESS',
    'ANNOTATION_READ_OWN', 'ANNOTATION_WRITE', 'ANNOTATION_DELETE',
    'MASK_READ', 'MASK_WRITE', 'MASK_DELETE'
  );

-- VIEWER → 읽기 전용 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'VIEWER' 
  AND c.name IN (
    'DICOM_READ_ACCESS',
    'ANNOTATION_READ_OWN', 'ANNOTATION_READ_ALL',
    'MASK_READ'
  );

-- 7. 기존 role_permission 테이블 정리 (하위 호환성을 위해 유지)
-- SUPER_ADMIN → 모든 Permission
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r, security_permission p
WHERE r.name = 'SUPER_ADMIN';

-- ADMIN → 관리 관련 Permission + 기본 Permission
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r, security_permission p
WHERE r.name = 'ADMIN' 
  AND (p.resource_type IN ('USER', 'ROLE', 'PROJECT') 
       OR (p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action IN ('READ', 'DOWNLOAD', 'SHARE'))
       OR (p.resource_type = 'ANNOTATION' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE', 'SHARE'))
       OR (p.resource_type = 'MASK' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE', 'DOWNLOAD'))
       OR p.resource_type = 'HANGING_PROTOCOL');

-- PROJECT_ADMIN → 프로젝트 관련 Permission + 기본 Permission
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r, security_permission p
WHERE r.name = 'PROJECT_ADMIN' 
  AND (p.resource_type = 'PROJECT'
       OR (p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action IN ('READ', 'DOWNLOAD', 'SHARE'))
       OR (p.resource_type = 'ANNOTATION' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE', 'SHARE'))
       OR (p.resource_type = 'MASK' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE', 'DOWNLOAD')));

-- USER → 기본 Permission
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r, security_permission p
WHERE r.name = 'USER' 
  AND ((p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action = 'READ')
       OR (p.resource_type = 'ANNOTATION' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE'))
       OR (p.resource_type = 'MASK' AND p.action IN ('READ', 'CREATE', 'UPDATE', 'DELETE')));

-- VIEWER → 읽기 전용 Permission
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id
FROM security_role r, security_permission p
WHERE r.name = 'VIEWER' 
  AND ((p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action = 'READ')
       OR (p.resource_type = 'ANNOTATION' AND p.action = 'READ')
       OR (p.resource_type = 'MASK' AND p.action = 'READ'));

-- 8. 통계 확인
SELECT 'Roles' as table_name, COUNT(*) as count FROM security_role
UNION ALL
SELECT 'Permissions', COUNT(*) FROM security_permission
UNION ALL
SELECT 'Capabilities', COUNT(*) FROM security_capability
UNION ALL
SELECT 'Capability-Permission Mappings', COUNT(*) FROM security_capability_mapping
UNION ALL
SELECT 'Role-Capability Mappings', COUNT(*) FROM security_role_capability
UNION ALL
SELECT 'Role-Permission Mappings', COUNT(*) FROM security_role_permission;
