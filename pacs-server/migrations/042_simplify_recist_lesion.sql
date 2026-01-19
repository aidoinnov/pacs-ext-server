-- Migration: Simplify RECIST Lesion Structure (방안 2: 하이브리드)
-- Created: 2026-01-18
-- Description:
--   1. annotation_annotation에 lesion_type, lesion_number 추가 (사용자 입력)
--   2. recist_lesion 간소화 (서버 자동 관리)
--   3. 매핑 테이블 제거 (recist_lesion_annotation_map)
--   4. 불필요한 컬럼 제거 (organ_site, baseline_timepoint_id, project_id)

-- ========================================
-- 1. annotation_annotation에 lesion 정보 추가
-- ========================================

ALTER TABLE annotation_annotation
ADD COLUMN IF NOT EXISTS lesion_type VARCHAR(20),
ADD COLUMN IF NOT EXISTS lesion_number INTEGER;

-- ========================================
-- 2. 기존 매핑 데이터 이전
-- ========================================

-- recist_lesion_annotation_map 데이터를 annotation_annotation로 이전
UPDATE annotation_annotation a
SET
    lesion_type = l.lesion_type::text,
    lesion_number = l.lesion_number
FROM recist_lesion_annotation_map m
JOIN recist_lesion l ON l.id = m.lesion_id
WHERE a.id = m.annotation_id
  AND a.lesion_type IS NULL;

-- ========================================
-- 3. 매핑 테이블 삭제
-- ========================================

DROP TABLE IF EXISTS recist_lesion_annotation_map;

-- ========================================
-- 4. recist_lesion 불필요한 컬럼 삭제
-- ========================================

-- organ_site 삭제 (description으로 충분)
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS organ_site;

-- baseline_timepoint_id 삭제 (lesion_type으로 충분)
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS baseline_timepoint_id;

-- project_id 삭제 (subject_id에서 조회 가능)
ALTER TABLE recist_lesion DROP COLUMN IF EXISTS project_id;

-- ========================================
-- 5. recist_lesion 제약 조건 변경
-- ========================================

-- 기존 제약 조건 삭제
ALTER TABLE recist_lesion DROP CONSTRAINT IF EXISTS uq_subject_lesion_number;
ALTER TABLE recist_lesion DROP CONSTRAINT IF EXISTS ck_baseline_required;

-- 새 제약 조건: (subject_id, lesion_type, lesion_number) 유일
ALTER TABLE recist_lesion
ADD CONSTRAINT uq_subject_lesion UNIQUE (subject_id, lesion_type, lesion_number);

-- ========================================
-- 6. lesion_type을 VARCHAR로 변경 (ENUM 제거)
-- ========================================

-- ENUM에서 VARCHAR로 변경
ALTER TABLE recist_lesion
ALTER COLUMN lesion_type TYPE VARCHAR(20);

-- ========================================
-- 7. 인덱스 추가
-- ========================================

-- annotation_annotation.lesion_type 인덱스
CREATE INDEX IF NOT EXISTS idx_annotation_lesion_type
ON annotation_annotation(lesion_type)
WHERE lesion_type IS NOT NULL;

-- recist_lesion.subject_id 인덱스
CREATE INDEX IF NOT EXISTS idx_recist_lesion_subject
ON recist_lesion(subject_id);

-- ========================================
-- 8. 주석 업데이트
-- ========================================

COMMENT ON TABLE recist_lesion IS 'RECIST 병변 추적 (서버 자동 관리) - Subject별 Lesion 분석/추적용';
COMMENT ON COLUMN recist_lesion.lesion_type IS 'Lesion 타입 (TARGET/NON_TARGET/TARGET_NEW/NON_TARGET_NEW)';
COMMENT ON COLUMN recist_lesion.lesion_number IS 'Lesion 타입별 순번 (1, 2, 3, ...)';
COMMENT ON COLUMN recist_lesion.description IS '병변 설명 (선택사항)';

COMMENT ON COLUMN annotation_annotation.lesion_type IS 'Lesion 타입 (사용자 입력)';
COMMENT ON COLUMN annotation_annotation.lesion_number IS 'Lesion 번호 (사용자 입력)';

-- ========================================
-- 9. 제약 조건 확인
-- ========================================

-- Subject별 (lesion_type, lesion_number) 유일성 확인
DO $$
DECLARE
    duplicate_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO duplicate_count
    FROM (
        SELECT subject_id, lesion_type, lesion_number, COUNT(*)
        FROM recist_lesion
        GROUP BY subject_id, lesion_type, lesion_number
        HAVING COUNT(*) > 1
    ) duplicates;

    IF duplicate_count > 0 THEN
        RAISE WARNING 'Found % duplicate (lesion_type, lesion_number) entries. Please fix manually.', duplicate_count;
    END IF;
END $$;

-- ========================================
-- VALIDATION QUERIES (for testing)
-- ========================================

-- 1. Annotation의 lesion 정보 확인
-- SELECT id, study_uid, lesion_type, lesion_number, label
-- FROM annotation_annotation
-- WHERE lesion_type IS NOT NULL
-- ORDER BY lesion_type, lesion_number;

-- 2. recist_lesion 테이블 확인
-- SELECT id, subject_id, lesion_type, lesion_number, description
-- FROM recist_lesion
-- ORDER BY subject_id, lesion_type, lesion_number;

-- 3. Subject 1의 Target Lesion 1 추적
-- SELECT
--     tp.name AS timepoint,
--     tp.order_index,
--     a.id AS annotation_id,
--     a.lesion_type,
--     a.lesion_number,
--     a.measurement_values->0->'values'->0 AS size_mm,
--     a.created_at
-- FROM annotation_annotation a
-- JOIN subject_timepoint_study_map m ON m.study_uid = a.study_uid
-- JOIN subject_timepoint tp ON tp.id = m.timepoint_id
-- WHERE m.subject_id = 1
--   AND a.lesion_type = 'TARGET'
--   AND a.lesion_number = 1
-- ORDER BY tp.order_index;

-- 4. Subject의 모든 Lesion 요약
-- SELECT
--     lesion_type,
--     lesion_number,
--     description,
--     created_at,
--     (SELECT COUNT(*) FROM annotation_annotation a
--      JOIN subject_timepoint_study_map m ON m.study_uid = a.study_uid
--      WHERE m.subject_id = l.subject_id
--        AND a.lesion_type = l.lesion_type
--        AND a.lesion_number = l.lesion_number) AS annotation_count
-- FROM recist_lesion l
-- WHERE subject_id = 1
-- ORDER BY lesion_type, lesion_number;

