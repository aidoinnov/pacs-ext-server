-- GC 삭제 로그 테이블 생성
CREATE TABLE IF NOT EXISTS gc_deletion_log (
    id BIGSERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL,
    snapshot_image_key TEXT NOT NULL,
    file_size BIGINT,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    dry_run BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed', 'skipped')),
    error_message TEXT,
    
    -- 인덱스
    CONSTRAINT fk_annotation FOREIGN KEY (annotation_id) 
        REFERENCES annotation_annotation(id) ON DELETE CASCADE
);

-- 인덱스 생성
CREATE INDEX idx_gc_deletion_log_annotation_id ON gc_deletion_log(annotation_id);
CREATE INDEX idx_gc_deletion_log_deleted_at ON gc_deletion_log(deleted_at);
CREATE INDEX idx_gc_deletion_log_status ON gc_deletion_log(status);

-- 코멘트 추가
COMMENT ON TABLE gc_deletion_log IS 'GC 작업으로 삭제된 스냅샷 이미지 로그';
COMMENT ON COLUMN gc_deletion_log.annotation_id IS '삭제된 스냅샷의 어노테이션 ID';
COMMENT ON COLUMN gc_deletion_log.snapshot_image_key IS '삭제된 S3 object key';
COMMENT ON COLUMN gc_deletion_log.file_size IS '삭제된 파일 크기 (바이트)';
COMMENT ON COLUMN gc_deletion_log.deleted_at IS '삭제 시각';
COMMENT ON COLUMN gc_deletion_log.dry_run IS 'Dry-run 모드 여부';
COMMENT ON COLUMN gc_deletion_log.status IS '삭제 상태 (success, failed, skipped)';
COMMENT ON COLUMN gc_deletion_log.error_message IS '실패 시 에러 메시지';