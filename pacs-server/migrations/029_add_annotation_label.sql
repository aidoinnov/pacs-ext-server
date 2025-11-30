-- Migration: Add Label Field to Annotation Table
-- Created: 2025-11-12
-- Description: Add label field to annotation_annotation table for storing annotation labels

-- annotation_annotation 테이블에 label 필드 추가
ALTER TABLE annotation_annotation 
ADD COLUMN label TEXT DEFAULT '';

-- 컬럼 설명 추가
COMMENT ON COLUMN annotation_annotation.label IS '어노테이션 라벨 (예: Tumor, Lesion, Normal)';

-- 인덱스 추가 (라벨로 검색할 경우를 대비)
CREATE INDEX idx_annotation_label ON annotation_annotation(label);

