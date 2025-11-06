-- ================================================
-- 015: Add Composite Index for User-Project Matrix Performance
-- ================================================
-- Description: 복합 인덱스 추가로 유저-프로젝트 매트릭스 조회 성능 향상
-- Date: 2024-12-01
-- 
-- 최적화 목적:
-- - User-Centered Matrix API의 배치 조회 쿼리 성능 향상
-- - (user_id, project_id) 조건의 WHERE 절 성능 개선
-- - 기존 개별 인덱스 대비 복합 인덱스 사용으로 조회 속도 향상
-- 
-- 기대 효과:
-- - 단일 조회 대비 10-20ms 추가 개선
-- - 데이터가 많을수록 효과 증가

-- Drop existing composite index if exists (to ensure clean migration)
DROP INDEX IF EXISTS idx_user_project_composite;

-- Create composite index for user-project membership queries
-- This index optimizes WHERE clauses with (user_id, project_id) conditions
CREATE INDEX idx_user_project_composite 
ON security_user_project(user_id, project_id);

-- Verify the index was created
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM pg_indexes 
        WHERE indexname = 'idx_user_project_composite'
    ) THEN
        RAISE NOTICE 'Index idx_user_project_composite created successfully';
    ELSE
        RAISE EXCEPTION 'Failed to create idx_user_project_composite';
    END IF;
END $$;

