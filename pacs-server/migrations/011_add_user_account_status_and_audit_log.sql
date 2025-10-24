-- Migration: 011_add_user_account_status_and_audit_log.sql
-- 사용자 계정 상태 및 감사 로그 테이블 추가

-- 사용자 계정 상태 열거형 생성 (이미 존재할 수 있으므로 IF NOT EXISTS 사용)
DO $$ BEGIN
    CREATE TYPE user_account_status_enum AS ENUM (
        'PENDING_EMAIL',     -- 이메일 인증 대기
        'PENDING_APPROVAL',  -- 관리자 승인 대기
        'ACTIVE',            -- 활성
        'SUSPENDED',         -- 정지
        'DELETED'            -- 삭제됨
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- security_user 테이블에 계정 상태 관련 필드 추가 (이미 존재할 수 있으므로 IF NOT EXISTS 사용)
DO $$ BEGIN
    ALTER TABLE security_user
    ADD COLUMN account_status user_account_status_enum NOT NULL DEFAULT 'PENDING_EMAIL',
    ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN email_verification_token TEXT,
    ADD COLUMN email_verification_expires_at TIMESTAMPTZ,
    ADD COLUMN approved_by INTEGER REFERENCES security_user(id),
    ADD COLUMN approved_at TIMESTAMPTZ,
    ADD COLUMN suspended_at TIMESTAMPTZ,
    ADD COLUMN suspended_reason TEXT,
    ADD COLUMN deleted_at TIMESTAMPTZ;
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

-- 사용자 계정 감사 로그 테이블 생성 (이미 존재할 수 있으므로 IF NOT EXISTS 사용)
CREATE TABLE IF NOT EXISTS security_user_audit_log (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id INTEGER,  -- NULL 허용 (사용자 삭제 후에도 로그 유지)
    action TEXT NOT NULL,  -- 'SIGNUP_REQUESTED', 'EMAIL_VERIFIED', 'APPROVED', 'LOGIN', 'DELETED' 등
    actor_id INTEGER REFERENCES security_user(id),  -- 작업 수행자 (NULL 허용)
    keycloak_sync_status TEXT,  -- 'SUCCESS', 'FAILED', 'PENDING', 'ROLLED_BACK'
    keycloak_user_id TEXT,
    error_message TEXT,
    metadata JSONB,  -- 추가 정보 (IP, user agent, 요청 데이터 등)
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 성능 최적화를 위한 인덱스 생성
CREATE INDEX idx_user_audit_log_user_id ON security_user_audit_log(user_id);
CREATE INDEX idx_user_audit_log_action ON security_user_audit_log(action);
CREATE INDEX idx_user_audit_log_created_at ON security_user_audit_log(created_at);
CREATE INDEX idx_user_account_status ON security_user(account_status);
CREATE INDEX idx_user_email_verified ON security_user(email_verified);
CREATE INDEX idx_user_approved_by ON security_user(approved_by);

-- 테이블 및 컬럼에 대한 주석 추가
COMMENT ON TABLE security_user_audit_log IS '사용자 계정 감사 로그 - 사용자 삭제 후에도 영구 보관';
COMMENT ON COLUMN security_user_audit_log.user_id IS '사용자 ID (삭제 후에도 NULL이 아닌 ID 유지)';
COMMENT ON COLUMN security_user_audit_log.action IS '수행된 작업 (SIGNUP_REQUESTED, EMAIL_VERIFIED, APPROVED, DELETED 등)';
COMMENT ON COLUMN security_user_audit_log.actor_id IS '작업을 수행한 사용자 ID (시스템 작업의 경우 NULL)';
COMMENT ON COLUMN security_user_audit_log.keycloak_sync_status IS 'Keycloak 동기화 상태 (SUCCESS, FAILED, PENDING, ROLLED_BACK)';
COMMENT ON COLUMN security_user_audit_log.keycloak_user_id IS 'Keycloak에서의 사용자 ID';
COMMENT ON COLUMN security_user_audit_log.error_message IS '오류 발생 시 오류 메시지';
COMMENT ON COLUMN security_user_audit_log.metadata IS '추가 메타데이터 (IP, User-Agent, 요청 데이터 등)';

COMMENT ON COLUMN security_user.account_status IS '사용자 계정 상태';
COMMENT ON COLUMN security_user.email_verified IS '이메일 인증 완료 여부';
COMMENT ON COLUMN security_user.email_verification_token IS '이메일 인증 토큰';
COMMENT ON COLUMN security_user.email_verification_expires_at IS '이메일 인증 토큰 만료 시간';
COMMENT ON COLUMN security_user.approved_by IS '승인한 관리자 ID';
COMMENT ON COLUMN security_user.approved_at IS '승인 시간';
COMMENT ON COLUMN security_user.suspended_at IS '정지 시간';
COMMENT ON COLUMN security_user.suspended_reason IS '정지 사유';
COMMENT ON COLUMN security_user.deleted_at IS '삭제 시간';
