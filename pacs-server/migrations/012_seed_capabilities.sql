-- Migration: Seed Capability Data
-- Created: 2025-10-25
-- Description: Define capabilities and their mappings to permissions

-- Capability 정의
INSERT INTO security_capability (name, display_name, description, category) VALUES
    ('MANAGE_ADMIN', '관리자 권한', '시스템 전체 관리', '관리'),
    ('MANAGE_USERS', '사용자 관리', '사용자 생성, 수정, 삭제', '관리'),
    ('MANAGE_PROJECTS', '프로젝트 관리', '프로젝트 생성, 수정, 삭제', '관리'),
    ('MANAGE_ROLES', '역할 관리', '역할 및 권한 관리', '관리'),
    
    ('PROJECT_CREATE', '프로젝트 생성', '새 프로젝트 생성', '프로젝트'),
    ('PROJECT_ASSIGN', '프로젝트 할당', '프로젝트에 사용자 할당', '프로젝트'),
    ('PROJECT_EDIT', '프로젝트 편집', '프로젝트 정보 수정', '프로젝트'),
    
    ('DICOM_READ', 'DICOM 읽기', 'DICOM 데이터 조회', '데이터'),
    ('DICOM_WRITE', 'DICOM 쓰기', 'DICOM 데이터 업로드', '데이터'),
    ('DICOM_DELETE', 'DICOM 삭제', 'DICOM 데이터 삭제', '데이터'),
    
    ('ANNOTATION_READ_OWN', '본인 어노테이션 읽기', '자신이 작성한 어노테이션 조회', '어노테이션'),
    ('ANNOTATION_READ_ALL', '모든 어노테이션 읽기', '모든 사용자의 어노테이션 조회', '어노테이션'),
    ('ANNOTATION_WRITE', '어노테이션 작성', '어노테이션 생성 및 수정', '어노테이션'),
    ('ANNOTATION_DELETE', '어노테이션 삭제', '어노테이션 삭제', '어노테이션');

-- Capability-Permission 매핑
-- MANAGE_USERS → USER:*, ROLE:READ
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'MANAGE_USERS'
  AND (p.resource_type = 'USER' OR (p.resource_type = 'ROLE' AND p.action = 'READ'));

-- MANAGE_ROLES → ROLE:*
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'MANAGE_ROLES'
  AND p.resource_type = 'ROLE';

-- MANAGE_PROJECTS → PROJECT:*
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'MANAGE_PROJECTS'
  AND p.resource_type = 'PROJECT';

-- PROJECT_CREATE → PROJECT:CREATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'PROJECT_CREATE'
  AND p.resource_type = 'PROJECT' AND p.action = 'CREATE';

-- PROJECT_EDIT → PROJECT:UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'PROJECT_EDIT'
  AND p.resource_type = 'PROJECT' AND p.action = 'UPDATE';

-- PROJECT_ASSIGN → PROJECT:UPDATE (할당도 수정의 일종)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'PROJECT_ASSIGN'
  AND p.resource_type = 'PROJECT' AND p.action = 'UPDATE';

-- DICOM_READ → STUDY:READ, SERIES:READ, INSTANCE:READ
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'DICOM_READ'
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action = 'READ';

-- DICOM_WRITE → STUDY:*, SERIES:*, INSTANCE:* (CREATE, UPDATE)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'DICOM_WRITE'
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') 
  AND p.action IN ('CREATE', 'UPDATE');

-- DICOM_DELETE → STUDY:DELETE, SERIES:DELETE, INSTANCE:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'DICOM_DELETE'
  AND p.resource_type IN ('STUDY', 'SERIES', 'INSTANCE') AND p.action = 'DELETE';

-- ANNOTATION_READ_OWN → ANNOTATION:READ (자신의 것만)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'ANNOTATION_READ_OWN'
  AND p.resource_type = 'ANNOTATION' AND p.action = 'READ';

-- ANNOTATION_READ_ALL → ANNOTATION:READ (모든 것)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'ANNOTATION_READ_ALL'
  AND p.resource_type = 'ANNOTATION' AND p.action = 'READ';

-- ANNOTATION_WRITE → ANNOTATION:CREATE, ANNOTATION:UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'ANNOTATION_WRITE'
  AND p.resource_type = 'ANNOTATION' AND p.action IN ('CREATE', 'UPDATE');

-- ANNOTATION_DELETE → ANNOTATION:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c
CROSS JOIN security_permission p
WHERE c.name = 'ANNOTATION_DELETE'
  AND p.resource_type = 'ANNOTATION' AND p.action = 'DELETE';

-- MANAGE_ADMIN → 모든 Capability (특별 처리)
-- 이는 Role-Capability 매핑에서 처리

-- Role-Capability 매핑
-- SUPER_ADMIN → 모든 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r
CROSS JOIN security_capability c
WHERE r.name = 'SUPER_ADMIN';

-- PROJECT_ADMIN → 프로젝트 관련 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r
CROSS JOIN security_capability c
WHERE r.name = 'PROJECT_ADMIN'
  AND c.category IN ('프로젝트', '데이터', '어노테이션');

-- RESEARCHER → 읽기 및 어노테이션 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r
CROSS JOIN security_capability c
WHERE r.name = 'RESEARCHER'
  AND c.name IN ('DICOM_READ', 'ANNOTATION_READ_OWN', 'ANNOTATION_WRITE', 'ANNOTATION_DELETE');

-- ANNOTATOR → 어노테이션 Capability
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r
CROSS JOIN security_capability c
WHERE r.name = 'ANNOTATOR'
  AND c.name IN ('ANNOTATION_READ_OWN', 'ANNOTATION_WRITE', 'ANNOTATION_DELETE');
