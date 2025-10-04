-- ==========================
-- PACS Extension Server DDL (2025-10 Revision)
-- ==========================

-- ==========================
-- ENUMS
-- ==========================

CREATE TYPE condition_type_enum AS ENUM ('ALLOW', 'DENY', 'LIMIT');
CREATE TYPE resource_level_enum AS ENUM ('STUDY', 'SERIES', 'INSTANCE');
CREATE TYPE grant_action_enum AS ENUM ('GRANT', 'REVOKE');

-- ==========================
-- SECURITY SCHEMA
-- ==========================

CREATE TABLE security_user (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    keycloak_id UUID UNIQUE NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE security_project (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE security_role (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    scope TEXT NOT NULL DEFAULT 'GLOBAL' CHECK (scope IN ('GLOBAL','PROJECT')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE security_permission (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    resource_type TEXT NOT NULL,
    action TEXT NOT NULL,
    UNIQUE (resource_type, action)
);

CREATE TABLE security_role_permission (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permission(id) ON DELETE CASCADE,
    scope TEXT,
    UNIQUE (role_id, permission_id)
);

CREATE TABLE security_user_project (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    UNIQUE (user_id, project_id)
);

CREATE TABLE security_project_role (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    UNIQUE (project_id, role_id)
);

CREATE TABLE security_project_permission (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permission(id) ON DELETE CASCADE,
    scope TEXT,
    inherits_from_role_permission BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE (project_id, permission_id)
);

CREATE TABLE security_access_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    resource_type TEXT NOT NULL,
    resource_level resource_level_enum NOT NULL DEFAULT 'INSTANCE',
    dicom_tag TEXT,
    operator TEXT NOT NULL,
    value TEXT,
    condition_type condition_type_enum NOT NULL
);

CREATE TABLE security_role_access_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    access_condition_id INTEGER NOT NULL REFERENCES security_access_condition(id) ON DELETE CASCADE,
    UNIQUE (role_id, access_condition_id)
);

CREATE TABLE security_project_access_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    access_condition_id INTEGER NOT NULL REFERENCES security_access_condition(id) ON DELETE CASCADE,
    UNIQUE (project_id, access_condition_id)
);

CREATE TABLE security_grant_log (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    granted_by INTEGER NOT NULL REFERENCES security_user(id),
    granted_to INTEGER NOT NULL REFERENCES security_user(id),
    role_id INTEGER REFERENCES security_role(id),
    project_id INTEGER REFERENCES security_project(id),
    action grant_action_enum NOT NULL DEFAULT 'GRANT',
    via_group_id INTEGER,
    logged_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE security_access_log (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES security_user(id),
    project_id INTEGER REFERENCES security_project(id),
    resource_type TEXT NOT NULL,
    study_uid TEXT,
    series_uid TEXT,
    instance_uid TEXT,
    action TEXT NOT NULL,
    result TEXT NOT NULL,
    dicom_tag_check TEXT,
    ae_title TEXT,
    ip_address TEXT,
    session_id TEXT,
    via_group_id INTEGER,
    logged_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ==========================
-- OPTIONAL GROUP EXTENSION
-- ==========================

CREATE TABLE security_group (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_id, name)
);

CREATE TABLE security_user_group (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    group_id INTEGER NOT NULL REFERENCES security_group(id) ON DELETE CASCADE,
    UNIQUE (user_id, group_id)
);

CREATE TABLE security_group_role (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES security_group(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    UNIQUE (group_id, role_id)
);

-- ==========================
-- VIEWER SCHEMA
-- ==========================

CREATE TABLE viewer_hanging_protocol (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    owner_user_id INTEGER NOT NULL REFERENCES security_user(id),
    name TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE viewer_hp_condition (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    protocol_id INTEGER NOT NULL REFERENCES viewer_hanging_protocol(id) ON DELETE CASCADE,
    dicom_tag TEXT NOT NULL,
    operator TEXT NOT NULL,
    value TEXT
);

CREATE TABLE viewer_hp_layout (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    protocol_id INTEGER NOT NULL REFERENCES viewer_hanging_protocol(id) ON DELETE CASCADE,
    rows INTEGER NOT NULL,
    cols INTEGER NOT NULL
);

CREATE TABLE viewer_hp_viewport (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    layout_id INTEGER NOT NULL REFERENCES viewer_hp_layout(id) ON DELETE CASCADE,
    position_row INTEGER NOT NULL,
    position_col INTEGER NOT NULL,
    selection_rule TEXT,
    sort_order TEXT
);

-- ==========================
-- ANNOTATION SCHEMA
-- ==========================

CREATE TABLE annotation_annotation (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES security_user(id),
    study_uid TEXT NOT NULL,
    series_uid TEXT,
    instance_uid TEXT,
    tool_name TEXT NOT NULL,
    tool_version TEXT,
    data JSONB NOT NULL,
    is_shared BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE annotation_annotation_history (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES security_user(id),
    action TEXT NOT NULL,
    data_before JSONB,
    data_after JSONB,
    action_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ==========================
-- FOREIGN KEY EXTENSIONS
-- ==========================

ALTER TABLE security_grant_log
    ADD CONSTRAINT fk_grant_log_via_group
    FOREIGN KEY (via_group_id) REFERENCES security_group(id);

ALTER TABLE security_access_log
    ADD CONSTRAINT fk_access_log_via_group
    FOREIGN KEY (via_group_id) REFERENCES security_group(id);

-- ==========================
-- INDEXES
-- ==========================

-- (same index set as before, unchanged)
CREATE INDEX idx_user_keycloak_id ON security_user(keycloak_id);
CREATE INDEX idx_user_username ON security_user(username);
CREATE INDEX idx_user_email ON security_user(email);

CREATE INDEX idx_project_name ON security_project(name);
CREATE INDEX idx_project_active ON security_project(is_active);

CREATE INDEX idx_user_project_user ON security_user_project(user_id);
CREATE INDEX idx_user_project_project ON security_user_project(project_id);

CREATE INDEX idx_project_role_project ON security_project_role(project_id);
CREATE INDEX idx_project_role_role ON security_project_role(role_id);

CREATE INDEX idx_role_permission_role ON security_role_permission(role_id);
CREATE INDEX idx_role_permission_permission ON security_role_permission(permission_id);

CREATE INDEX idx_grant_log_granted_by ON security_grant_log(granted_by);
CREATE INDEX idx_grant_log_granted_to ON security_grant_log(granted_to);
CREATE INDEX idx_grant_log_project ON security_grant_log(project_id);
CREATE INDEX idx_grant_log_logged_at ON security_grant_log(logged_at);

CREATE INDEX idx_access_log_user ON security_access_log(user_id);
CREATE INDEX idx_access_log_project ON security_access_log(project_id);
CREATE INDEX idx_access_log_logged_at ON security_access_log(logged_at);
CREATE INDEX idx_access_log_study_uid ON security_access_log(study_uid);
CREATE INDEX idx_access_log_series_uid ON security_access_log(series_uid);

CREATE INDEX idx_group_project ON security_group(project_id);
CREATE INDEX idx_user_group_user ON security_user_group(user_id);
CREATE INDEX idx_user_group_group ON security_user_group(group_id);
CREATE INDEX idx_group_role_group ON security_group_role(group_id);
CREATE INDEX idx_group_role_role ON security_group_role(role_id);

CREATE INDEX idx_hanging_protocol_project ON viewer_hanging_protocol(project_id);
CREATE INDEX idx_hanging_protocol_owner ON viewer_hanging_protocol(owner_user_id);
CREATE INDEX idx_hp_condition_protocol ON viewer_hp_condition(protocol_id);
CREATE INDEX idx_hp_layout_protocol ON viewer_hp_layout(protocol_id);
CREATE INDEX idx_hp_viewport_layout ON viewer_hp_viewport(layout_id);

CREATE INDEX idx_annotation_project ON annotation_annotation(project_id);
CREATE INDEX idx_annotation_user ON annotation_annotation(user_id);
CREATE INDEX idx_annotation_study ON annotation_annotation(study_uid);
CREATE INDEX idx_annotation_series ON annotation_annotation(series_uid);
CREATE INDEX idx_annotation_history_annotation ON annotation_annotation_history(annotation_id);
CREATE INDEX idx_annotation_history_timestamp ON annotation_annotation_history(action_at);

-- ==========================
-- COMMENTS
-- ==========================

COMMENT ON TABLE security_user IS '사용자 정보 (Keycloak 연동)';
COMMENT ON TABLE security_project IS '프로젝트 (연구/임상 시험 등)';
COMMENT ON TABLE security_role IS '역할 정의 (scope: GLOBAL / PROJECT)';
COMMENT ON TABLE security_permission IS '권한 정의 (resource_type + action)';
COMMENT ON TABLE security_user_project IS '사용자-프로젝트 멤버십';
COMMENT ON TABLE security_project_role IS '프로젝트-역할 매핑';
COMMENT ON TABLE security_project_permission IS '프로젝트-권한 매핑 (role 권한 상속 여부 포함)';
COMMENT ON TABLE security_access_condition IS 'DICOM 태그 기반 접근 조건 (resource_level 포함)';
COMMENT ON TABLE security_grant_log IS '권한 부여 이력 (direct / group 경유)';
COMMENT ON TABLE security_access_log IS '접근 로그 (DICOM 리소스 단위)';
COMMENT ON TABLE security_group IS '그룹 (프로젝트 내 사용자 그룹)';
COMMENT ON TABLE viewer_hanging_protocol IS 'Hanging Protocol 정의';
COMMENT ON TABLE annotation_annotation IS 'DICOM 인스턴스에 대한 주석 (JSONB 데이터 포함)';
