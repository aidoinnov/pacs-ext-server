-- Migration: Add updated_at columns to mask tables
-- Created: 2025-01-15
-- Description: Adds updated_at columns to annotation_mask_group and annotation_mask tables

-- annotation_mask_group 테이블에 updated_at 컬럼 추가
ALTER TABLE annotation_mask_group 
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- annotation_mask 테이블에 updated_at 컬럼 추가
ALTER TABLE annotation_mask 
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- 기존 레코드들의 updated_at을 created_at과 동일하게 설정
UPDATE annotation_mask_group 
SET updated_at = created_at 
WHERE updated_at IS NULL;

UPDATE annotation_mask 
SET updated_at = created_at 
WHERE updated_at IS NULL;

-- updated_at 컬럼을 NOT NULL로 변경
ALTER TABLE annotation_mask_group 
ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE annotation_mask 
ALTER COLUMN updated_at SET NOT NULL;
