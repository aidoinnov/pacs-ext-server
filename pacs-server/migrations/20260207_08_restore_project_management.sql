-- PROJECT_MANAGEMENT capability 복원 (통합 후 누락 시)
-- 프로젝트 관련 모든 권한을 하나의 capability로

-- 1. PROJECT_MANAGEMENT capability (없을 때만)
INSERT INTO security_capability (name, display_name, display_label, description, category, category_label)
SELECT 'PROJECT_MANAGEMENT', '프로젝트 관리', 'Projects', '프로젝트 생성, 조회, 수정, 삭제, 할당, 스터디 매핑 권한', '관리', 'MANAGE'
WHERE NOT EXISTS (SELECT 1 FROM security_capability WHERE name = 'PROJECT_MANAGEMENT');

-- 2. PROJECT_MANAGEMENT → PROJECT:* 매핑
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_MANAGEMENT' AND p.resource_type = 'PROJECT'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 3. PROJECT_MANAGEMENT → project_data.assign 매핑
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_MANAGEMENT' AND p.resource_type = 'project_data' AND p.action = 'assign'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 4. 역할 할당: SUPER_ADMIN, PROJECT_ADMIN, ADMIN
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN', 'ADMIN') AND c.name = 'PROJECT_MANAGEMENT'
ON CONFLICT (role_id, capability_id) DO NOTHING;
