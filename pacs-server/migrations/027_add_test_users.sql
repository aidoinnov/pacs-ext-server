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
-- CREATE TEST GROUPS
-- ==========================

-- 1. SUPER_ADMIN 그룹 생성
INSERT INTO security_group (
    name,
    description,
    created_at,
    updated_at
) VALUES (
    'Test SUPER_ADMIN Group',
    'Test group for SUPER_ADMIN role',
    NOW(),
    NOW()
) ON CONFLICT DO NOTHING;

-- 2. ADMIN 그룹 생성
INSERT INTO security_group (
    name,
    description,
    created_at,
    updated_at
) VALUES (
    'Test ADMIN Group',
    'Test group for ADMIN role',
    NOW(),
    NOW()
) ON CONFLICT DO NOTHING;

-- 3. USER 그룹 생성
INSERT INTO security_group (
    name,
    description,
    created_at,
    updated_at
) VALUES (
    'Test USER Group',
    'Test group for USER role',
    NOW(),
    NOW()
) ON CONFLICT DO NOTHING;

-- ==========================
-- ASSIGN ROLES TO GROUPS
-- ==========================

-- SUPER_ADMIN 역할을 SUPER_ADMIN 그룹에 할당
INSERT INTO security_group_role (group_id, role_id)
SELECT g.id, r.id
FROM security_group g, security_role r
WHERE g.name = 'Test SUPER_ADMIN Group' AND r.name = 'SUPER_ADMIN'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- ADMIN 역할을 ADMIN 그룹에 할당
INSERT INTO security_group_role (group_id, role_id)
SELECT g.id, r.id
FROM security_group g, security_role r
WHERE g.name = 'Test ADMIN Group' AND r.name = 'ADMIN'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- USER 역할을 USER 그룹에 할당
INSERT INTO security_group_role (group_id, role_id)
SELECT g.id, r.id
FROM security_group g, security_role r
WHERE g.name = 'Test USER Group' AND r.name = 'USER'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- ==========================
-- ASSIGN USERS TO GROUPS
-- ==========================

-- test_super_admin을 SUPER_ADMIN 그룹에 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_super_admin' AND g.name = 'Test SUPER_ADMIN Group'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- test_admin을 ADMIN 그룹에 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_admin' AND g.name = 'Test ADMIN Group'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- test_user를 USER 그룹에 할당
INSERT INTO security_user_group (user_id, group_id)
SELECT u.id, g.id
FROM security_user u, security_group g
WHERE u.username = 'test_user' AND g.name = 'Test USER Group'
ON CONFLICT (user_id, group_id) DO NOTHING;

-- ==========================
-- VERIFICATION
-- ==========================

-- 생성된 테스트 계정 및 역할 확인
SELECT
    u.username,
    u.email,
    g.name as group_name,
    r.name as role_name,
    r.scope as role_scope,
    u.account_status,
    u.email_verified
FROM security_user u
LEFT JOIN security_user_group ug ON u.id = ug.user_id
LEFT JOIN security_group g ON ug.group_id = g.id
LEFT JOIN security_group_role gr ON g.id = gr.group_id
LEFT JOIN security_role r ON gr.role_id = r.id
WHERE u.username IN ('test_super_admin', 'test_admin', 'test_user')
ORDER BY u.username;

