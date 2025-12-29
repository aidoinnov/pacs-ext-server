-- Migration: Create series_user_report and guide template system
-- Created: 2025-01-15
-- Description: Creates tables for user-specific reports on DICOM Series
-- Supports both project-scoped and global reports, with guide template system
-- Includes original templates, user custom templates, and image ownership management

-- ==========================
-- 1. SERIES USER REPORT
-- ==========================

CREATE TABLE IF NOT EXISTS series_user_report (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES project_data_series(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    project_id INTEGER NULL REFERENCES security_project(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'unread' CHECK (status IN ('unread', 'approval', 'unapproval')),
    dictate_file_path TEXT NULL,
    dictate_file_size BIGINT NULL,
    dictate_mime_type TEXT NULL,
    description TEXT NOT NULL,
    conclusion TEXT NOT NULL,
    bodypart TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (series_id, user_id, project_id)
);

-- ==========================
-- 2. REPORT GUIDE TEMPLATE (Original Templates)
-- ==========================

CREATE TABLE IF NOT EXISTS report_guide_template (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    conclusion TEXT,
    bodypart TEXT,
    is_shared BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by INTEGER NOT NULL REFERENCES security_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ==========================
-- 3. TEMPLATE MODALITY MAPPING
-- ==========================

CREATE TABLE IF NOT EXISTS report_guide_template_modality (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id INTEGER NOT NULL REFERENCES report_guide_template(id) ON DELETE CASCADE,
    modality TEXT NOT NULL,
    UNIQUE (template_id, modality)
);

-- ==========================
-- 4. TEMPLATE IMAGES
-- ==========================

CREATE TABLE IF NOT EXISTS report_guide_template_image (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id INTEGER NOT NULL REFERENCES report_guide_template(id) ON DELETE CASCADE,
    image_path TEXT NOT NULL,
    image_url TEXT NOT NULL,
    file_size BIGINT,
    mime_type TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_shared BOOLEAN NOT NULL DEFAULT true,
    uploaded_by INTEGER NOT NULL REFERENCES security_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_id, image_path)
);

-- ==========================
-- 5. USER CUSTOM TEMPLATES
-- ==========================

CREATE TABLE IF NOT EXISTS user_custom_report_template (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    base_template_id INTEGER NULL REFERENCES report_guide_template(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    conclusion TEXT,
    bodypart TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, name)
);

-- ==========================
-- 6. CUSTOM TEMPLATE MODALITY
-- ==========================

CREATE TABLE IF NOT EXISTS user_custom_template_modality (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    custom_template_id INTEGER NOT NULL REFERENCES user_custom_report_template(id) ON DELETE CASCADE,
    modality TEXT NOT NULL,
    UNIQUE (custom_template_id, modality)
);

-- ==========================
-- 7. CUSTOM TEMPLATE IMAGES
-- ==========================

CREATE TABLE IF NOT EXISTS user_custom_template_image (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    custom_template_id INTEGER NOT NULL REFERENCES user_custom_report_template(id) ON DELETE CASCADE,
    image_path TEXT NOT NULL,
    image_url TEXT NOT NULL,
    file_size BIGINT,
    mime_type TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_shared BOOLEAN NOT NULL DEFAULT false,
    uploaded_by INTEGER NOT NULL REFERENCES security_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (custom_template_id, image_path)
);

-- ==========================
-- 8. REPORT-GUIDE MAPPING
-- ==========================

CREATE TABLE IF NOT EXISTS series_user_report_guide (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES series_user_report(id) ON DELETE CASCADE,
    template_id INTEGER NULL REFERENCES report_guide_template(id) ON DELETE CASCADE,
    custom_template_id INTEGER NULL REFERENCES user_custom_report_template(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (template_id IS NOT NULL AND custom_template_id IS NULL) OR
        (template_id IS NULL AND custom_template_id IS NOT NULL)
    ),
    UNIQUE (report_id, COALESCE(template_id, -1), COALESCE(custom_template_id, -1))
);

-- ==========================
-- INDEXES
-- ==========================

-- Report indexes
CREATE INDEX IF NOT EXISTS idx_series_user_report_series ON series_user_report(series_id);
CREATE INDEX IF NOT EXISTS idx_series_user_report_user ON series_user_report(user_id);
CREATE INDEX IF NOT EXISTS idx_series_user_report_project ON series_user_report(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_series_user_report_series_user ON series_user_report(series_id, user_id);
CREATE INDEX IF NOT EXISTS idx_series_user_report_status ON series_user_report(status);

-- Template indexes
CREATE INDEX IF NOT EXISTS idx_report_guide_template_created_by ON report_guide_template(created_by);
CREATE INDEX IF NOT EXISTS idx_report_guide_template_active ON report_guide_template(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_report_guide_template_shared ON report_guide_template(is_shared) WHERE is_shared = true;

-- Template modality indexes
CREATE INDEX IF NOT EXISTS idx_template_modality_template ON report_guide_template_modality(template_id);
CREATE INDEX IF NOT EXISTS idx_template_modality_modality ON report_guide_template_modality(modality);

-- Template image indexes
CREATE INDEX IF NOT EXISTS idx_template_image_template ON report_guide_template_image(template_id);
CREATE INDEX IF NOT EXISTS idx_template_image_uploaded_by ON report_guide_template_image(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_template_image_shared ON report_guide_template_image(is_shared) WHERE is_shared = true;

-- Custom template indexes
CREATE INDEX IF NOT EXISTS idx_user_custom_template_user ON user_custom_report_template(user_id);
CREATE INDEX IF NOT EXISTS idx_user_custom_template_base ON user_custom_report_template(base_template_id) WHERE base_template_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_custom_template_active ON user_custom_report_template(is_active) WHERE is_active = true;

-- Custom template modality indexes
CREATE INDEX IF NOT EXISTS idx_custom_template_modality_template ON user_custom_template_modality(custom_template_id);
CREATE INDEX IF NOT EXISTS idx_custom_template_modality_modality ON user_custom_template_modality(modality);

-- Custom template image indexes
CREATE INDEX IF NOT EXISTS idx_custom_template_image_template ON user_custom_template_image(custom_template_id);
CREATE INDEX IF NOT EXISTS idx_custom_template_image_uploaded_by ON user_custom_template_image(uploaded_by);

-- Report-guide mapping indexes
CREATE INDEX IF NOT EXISTS idx_report_guide_report ON series_user_report_guide(report_id);
CREATE INDEX IF NOT EXISTS idx_report_guide_template ON series_user_report_guide(template_id) WHERE template_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_report_guide_custom_template ON series_user_report_guide(custom_template_id) WHERE custom_template_id IS NOT NULL;

-- ==========================
-- TRIGGERS
-- ==========================

-- Update updated_at for series_user_report
CREATE OR REPLACE FUNCTION update_series_user_report_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_series_user_report_updated_at
    BEFORE UPDATE ON series_user_report
    FOR EACH ROW
    EXECUTE FUNCTION update_series_user_report_updated_at();

-- Update updated_at for report_guide_template
CREATE OR REPLACE FUNCTION update_report_guide_template_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_report_guide_template_updated_at
    BEFORE UPDATE ON report_guide_template
    FOR EACH ROW
    EXECUTE FUNCTION update_report_guide_template_updated_at();

-- Update updated_at for user_custom_report_template
CREATE OR REPLACE FUNCTION update_user_custom_template_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_user_custom_template_updated_at
    BEFORE UPDATE ON user_custom_report_template
    FOR EACH ROW
    EXECUTE FUNCTION update_user_custom_template_updated_at();

-- ==========================
-- COMMENTS
-- ==========================

COMMENT ON TABLE series_user_report IS '사용자별 DICOM Series 리포트 (프로젝트 종속 또는 전역)';
COMMENT ON COLUMN series_user_report.series_id IS 'Series ID (project_data_series 참조)';
COMMENT ON COLUMN series_user_report.user_id IS '사용자 ID (security_user 참조)';
COMMENT ON COLUMN series_user_report.project_id IS '프로젝트 ID (NULL이면 전역 report, 값이 있으면 프로젝트별 report)';
COMMENT ON COLUMN series_user_report.status IS '리포트 상태: unread, approval, unapproval';
COMMENT ON COLUMN series_user_report.dictate_file_path IS '오디오 파일 경로 (Object Storage)';
COMMENT ON COLUMN series_user_report.description IS '리포트 설명';
COMMENT ON COLUMN series_user_report.conclusion IS '리포트 결론';
COMMENT ON COLUMN series_user_report.bodypart IS '신체 부위';

COMMENT ON TABLE report_guide_template IS '리포트 가이드 원본 템플릿 (관리자/템플릿 작성자만 생성/수정 가능)';
COMMENT ON TABLE user_custom_report_template IS '사용자 커스텀 리포트 템플릿 (원본 템플릿 복사하여 수정 가능)';
COMMENT ON TABLE report_guide_template_image IS '템플릿 이미지 (공유/사용자 전용 지원)';
COMMENT ON TABLE user_custom_template_image IS '커스텀 템플릿 이미지 (사용자 전용 기본값)';
COMMENT ON TABLE series_user_report_guide IS 'Report와 가이드 템플릿 매핑 (원본 또는 커스텀 템플릿)';



