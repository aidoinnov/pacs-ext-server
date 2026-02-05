-- PROJECT_MANAGEMENT 역할 할당 보정 (20260207_06에서 누락된 역할)
INSERT INTO security_role_capability (role_id, capability_id)
SELECT r.id, c.id FROM security_role r, security_capability c
WHERE r.name IN ('SUPER_ADMIN', 'PROJECT_ADMIN', 'ADMIN')
  AND c.name = 'PROJECT_MANAGEMENT'
ON CONFLICT (role_id, capability_id) DO NOTHING;
