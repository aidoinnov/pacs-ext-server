-- PROJECT_DATA_ASSIGN: settings.study_assignment → project_data.assign 로 이전
-- PROJECT_DELETE: 프로젝트 삭제 capability 추가

-- 1. Permission: project_data.assign
INSERT INTO security_permission (category, resource_type, action) VALUES
    ('프로젝트 데이터', 'project_data', 'assign')
ON CONFLICT (resource_type, action) DO NOTHING;

-- 2. Capability: PROJECT_DATA_ASSIGN (스터디-프로젝트 매핑 관리)
INSERT INTO security_capability (name, display_name, display_label, description, category, category_label)
SELECT 'PROJECT_DATA_ASSIGN', '스터디 할당', '스터디할당', '스터디-프로젝트 매핑 관리', '프로젝트 데이터', 'PROJECT_DATA'
WHERE NOT EXISTS (SELECT 1 FROM security_capability WHERE name = 'PROJECT_DATA_ASSIGN');

-- 3. Capability-Permission: PROJECT_DATA_ASSIGN → project_data.assign
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_DATA_ASSIGN' AND p.resource_type = 'project_data' AND p.action = 'assign'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 4. Capability: PROJECT_DELETE
INSERT INTO security_capability (name, display_name, display_label, description, category, category_label)
SELECT 'PROJECT_DELETE', '프로젝트 삭제', 'DELETE', '프로젝트 삭제 권한', '프로젝트', 'PROJECT'
WHERE NOT EXISTS (SELECT 1 FROM security_capability WHERE name = 'PROJECT_DELETE');

-- 5. Capability-Permission: PROJECT_DELETE → PROJECT:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id FROM security_capability c, security_permission p
WHERE c.name = 'PROJECT_DELETE' AND p.resource_type = 'PROJECT' AND p.action = 'DELETE'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 6. 역할 → PROJECT_DATA_ASSIGN (SUPER_ADMIN, PROJECT_ADMIN)
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN') AND c.name = 'PROJECT_DATA_ASSIGN'
ON CONFLICT (role_id, capability_id) DO NOTHING;

-- 7. 역할 → project_data.assign permission (get_my_permission_codes용)
INSERT INTO security_role_permission (role_id, permission_id)
SELECT r.id, p.id FROM security_role r, security_permission p
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN') AND p.resource_type = 'project_data' AND p.action = 'assign'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 8. 역할 → PROJECT_DELETE (SUPER_ADMIN, PROJECT_ADMIN, ADMIN)
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN') AND r.scope IN ('GLOBAL', 'PROJECT') AND c.name = 'PROJECT_DELETE'
ON CONFLICT (role_id, capability_id) DO NOTHING;
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name = 'ADMIN' AND r.scope = 'GLOBAL' AND c.name = 'PROJECT_DELETE'
ON CONFLICT (role_id, capability_id) DO NOTHING;

-- 9. 기존 settings.study_assignment / STUDY_ASSIGNMENT_MANAGE 역할 매핑 제거
DELETE FROM security_role_permission
WHERE permission_id IN (SELECT id FROM security_permission WHERE resource_type = 'settings' AND action = 'study_assignment');
DELETE FROM security_role_capability
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'STUDY_ASSIGNMENT_MANAGE');
