-- SETTINGS_ACCESS_AND_ROLES, settings.access_and_roles 제거
-- ROLE_MANAGEMENT가 이미 역할/접근 관리 권한을 커버함 (중복 제거)

-- 1. 역할 매핑 제거
DELETE FROM security_role_permission
WHERE permission_id IN (SELECT id FROM security_permission WHERE resource_type = 'settings' AND action = 'access_and_roles');
DELETE FROM security_role_capability
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'SETTINGS_ACCESS_AND_ROLES');

-- 2. capability_mapping 제거
DELETE FROM security_capability_mapping
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'SETTINGS_ACCESS_AND_ROLES');

-- 3. capability 레코드 제거
DELETE FROM security_capability WHERE name = 'SETTINGS_ACCESS_AND_ROLES';

-- 4. permission 레코드 제거
DELETE FROM security_permission
WHERE resource_type = 'settings' AND action = 'access_and_roles';
