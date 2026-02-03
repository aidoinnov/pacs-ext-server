-- ==========================
-- Guide Image Independent Management
-- ==========================
-- 작성일: 2026-02-01
-- 목적: 템플릿과 독립적인 가이드 이미지 관리
--       이미지를 먼저 업로드하고, 템플릿 생성/수정 시 이미지 ID로 연결

-- ==========================
-- 1. 독립적인 가이드 이미지 테이블
-- ==========================

CREATE TABLE IF NOT EXISTS guide_image (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    image_path TEXT NOT NULL UNIQUE,
    image_url TEXT NOT NULL,
    file_size BIGINT,
    mime_type TEXT,
    is_shared BOOLEAN NOT NULL DEFAULT true,
    uploaded_by INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE guide_image IS '템플릿과 독립적인 가이드 이미지 (재사용 가능)';
COMMENT ON COLUMN guide_image.image_path IS 'S3 파일 경로 (고유값)';
COMMENT ON COLUMN guide_image.image_url IS 'S3 접근 URL';
COMMENT ON COLUMN guide_image.is_shared IS '공유 여부 (true: 모든 사용자, false: 업로더만)';
COMMENT ON COLUMN guide_image.uploaded_by IS '업로드한 사용자 ID';

-- ==========================
-- 2. 템플릿-이미지 연결 테이블
-- ==========================

CREATE TABLE IF NOT EXISTS report_guide_template_image_mapping (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id INTEGER NOT NULL REFERENCES report_guide_template(id) ON DELETE CASCADE,
    image_id INTEGER NOT NULL REFERENCES guide_image(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_id, image_id)
);

COMMENT ON TABLE report_guide_template_image_mapping IS '원본 템플릿과 가이드 이미지 연결';
COMMENT ON COLUMN report_guide_template_image_mapping.display_order IS '이미지 표시 순서';

-- ==========================
-- 3. 커스텀 템플릿-이미지 연결 테이블
-- ==========================

CREATE TABLE IF NOT EXISTS user_custom_template_image_mapping (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    custom_template_id INTEGER NOT NULL REFERENCES user_custom_report_template(id) ON DELETE CASCADE,
    image_id INTEGER NOT NULL REFERENCES guide_image(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (custom_template_id, image_id)
);

COMMENT ON TABLE user_custom_template_image_mapping IS '커스텀 템플릿과 가이드 이미지 연결';
COMMENT ON COLUMN user_custom_template_image_mapping.display_order IS '이미지 표시 순서';

-- ==========================
-- 4. 인덱스 생성
-- ==========================

-- guide_image 인덱스
CREATE INDEX IF NOT EXISTS idx_guide_image_uploaded_by ON guide_image(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_guide_image_shared ON guide_image(is_shared) WHERE is_shared = true;
CREATE INDEX IF NOT EXISTS idx_guide_image_created_at ON guide_image(created_at DESC);

-- 템플릿-이미지 매핑 인덱스
CREATE INDEX IF NOT EXISTS idx_template_image_mapping_template ON report_guide_template_image_mapping(template_id);
CREATE INDEX IF NOT EXISTS idx_template_image_mapping_image ON report_guide_template_image_mapping(image_id);
CREATE INDEX IF NOT EXISTS idx_template_image_mapping_order ON report_guide_template_image_mapping(template_id, display_order);

-- 커스텀 템플릿-이미지 매핑 인덱스
CREATE INDEX IF NOT EXISTS idx_custom_template_image_mapping_template ON user_custom_template_image_mapping(custom_template_id);
CREATE INDEX IF NOT EXISTS idx_custom_template_image_mapping_image ON user_custom_template_image_mapping(image_id);
CREATE INDEX IF NOT EXISTS idx_custom_template_image_mapping_order ON user_custom_template_image_mapping(custom_template_id, display_order);

-- ==========================
-- 5. 데이터 마이그레이션 (기존 이미지 → 새 구조)
-- ==========================

-- 기존 report_guide_template_image 데이터를 guide_image로 복사
INSERT INTO guide_image (image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at)
SELECT DISTINCT ON (image_path)
    image_path,
    image_url,
    file_size,
    mime_type,
    is_shared,
    uploaded_by,
    created_at
FROM report_guide_template_image
ON CONFLICT (image_path) DO NOTHING;

-- 템플릿-이미지 매핑 생성
INSERT INTO report_guide_template_image_mapping (template_id, image_id, display_order, created_at)
SELECT 
    old_img.template_id,
    new_img.id,
    old_img.display_order,
    old_img.created_at
FROM report_guide_template_image old_img
JOIN guide_image new_img ON old_img.image_path = new_img.image_path
ON CONFLICT (template_id, image_id) DO NOTHING;

-- 기존 user_custom_template_image 데이터를 guide_image로 복사
INSERT INTO guide_image (image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at)
SELECT DISTINCT ON (image_path)
    image_path,
    image_url,
    file_size,
    mime_type,
    is_shared,
    uploaded_by,
    created_at
FROM user_custom_template_image
ON CONFLICT (image_path) DO NOTHING;

-- 커스텀 템플릿-이미지 매핑 생성
INSERT INTO user_custom_template_image_mapping (custom_template_id, image_id, display_order, created_at)
SELECT 
    old_img.custom_template_id,
    new_img.id,
    old_img.display_order,
    old_img.created_at
FROM user_custom_template_image old_img
JOIN guide_image new_img ON old_img.image_path = new_img.image_path
ON CONFLICT (custom_template_id, image_id) DO NOTHING;

-- ==========================
-- 6. 참고사항
-- ==========================

-- 기존 테이블 (report_guide_template_image, user_custom_template_image)은 
-- 하위 호환성을 위해 유지하되, 새로운 API는 guide_image를 사용합니다.
-- 향후 충분한 테스트 후 기존 테이블을 제거할 수 있습니다.

