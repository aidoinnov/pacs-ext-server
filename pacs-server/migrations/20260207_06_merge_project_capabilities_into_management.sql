-- Project 관련 capability를 PROJECT_MANAGEMENT 하나로 통합
-- PROJECT_CREATE, PROJECT_EDIT, PROJECT_ASSIGN, PROJECT_DATA_ASSIGN, PROJECT_DELETE 제거

-- 1. PROJECT_MANAGEMENT에 project_data.assign 매핑 추가
--    (PROJECT:* 는 이미 015에서 매핑됨, project_data.assign만 추가)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_MANAGEMENT'
  AND p.resource_type = 'project_data' AND p.action = 'assign'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 2. PROJECT_MANAGEMENT 설명 확장
UPDATE security_capability
SET description = '프로젝트 생성, 조회, 수정, 삭제, 할당, 스터디 매핑 권한'
WHERE name = 'PROJECT_MANAGEMENT';

-- 3. 역할 매핑 제거 (제거될 capability들)
DELETE FROM security_role_capability
WHERE capability_id IN (
    SELECT id FROM security_capability
    WHERE name IN ('PROJECT_CREATE', 'PROJECT_EDIT', 'PROJECT_ASSIGN', 'PROJECT_DATA_ASSIGN', 'PROJECT_DELETE')
);

-- 4. capability-permission 매핑 제거
DELETE FROM security_capability_mapping
WHERE capability_id IN (
    SELECT id FROM security_capability
    WHERE name IN ('PROJECT_CREATE', 'PROJECT_EDIT', 'PROJECT_ASSIGN', 'PROJECT_DATA_ASSIGN', 'PROJECT_DELETE')
);

-- 5. capability 레코드 제거
DELETE FROM security_capability
WHERE name IN ('PROJECT_CREATE', 'PROJECT_EDIT', 'PROJECT_ASSIGN', 'PROJECT_DATA_ASSIGN', 'PROJECT_DELETE');

-- 6. PROJECT_MANAGEMENT 역할 할당 (기존에 없을 수 있는 역할 보정)
--    SUPER_ADMIN, PROJECT_ADMIN, ADMIN
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN', 'ADMIN') AND c.name = 'PROJECT_MANAGEMENT'
ON CONFLICT (role_id, capability_id) DO NOTHING;
