-- PROJECT 카테고리 → MANAGE(관리)로 통합
-- 프로젝트 관련 capability를 관리 카테고리 내로 통합

UPDATE security_capability
SET category = '관리', category_label = 'MANAGE'
WHERE name IN (
    'PROJECT_CREATE', 'PROJECT_EDIT', 'PROJECT_ASSIGN',
    'PROJECT_DATA_ASSIGN', 'PROJECT_DELETE'
);
