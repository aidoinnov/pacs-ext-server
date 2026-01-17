-- DICOM_GLOBAL_ACCESS 권한 확인 스크립트

-- 1. DICOM_GLOBAL_ACCESS Capability 존재 확인
SELECT 'Capability 확인' as check_type, 
       id, name, display_name, category, category_label
FROM security_capability 
WHERE name = 'DICOM_GLOBAL_ACCESS';

-- 2. DICOM:READ_ALL Permission 존재 확인
SELECT 'Permission 확인' as check_type,
       id, category, resource_type, action
FROM security_permission 
WHERE resource_type = 'DICOM' AND action = 'READ_ALL';

-- 3. Capability-Permission 매핑 확인
SELECT 'Capability-Permission 매핑' as check_type,
       c.name as capability_name, 
       p.resource_type, 
       p.action
FROM security_capability_mapping cm
JOIN security_capability c ON cm.capability_id = c.id
JOIN security_permission p ON cm.permission_id = p.id
WHERE c.name = 'DICOM_GLOBAL_ACCESS';

-- 4. DICOM_GLOBAL_ACCESS가 할당된 Role 확인
SELECT 'Role에 할당된 Capability' as check_type,
       r.id as role_id,
       r.name as role_name, 
       r.scope,
       c.name as capability_name
FROM security_role_capability rc
JOIN security_role r ON rc.role_id = r.id
JOIN security_capability c ON rc.capability_id = c.id
WHERE c.name = 'DICOM_GLOBAL_ACCESS';

-- 5. DICOM_GLOBAL_ACCESS 권한을 가진 사용자 확인
SELECT 'DICOM_GLOBAL_ACCESS 권한 보유 사용자' as check_type,
       u.id as user_id,
       u.username,
       u.email,
       r.name as role_name,
       p.id as project_id,
       p.name as project_name
FROM security_user_project sup
JOIN security_user u ON sup.user_id = u.id
JOIN security_role r ON sup.role_id = r.id
JOIN security_project p ON sup.project_id = p.id
JOIN security_role_capability src ON r.id = src.role_id
JOIN security_capability c ON src.capability_id = c.id
WHERE c.name = 'DICOM_GLOBAL_ACCESS'
ORDER BY u.id, p.id;

-- 6. 모든 사용자와 Role 확인 (참고용)
SELECT 'All Users and Roles' as check_type,
       u.id as user_id,
       u.username,
       u.email,
       r.name as role_name,
       p.id as project_id,
       p.name as project_name
FROM security_user u
LEFT JOIN security_user_project sup ON u.id = sup.user_id
LEFT JOIN security_role r ON sup.role_id = r.id
LEFT JOIN security_project p ON sup.project_id = p.id
ORDER BY u.id, p.id;

