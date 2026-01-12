-- Migration: Fix snapshot_uploaded_at column type
-- Created: 2026-01-12
-- Description: TIMESTAMP를 TIMESTAMPTZ로 변경하여 다른 날짜 컬럼과 일관성 유지

-- Step 1: 기존 컬럼 타입 변경
ALTER TABLE annotation_annotation
ALTER COLUMN snapshot_uploaded_at TYPE TIMESTAMPTZ;

-- Step 2: 컬럼 주석 업데이트
COMMENT ON COLUMN annotation_annotation.snapshot_uploaded_at IS '스냅샷 업로드 완료 시간 (UTC)';

