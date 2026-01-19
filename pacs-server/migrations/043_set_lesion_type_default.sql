-- Migration: Set lesion_type default value to UNSPECIFIED
-- Created: 2026-01-19
-- Description:
--   1. annotation_annotation.lesion_type 컬럼에 기본값 'UNSPECIFIED' 설정
--   2. 기존 NULL 값을 'UNSPECIFIED'로 업데이트
--   3. NOT NULL 제약 조건 추가

-- ========================================
-- 1. 기존 NULL 값을 'UNSPECIFIED'로 업데이트
-- ========================================

UPDATE annotation_annotation
SET lesion_type = 'UNSPECIFIED'
WHERE lesion_type IS NULL;

-- ========================================
-- 2. 기본값 설정 및 NOT NULL 제약 조건 추가
-- ========================================

ALTER TABLE annotation_annotation
ALTER COLUMN lesion_type SET DEFAULT 'UNSPECIFIED',
ALTER COLUMN lesion_type SET NOT NULL;

-- ========================================
-- 3. 주석 업데이트
-- ========================================

COMMENT ON COLUMN annotation_annotation.lesion_type IS 'Lesion 타입 (TARGET/NON_TARGET/TARGET_NEW/NON_TARGET_NEW/UNSPECIFIED) - 기본값: UNSPECIFIED';

-- ========================================
-- VALIDATION QUERIES (for testing)
-- ========================================

-- 1. lesion_type 분포 확인
-- SELECT lesion_type, COUNT(*) as count
-- FROM annotation_annotation
-- GROUP BY lesion_type
-- ORDER BY count DESC;

-- 2. NULL 값이 남아있는지 확인 (0이어야 함)
-- SELECT COUNT(*) as null_count
-- FROM annotation_annotation
-- WHERE lesion_type IS NULL;

-- 3. 기본값 확인
-- SELECT column_name, column_default, is_nullable
-- FROM information_schema.columns
-- WHERE table_name = 'annotation_annotation'
--   AND column_name = 'lesion_type';

