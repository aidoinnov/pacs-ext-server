-- Migration: Add snapshot image support to annotations
-- Created: 2026-01-11
-- Description: S3에 저장된 어노테이션 스냅샷 이미지 경로 및 상태를 저장하기 위한 컬럼 추가

-- Step 1: 스냅샷 상태 ENUM 타입 생성 (이미 존재하면 무시)
DO $$ BEGIN
    CREATE TYPE snapshot_upload_status AS ENUM (
        'pending',      -- URL 생성됨, 업로드 대기 중
        'uploading',    -- 업로드 진행 중
        'completed',    -- 업로드 완료
        'failed'        -- 업로드 실패
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Step 2: 스냅샷 관련 컬럼 추가
ALTER TABLE annotation_annotation
ADD COLUMN IF NOT EXISTS snapshot_image_key VARCHAR(512) NULL,
ADD COLUMN IF NOT EXISTS snapshot_status snapshot_upload_status NULL DEFAULT NULL,
ADD COLUMN IF NOT EXISTS snapshot_uploaded_at TIMESTAMPTZ NULL;

-- Step 3: 컬럼 주석 추가
COMMENT ON COLUMN annotation_annotation.snapshot_image_key IS 'S3에 저장된 스냅샷 이미지의 object key';
COMMENT ON COLUMN annotation_annotation.snapshot_status IS '스냅샷 업로드 상태 (pending/uploading/completed/failed)';
COMMENT ON COLUMN annotation_annotation.snapshot_uploaded_at IS '스냅샷 업로드 완료 시간';

-- Step 4: 인덱스 추가 (이미지가 있는 어노테이션 조회 최적화)
CREATE INDEX IF NOT EXISTS idx_annotation_snapshot_image_key
ON annotation_annotation(snapshot_image_key)
WHERE snapshot_image_key IS NOT NULL;

-- Step 5: 인덱스 추가 (업로드 상태별 조회 최적화)
CREATE INDEX IF NOT EXISTS idx_annotation_snapshot_status
ON annotation_annotation(snapshot_status)
WHERE snapshot_status IS NOT NULL;