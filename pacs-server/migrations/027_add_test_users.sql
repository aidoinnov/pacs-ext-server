-- Migration: Add Test Users
-- Created: 2025-01-11
-- Description: API 테스트를 위한 테스트 계정 추가

-- ==========================
-- TEST USERS
-- ==========================

-- 1. SUPER_ADMIN 테스트 계정
INSERT INTO security_user (
    keycloak_id,
    username,
    email,
    full_name,
    account_status,
    email_verified,
    created_at,
    updated_at
) VALUES (
    'a0000000-0000-0000-0000-000000000001'::uuid,
    'test_super_admin',
    'test_super_admin@example.com',
    'Test SuperAdmin',
    'ACTIVE',
    true,
    NOW(),
    NOW()
) ON CONFLICT (keycloak_id) DO NOTHING;

-- 2. ADMIN 테스트 계정
INSERT INTO security_user (
    keycloak_id,
    username,
    email,
    full_name,
    account_status,
    email_verified,
    created_at,
    updated_at
) VALUES (
    'a0000000-0000-0000-0000-000000000002'::uuid,
    'test_admin',
    'test_admin@example.com',
    'Test Admin',
    'ACTIVE',
    true,
    NOW(),
    NOW()
) ON CONFLICT (keycloak_id) DO NOTHING;

-- 3. USER 테스트 계정
INSERT INTO security_user (
    keycloak_id,
    username,
    email,
    full_name,
    account_status,
    email_verified,
    created_at,
    updated_at
) VALUES (
    'a0000000-0000-0000-0000-000000000003'::uuid,
    'test_user',
    'test_user@example.com',
    'Test User',
    'ACTIVE',
    true,
    NOW(),
    NOW()
) ON CONFLICT (keycloak_id) DO NOTHING;

-- ==========================
-- ASSIGN ROLES TO TEST USERS
-- ==========================

-- SUPER_ADMIN 그룹 찾기 및 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_super_admin' AND g.name = 'SUPER_ADMIN'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- ADMIN 그룹 찾기 및 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_admin' AND g.name = 'ADMIN'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- USER 그룹 찾기 및 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_user' AND g.name = 'USER'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- ==========================
-- VERIFICATION
-- ==========================

-- 생성된 테스트 계정 확인
SELECT
    u.username,
    u.email,
    g.name as group_name,
    u.account_status,
    u.email_verified
FROM security_user u
LEFT JOIN security_user_group ug ON u.id = ug.user_id
LEFT JOIN security_group g ON ug.group_id = g.id
WHERE u.username IN ('test_super_admin', 'test_admin', 'test_user')
ORDER BY u.username;

