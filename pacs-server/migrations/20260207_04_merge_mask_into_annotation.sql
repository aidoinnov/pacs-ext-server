-- MASK capability → ANNOTATION으로 통합
-- MASK(AI 어노테이션)와 ANNOTATION(수동 어노테이션)은 동일한 접근 제어로 처리

-- 1. ANNOTATION capability에 MASK permission 매핑 추가
--    (ANNOTATION_READ_OWN, ANNOTATION_READ_ALL → MASK:READ, DOWNLOAD)
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name IN ('ANNOTATION_READ_OWN', 'ANNOTATION_READ_ALL')
  AND p.resource_type = 'MASK' AND p.action IN ('READ', 'DOWNLOAD')
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- ANNOTATION_WRITE → MASK:CREATE, UPDATE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_WRITE'
  AND p.resource_type = 'MASK' AND p.action IN ('CREATE', 'UPDATE')
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- ANNOTATION_DELETE → MASK:DELETE
INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_DELETE'
  AND p.resource_type = 'MASK' AND p.action = 'DELETE'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

-- 2. ANNOTATION capability 설명 업데이트 (어노테이션+마스크 통합)
UPDATE security_capability
SET description = '어노테이션 및 마스크(AI) 조회 권한'
WHERE name IN ('ANNOTATION_READ_OWN', 'ANNOTATION_READ_ALL');
UPDATE security_capability
SET description = '어노테이션 및 마스크(AI) 생성·수정 권한'
WHERE name = 'ANNOTATION_WRITE';
UPDATE security_capability
SET description = '어노테이션 및 마스크(AI) 삭제 권한'
WHERE name = 'ANNOTATION_DELETE';

-- 3. MASK capability 역할 매핑 제거
DELETE FROM security_role_capability
WHERE capability_id IN (SELECT id FROM security_capability WHERE name IN ('MASK_READ', 'MASK_WRITE', 'MASK_DELETE'));

-- 4. MASK capability-permission 매핑 제거
DELETE FROM security_capability_mapping
WHERE capability_id IN (SELECT id FROM security_capability WHERE name IN ('MASK_READ', 'MASK_WRITE', 'MASK_DELETE'));

-- 5. MASK capability 레코드 제거
DELETE FROM security_capability WHERE name IN ('MASK_READ', 'MASK_WRITE', 'MASK_DELETE');
