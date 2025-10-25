-- Migration: Add Capability Abstraction Layer
-- Created: 2025-10-25
-- Description: Add capability abstraction layer between roles and permissions

-- 1. Capability 테이블 생성
CREATE TABLE security_capability (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE security_capability IS 'Abstract capabilities that represent user-friendly "what users can do"';
COMMENT ON COLUMN security_capability.name IS 'Internal capability name (e.g., MANAGE_USERS)';
COMMENT ON COLUMN security_capability.display_name IS 'UI display name (e.g., "사용자 관리")';
COMMENT ON COLUMN security_capability.category IS 'Capability category for grouping (e.g., "관리", "프로젝트", "데이터")';

-- 2. Capability-Permission 매핑 테이블
CREATE TABLE security_capability_mapping (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    capability_id INTEGER NOT NULL REFERENCES security_capability(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permission(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(capability_id, permission_id)
);

COMMENT ON TABLE security_capability_mapping IS 'Maps capabilities to their underlying permissions';

-- 3. Role-Capability 테이블
CREATE TABLE security_role_capability (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    capability_id INTEGER NOT NULL REFERENCES security_capability(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, capability_id)
);

COMMENT ON TABLE security_role_capability IS 'Assigns capabilities to roles';

-- 4. 인덱스 생성
CREATE INDEX idx_capability_mapping_capability ON security_capability_mapping(capability_id);
CREATE INDEX idx_capability_mapping_permission ON security_capability_mapping(permission_id);
CREATE INDEX idx_role_capability_role ON security_role_capability(role_id);
CREATE INDEX idx_role_capability_capability ON security_role_capability(capability_id);
