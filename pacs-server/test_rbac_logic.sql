-- RBAC 로직 테스트 SQL 스크립트
-- 이 스크립트는 새로운 RBAC 로직을 테스트합니다

-- ============================================================================
-- 1. 테스트 데이터 준비
-- ============================================================================

-- 프로젝트 생성 (이미 있으면 스킵)
INSERT INTO security_project (name, description, sponsor, start_date, end_date, status, is_active)
VALUES ('Test Project', 'RBAC 테스트용 프로젝트', 'Test Sponsor', CURRENT_DATE, CURRENT_DATE + INTERVAL '1 year', 'ACTIVE', true)
ON CONFLICT DO NOTHING;

-- 사용자 생성 (이미 있으면 스킵)
INSERT INTO security_user (keycloak_id, username, email, full_name, is_active)
VALUES 
    ('00000000-0000-0000-0000-000000000001'::uuid, 'test_user_1', 'user1@test.com', 'Test User 1', true),
    ('00000000-0000-0000-0000-000000000002'::uuid, 'test_user_2', 'user2@test.com', 'Test User 2', true),
    ('00000000-0000-0000-0000-000000000003'::uuid, 'test_user_3', 'user3@test.com', 'Test User 3', true)
ON CONFLICT (keycloak_id) DO NOTHING;

-- 프로젝트 멤버십 추가
INSERT INTO security_user_project (user_id, project_id)
SELECT u.id, p.id
FROM security_user u, security_project p
WHERE u.username IN ('test_user_1', 'test_user_2', 'test_user_3')
  AND p.name = 'Test Project'
ON CONFLICT DO NOTHING;

-- Study 데이터 생성 (project_id 없이)
INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
VALUES 
    ('1.2.3.100', 'Test Study 100', 'P001', 'Patient 001', '2024-01-01'),
    ('1.2.3.101', 'Test Study 101', 'P002', 'Patient 002', '2024-01-02'),
    ('1.2.3.102', 'Test Study 102', 'P003', 'Patient 003', '2024-01-03')
ON CONFLICT (study_uid) DO NOTHING;

-- project_data 테이블에 Study 매핑 (프로젝트에 Study 포함)
INSERT INTO project_data (project_id, resource_level, study_id)
SELECT p.id, 'STUDY', s.id
FROM security_project p, project_data_study s
WHERE p.name = 'Test Project'
  AND s.study_uid IN ('1.2.3.100', '1.2.3.101', '1.2.3.102')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 2. 테스트 시나리오 설정
-- ============================================================================

-- 시나리오 1: User 1 - 기본 접근 (모든 Study 접근 가능)
-- → project_data_access에 레코드 없음 = 기본 허용

-- 시나리오 2: User 2 - Study 100 거부
INSERT INTO project_data_access (project_id, user_id, resource_level, study_id, status)
SELECT p.id, u.id, 'STUDY', s.id, 'DENIED'
FROM security_project p, security_user u, project_data_study s
WHERE p.name = 'Test Project'
  AND u.username = 'test_user_2'
  AND s.study_uid = '1.2.3.100'
ON CONFLICT DO NOTHING;

-- 시나리오 3: User 3 - Study 101만 명시적 승인, 나머지는 기본 허용
INSERT INTO project_data_access (project_id, user_id, resource_level, study_id, status)
SELECT p.id, u.id, 'STUDY', s.id, 'APPROVED'
FROM security_project p, security_user u, project_data_study s
WHERE p.name = 'Test Project'
  AND u.username = 'test_user_3'
  AND s.study_uid = '1.2.3.101'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 3. 테스트 쿼리 (RBAC 로직 시뮬레이션)
-- ============================================================================

-- User 1의 접근 가능한 Study 조회 (기본 허용 - 모든 Study)
SELECT 
    'User 1 (기본 허용)' as scenario,
    s.study_uid,
    s.study_description,
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'DENIED'
        ) THEN '❌ DENIED'
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'APPROVED'
        ) THEN '✅ APPROVED'
        ELSE '✅ DEFAULT (프로젝트 멤버)'
    END as access_status
FROM security_user u
CROSS JOIN security_project p
CROSS JOIN project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
WHERE u.username = 'test_user_1'
  AND p.name = 'Test Project'
ORDER BY s.study_uid;

-- User 2의 접근 가능한 Study 조회 (Study 100 거부)
SELECT 
    'User 2 (Study 100 거부)' as scenario,
    s.study_uid,
    s.study_description,
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'DENIED'
        ) THEN '❌ DENIED'
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'APPROVED'
        ) THEN '✅ APPROVED'
        ELSE '✅ DEFAULT (프로젝트 멤버)'
    END as access_status
FROM security_user u
CROSS JOIN security_project p
CROSS JOIN project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
WHERE u.username = 'test_user_2'
  AND p.name = 'Test Project'
ORDER BY s.study_uid;

-- User 3의 접근 가능한 Study 조회 (Study 101 명시적 승인)
SELECT 
    'User 3 (Study 101 승인)' as scenario,
    s.study_uid,
    s.study_description,
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'DENIED'
        ) THEN '❌ DENIED'
        WHEN EXISTS (
            SELECT 1 FROM project_data_access pda
            WHERE pda.user_id = u.id 
              AND pda.project_id = p.id
              AND pda.study_id = s.id
              AND pda.status = 'APPROVED'
        ) THEN '✅ APPROVED'
        ELSE '✅ DEFAULT (프로젝트 멤버)'
    END as access_status
FROM security_user u
CROSS JOIN security_project p
CROSS JOIN project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
WHERE u.username = 'test_user_3'
  AND p.name = 'Test Project'
ORDER BY s.study_uid;

-- ============================================================================
-- 4. 예상 결과
-- ============================================================================
/*
User 1 (기본 허용):
- Study 100: ✅ DEFAULT (프로젝트 멤버)
- Study 101: ✅ DEFAULT (프로젝트 멤버)
- Study 102: ✅ DEFAULT (프로젝트 멤버)

User 2 (Study 100 거부):
- Study 100: ❌ DENIED
- Study 101: ✅ DEFAULT (프로젝트 멤버)
- Study 102: ✅ DEFAULT (프로젝트 멤버)

User 3 (Study 101 승인):
- Study 100: ✅ DEFAULT (프로젝트 멤버)
- Study 101: ✅ APPROVED
- Study 102: ✅ DEFAULT (프로젝트 멤버)
*/

