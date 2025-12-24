-- Migration: Create series_user_note table
-- Created: 2025-01-15
-- Description: Creates table for user-specific notes on DICOM Series
-- Supports both project-scoped and global notes

CREATE TABLE IF NOT EXISTS series_user_note (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES project_data_series(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES security_user(id) ON DELETE CASCADE,
    project_id INTEGER NULL REFERENCES security_project(id) ON DELETE CASCADE,
    note TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 한 사용자는 한 Series에 대해:
    -- - 전역 note 1개 (project_id = NULL)
    -- - 프로젝트별 note 여러 개 (project_id = 1, 2, 3...)
    UNIQUE (series_id, user_id, project_id)
);

-- 인덱스: Series 조회 최적화
CREATE INDEX IF NOT EXISTS idx_series_user_note_series ON series_user_note(series_id);

-- 인덱스: 사용자별 note 조회 최적화
CREATE INDEX IF NOT EXISTS idx_series_user_note_user ON series_user_note(user_id);

-- 인덱스: 프로젝트별 note 조회 최적화 (NULL 값 제외)
CREATE INDEX IF NOT EXISTS idx_series_user_note_project ON series_user_note(project_id) WHERE project_id IS NOT NULL;

-- 인덱스: Series-User 조합 조회 최적화
CREATE INDEX IF NOT EXISTS idx_series_user_note_series_user ON series_user_note(series_id, user_id);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_series_user_note_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_series_user_note_updated_at
    BEFORE UPDATE ON series_user_note
    FOR EACH ROW
    EXECUTE FUNCTION update_series_user_note_updated_at();

-- 코멘트 추가
COMMENT ON TABLE series_user_note IS '사용자별 DICOM Series 메모 (프로젝트 종속 또는 전역)';
COMMENT ON COLUMN series_user_note.series_id IS 'Series ID (project_data_series 참조)';
COMMENT ON COLUMN series_user_note.user_id IS '사용자 ID (security_user 참조)';
COMMENT ON COLUMN series_user_note.project_id IS '프로젝트 ID (NULL이면 전역 note, 값이 있으면 프로젝트별 note)';
COMMENT ON COLUMN series_user_note.note IS '사용자가 작성한 텍스트 메모';

