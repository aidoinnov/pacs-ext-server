-- Migration: Add DICOM Global Access Permission and Capability
-- Created: 2025-01-28
-- Description: DICOM 전체 데이터 조회 권한 추가

-- 1. Permission 추가
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('DICOM 데이터 관리', 'DICOM', 'READ_ALL');

-- 2. Capability 추가
INSERT INTO security_capability (name, display_name, display_label, description, category, category_label) VALUES
    ('DICOM_GLOBAL_ACCESS', 'DICOM 전체 접근', 'Global', '모든 프로젝트의 DICOM 데이터 조회 권한', 'DICOM 데이터 관리', 'DICOM');

-- 3. Capability-Permission 매핑
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'DICOM_GLOBAL_ACCESS' 
  AND p.resource_type = 'DICOM' 
  AND p.action = 'READ_ALL';

-- 4. SUPER_ADMIN Role에 Capability 할당
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'SUPER_ADMIN' 
  AND c.name = 'DICOM_GLOBAL_ACCESS';

-- 5. ADMIN Role에 Capability 할당 (선택적)
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id
FROM security_role r, security_capability c
WHERE r.name = 'ADMIN' 
  AND c.name = 'DICOM_GLOBAL_ACCESS';

