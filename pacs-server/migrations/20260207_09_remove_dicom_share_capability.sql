-- DICOM_SHARE_ACCESS capability 제거

-- 1. 역할 매핑 제거
DELETE FROM security_role_capability
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'DICOM_SHARE_ACCESS');

-- 2. capability-permission 매핑 제거
DELETE FROM security_capability_mapping
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'DICOM_SHARE_ACCESS');

-- 3. capability 레코드 제거
DELETE FROM security_capability WHERE name = 'DICOM_SHARE_ACCESS';
