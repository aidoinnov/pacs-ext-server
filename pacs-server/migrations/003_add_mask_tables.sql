-- Migration: Add mask tables for PACS mask upload functionality
-- Created: 2025-10-07
-- Description: Creates annotation_mask_group and annotation_mask tables for storing segmentation masks

-- annotation_mask_group 테이블 생성
-- 어노테이션에 연결된 마스크 그룹을 관리하는 테이블
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    group_name TEXT,                       -- 마스크 그룹 이름 (예: Liver_Segmentation_v2)
    model_name TEXT,                       -- AI 모델명 (optional)
    version TEXT,                          -- 버전명 (optional)
    modality TEXT,                         -- CT/MR 등 의료 영상 모달리티
    slice_count INTEGER DEFAULT 1,        -- 슬라이스 수
    mask_type TEXT DEFAULT 'segmentation', -- 마스크 타입 (segmentation, detection 등)
    description TEXT,                      -- 그룹 설명
    created_by INTEGER,                    -- 생성자 ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- annotation_mask 테이블 생성
-- 각 슬라이스별 또는 라벨별 마스크 파일 정보를 저장하는 테이블
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id) ON DELETE CASCADE,
    slice_index INTEGER,                   -- 볼륨 내 슬라이스 인덱스
    sop_instance_uid TEXT,                 -- SOP Instance UID (DICOM 표준)
    label_name TEXT,                       -- 라벨 이름 (예: liver, spleen)
    file_path TEXT NOT NULL,               -- S3/스토리지 파일 경로
    mime_type TEXT DEFAULT 'image/png',    -- 파일 MIME 타입
    file_size BIGINT,                      -- 파일 크기 (바이트)
    checksum TEXT,                         -- 파일 체크섬 (무결성 검증용)
    width INTEGER,                         -- 이미지 너비
    height INTEGER,                        -- 이미지 높이
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 인덱스 생성
-- 성능 최적화를 위한 인덱스들
CREATE INDEX idx_mask_group_annotation_id ON annotation_mask_group(annotation_id);
CREATE INDEX idx_mask_mask_group_id ON annotation_mask(mask_group_id);
CREATE INDEX idx_mask_sop_instance_uid ON annotation_mask(sop_instance_uid);
CREATE INDEX idx_mask_label_name ON annotation_mask(label_name);

-- 코멘트 추가
COMMENT ON TABLE annotation_mask_group IS '어노테이션에 연결된 마스크 그룹을 관리하는 테이블';
COMMENT ON TABLE annotation_mask IS '각 슬라이스별 또는 라벨별 마스크 파일 정보를 저장하는 테이블';

COMMENT ON COLUMN annotation_mask_group.group_name IS '마스크 그룹 이름 (예: Liver_Segmentation_v2)';
COMMENT ON COLUMN annotation_mask_group.model_name IS 'AI 모델명 (optional)';
COMMENT ON COLUMN annotation_mask_group.version IS '버전명 (optional)';
COMMENT ON COLUMN annotation_mask_group.modality IS 'CT/MR 등 의료 영상 모달리티';
COMMENT ON COLUMN annotation_mask_group.slice_count IS '슬라이스 수';
COMMENT ON COLUMN annotation_mask_group.mask_type IS '마스크 타입 (segmentation, detection 등)';
COMMENT ON COLUMN annotation_mask_group.description IS '그룹 설명';
COMMENT ON COLUMN annotation_mask_group.created_by IS '생성자 ID';

COMMENT ON COLUMN annotation_mask.slice_index IS '볼륨 내 슬라이스 인덱스';
COMMENT ON COLUMN annotation_mask.sop_instance_uid IS 'SOP Instance UID (DICOM 표준)';
COMMENT ON COLUMN annotation_mask.label_name IS '라벨 이름 (예: liver, spleen)';
COMMENT ON COLUMN annotation_mask.file_path IS 'S3/스토리지 파일 경로';
COMMENT ON COLUMN annotation_mask.mime_type IS '파일 MIME 타입';
COMMENT ON COLUMN annotation_mask.file_size IS '파일 크기 (바이트)';
COMMENT ON COLUMN annotation_mask.checksum IS '파일 체크섬 (무결성 검증용)';
COMMENT ON COLUMN annotation_mask.width IS '이미지 너비';
COMMENT ON COLUMN annotation_mask.height IS '이미지 높이';
