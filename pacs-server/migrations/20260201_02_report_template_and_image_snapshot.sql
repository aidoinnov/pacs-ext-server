-- ==========================
-- Report 1:1 Template + Image Snapshot
-- ==========================
-- 작성일: 2026-02-01
-- 선행: 20260201_01_create_guide_image_tables.sql (guide_image 필요)
-- 목적: 리포트당 템플릿 1개, 이미지 스냅샷으로 템플릿 변경과 무관하게 유지
--       series_user_report_guide 폐기, series_user_report에 template 직접 저장

-- ==========================
-- 1. series_user_report 컬럼 추가
-- ==========================

ALTER TABLE series_user_report
    ADD COLUMN IF NOT EXISTS template_id INTEGER NULL REFERENCES report_guide_template(id) ON DELETE SET NULL;

ALTER TABLE series_user_report
    ADD COLUMN IF NOT EXISTS custom_template_id INTEGER NULL REFERENCES user_custom_report_template(id) ON DELETE SET NULL;

-- template_id, custom_template_id 둘 중 하나만 또는 둘 다 NULL
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_report_template_exclusive'
    ) THEN
        ALTER TABLE series_user_report
            ADD CONSTRAINT chk_report_template_exclusive CHECK (
                (template_id IS NOT NULL AND custom_template_id IS NULL) OR
                (template_id IS NULL AND custom_template_id IS NOT NULL) OR
                (template_id IS NULL AND custom_template_id IS NULL)
            );
    END IF;
END $$;

COMMENT ON COLUMN series_user_report.template_id IS '적용된 원본 템플릿 ID (출처용)';
COMMENT ON COLUMN series_user_report.custom_template_id IS '적용된 커스텀 템플릿 ID (출처용)';

-- ==========================
-- 2. report_image 테이블 생성
-- ==========================

CREATE TABLE IF NOT EXISTS report_image (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES series_user_report(id) ON DELETE CASCADE,
    image_id INTEGER NOT NULL REFERENCES guide_image(id) ON DELETE RESTRICT,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (report_id, image_id)
);

CREATE INDEX IF NOT EXISTS idx_report_image_report ON report_image(report_id);
CREATE INDEX IF NOT EXISTS idx_report_image_order ON report_image(report_id, display_order);

COMMENT ON TABLE report_image IS '리포트의 가이드 이미지 스냅샷 (템플릿 변경과 무관하게 유지)';

-- ==========================
-- 3. 기존 데이터 마이그레이션 (series_user_report_guide → report)
-- ==========================

-- 3.1 report에 template_id, custom_template_id 설정 (첫 번째 가이드 기준)
UPDATE series_user_report r
SET
    template_id = g.template_id,
    custom_template_id = g.custom_template_id
FROM (
    SELECT DISTINCT ON (report_id) report_id, template_id, custom_template_id
    FROM series_user_report_guide
    ORDER BY report_id, display_order, created_at
) g
WHERE r.id = g.report_id;

-- 3.2 report_image 삽입 (report_guide_template_image_mapping 사용)
INSERT INTO report_image (report_id, image_id, display_order, created_at)
SELECT rg.report_id, m.image_id, m.display_order, NOW()
FROM series_user_report_guide rg
JOIN report_guide_template_image_mapping m ON m.template_id = rg.template_id
WHERE rg.template_id IS NOT NULL
ON CONFLICT (report_id, image_id) DO NOTHING;

-- 3.3 report_image 삽입 (user_custom_template_image_mapping 사용)
INSERT INTO report_image (report_id, image_id, display_order, created_at)
SELECT rg.report_id, m.image_id, m.display_order, NOW()
FROM series_user_report_guide rg
JOIN user_custom_template_image_mapping m ON m.custom_template_id = rg.custom_template_id
WHERE rg.custom_template_id IS NOT NULL
ON CONFLICT (report_id, image_id) DO NOTHING;

-- ==========================
-- 4. series_user_report_guide 테이블 삭제
-- ==========================

DROP TABLE IF EXISTS series_user_report_guide CASCADE;
