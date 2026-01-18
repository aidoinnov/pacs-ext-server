-- Migration: RECIST Lesion Management for Target Lesion Tracking
-- Created: 2026-01-18
-- Description: Creates tables for managing RECIST lesions (Target, Non-target, New)
--              Supports RECIST 1.1 criteria with Baseline reference tracking
--              Links Annotations to Lesions across TimePoints

-- ========================================
-- ENUMS
-- ========================================

-- RECIST Lesion Type
CREATE TYPE recist_lesion_type_enum AS ENUM ('TARGET', 'NON_TARGET', 'NEW');

-- ========================================
-- TABLES
-- ========================================

-- RECIST Lesion 관리
CREATE TABLE recist_lesion (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES project_subject(id) ON DELETE CASCADE,
    lesion_type recist_lesion_type_enum NOT NULL,
    lesion_number INTEGER NOT NULL,              -- Subject 내 병변 번호 (1, 2, 3, ...)
    baseline_timepoint_id INTEGER REFERENCES subject_timepoint(id) ON DELETE SET NULL,
    organ_site VARCHAR(100),                     -- 장기 위치 (Liver, Lung, ...)
    description TEXT,                            -- 병변 설명
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT uq_subject_lesion_number UNIQUE (subject_id, lesion_number),
    
    -- TARGET/NON_TARGET은 baseline_timepoint_id 필수, NEW는 NULL
    CONSTRAINT ck_baseline_required 
        CHECK (
            (lesion_type IN ('TARGET', 'NON_TARGET') AND baseline_timepoint_id IS NOT NULL)
            OR (lesion_type = 'NEW' AND baseline_timepoint_id IS NULL)
        )
);

-- Lesion ↔ Annotation 매핑 (TimePoint별 측정값 추적)
CREATE TABLE recist_lesion_annotation_map (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    lesion_id INTEGER NOT NULL REFERENCES recist_lesion(id) ON DELETE CASCADE,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    timepoint_id INTEGER NOT NULL REFERENCES subject_timepoint(id) ON DELETE CASCADE,
    measured_length_mm FLOAT,                    -- 측정된 길이 (mm)
    measured_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT uq_lesion_annotation UNIQUE (lesion_id, annotation_id),
    CONSTRAINT uq_annotation_timepoint UNIQUE (annotation_id, timepoint_id)
);

-- ========================================
-- INDEXES
-- ========================================

-- recist_lesion 인덱스
CREATE INDEX idx_recist_lesion_subject ON recist_lesion(subject_id);
CREATE INDEX idx_recist_lesion_project ON recist_lesion(project_id);
CREATE INDEX idx_recist_lesion_type ON recist_lesion(lesion_type);
CREATE INDEX idx_recist_lesion_baseline ON recist_lesion(baseline_timepoint_id) 
    WHERE baseline_timepoint_id IS NOT NULL;

-- recist_lesion_annotation_map 인덱스
CREATE INDEX idx_lesion_annotation_map_lesion ON recist_lesion_annotation_map(lesion_id);
CREATE INDEX idx_lesion_annotation_map_annotation ON recist_lesion_annotation_map(annotation_id);
CREATE INDEX idx_lesion_annotation_map_timepoint ON recist_lesion_annotation_map(timepoint_id);

-- ========================================
-- TRIGGERS
-- ========================================

-- recist_lesion updated_at 트리거
CREATE OR REPLACE FUNCTION update_recist_lesion_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_recist_lesion_updated_at
BEFORE UPDATE ON recist_lesion
FOR EACH ROW
EXECUTE FUNCTION update_recist_lesion_updated_at();

-- ========================================
-- COMMENTS
-- ========================================

-- recist_lesion 주석
COMMENT ON TABLE recist_lesion IS 'RECIST 병변 관리 - Target/Non-target/New Lesion';
COMMENT ON COLUMN recist_lesion.lesion_type IS 'Lesion 타입 (TARGET/NON_TARGET/NEW)';
COMMENT ON COLUMN recist_lesion.lesion_number IS 'Subject 내 병변 순번 (1, 2, 3, ...)';
COMMENT ON COLUMN recist_lesion.baseline_timepoint_id IS 'Baseline TimePoint 참조 (TARGET/NON_TARGET 필수)';
COMMENT ON COLUMN recist_lesion.organ_site IS '병변 위치 (Liver, Lung, Lymph Node 등)';
COMMENT ON COLUMN recist_lesion.description IS '병변 상세 설명';

-- recist_lesion_annotation_map 주석
COMMENT ON TABLE recist_lesion_annotation_map IS 'Lesion ↔ Annotation 매핑 - TimePoint별 측정값 추적';
COMMENT ON COLUMN recist_lesion_annotation_map.measured_length_mm IS '측정된 병변 길이 (mm)';
COMMENT ON COLUMN recist_lesion_annotation_map.measured_at IS '측정 시각';

-- ========================================
-- VALIDATION QUERIES (for testing)
-- ========================================

-- 1. Baseline 제약 확인 (TARGET/NON_TARGET은 baseline_timepoint_id 필수)
-- SELECT id, lesion_type, baseline_timepoint_id
-- FROM recist_lesion
-- WHERE (lesion_type IN ('TARGET', 'NON_TARGET') AND baseline_timepoint_id IS NULL)
--    OR (lesion_type = 'NEW' AND baseline_timepoint_id IS NOT NULL);

-- 2. Lesion Number 중복 확인
-- SELECT subject_id, lesion_number, COUNT(*)
-- FROM recist_lesion
-- GROUP BY subject_id, lesion_number
-- HAVING COUNT(*) > 1;

-- 3. TimePoint별 Lesion 측정값 조회
-- SELECT 
--     l.id AS lesion_id,
--     l.lesion_number,
--     l.lesion_type,
--     l.organ_site,
--     t.name AS timepoint_name,
--     m.measured_length_mm,
--     m.measured_at
-- FROM recist_lesion l
-- INNER JOIN recist_lesion_annotation_map m ON m.lesion_id = l.id
-- INNER JOIN subject_timepoint t ON t.id = m.timepoint_id
-- WHERE l.subject_id = :subject_id
-- ORDER BY l.lesion_number, t.order_index;

