-- ==========================
-- Fix: ANNOTATION_READ_ALL capability → ANNOTATION:READ_ALL permission 매핑 누락
--
-- 문제: ANNOTATION_READ_ALL capability(id=250)가 ANNOTATION:READ(id=447)에만 매핑되어 있고
--       ANNOTATION:READ_ALL(id=460)에는 매핑이 없었음.
--       서버의 check_permission("ANNOTATION", "READ_ALL") 호출이 항상 false 반환.
--       → READER_REVIEWER 역할이 타인의 annotation을 볼 수 없었음.
--
-- 수정: capability_mapping에 ANNOTATION_READ_ALL → ANNOTATION:READ_ALL 매핑 추가.
-- k3s 반영: 2026-04-13 직접 실행 완료.
-- ==========================

BEGIN;

INSERT INTO security_capability_mapping (capability_id, permission_id)
SELECT c.id, p.id
FROM security_capability c, security_permission p
WHERE c.name = 'ANNOTATION_READ_ALL'
  AND p.resource_type = 'ANNOTATION'
  AND p.action = 'READ_ALL'
ON CONFLICT (capability_id, permission_id) DO NOTHING;

COMMIT;

-- Verification
SELECT c.name AS capability, p.resource_type, p.action
FROM security_capability_mapping cm
JOIN security_capability c ON c.id = cm.capability_id
JOIN security_permission p ON p.id = cm.permission_id
WHERE c.name = 'ANNOTATION_READ_ALL'
ORDER BY p.action;
