-- ============================================================================
-- Migration 034: Add display_label to study_list_view_field
-- ============================================================================
-- 
-- 목적: View 필드에 사용자 정의 라벨(display_label) 추가
-- - 기본값은 NULL (원본 label 사용)
-- - 설정 시 원본 label 대신 display_label 표시
--
-- ============================================================================

-- display_label 컬럼 추가
ALTER TABLE study_list_view_field
ADD COLUMN IF NOT EXISTS display_label VARCHAR(255) NULL;

-- 코멘트 추가
COMMENT ON COLUMN study_list_view_field.display_label IS 'Custom display label (NULL = use original field label)';

