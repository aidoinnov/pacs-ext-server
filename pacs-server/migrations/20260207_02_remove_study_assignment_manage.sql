-- STUDY_ASSIGNMENT_MANAGE, settings.study_assignment 완전 제거
-- (20260207_01에서 역할 매핑 제거 후, orphan 레코드 정리)

-- 1. capability_mapping 제거 (STUDY_ASSIGNMENT_MANAGE ↔ settings.study_assignment)
DELETE FROM security_capability_mapping
WHERE capability_id IN (SELECT id FROM security_capability WHERE name = 'STUDY_ASSIGNMENT_MANAGE');

-- 2. capability 레코드 제거
DELETE FROM security_capability WHERE name = 'STUDY_ASSIGNMENT_MANAGE';

-- 3. permission 레코드 제거 (역할 매핑은 이미 20260207_01에서 제거됨)
DELETE FROM security_permission
WHERE resource_type = 'settings' AND action = 'study_assignment';
