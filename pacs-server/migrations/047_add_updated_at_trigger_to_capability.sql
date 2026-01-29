-- Migration 047: Add updated_at trigger to security_capability table
-- 
-- 목적: Capability 메타데이터 변경 시 updated_at 자동 갱신
-- 이유: 현재는 Capability 메타데이터(name, description 등) 변경 시 
--       updated_at이 자동으로 갱신되지 않아 ETag 캐싱이 제대로 동작하지 않음

-- 1. 트리거 함수 생성
CREATE OR REPLACE FUNCTION update_security_capability_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 2. 트리거 생성
CREATE TRIGGER trigger_update_security_capability_updated_at
    BEFORE UPDATE ON security_capability
    FOR EACH ROW
    EXECUTE FUNCTION update_security_capability_updated_at();

-- 3. 기존 데이터의 updated_at을 created_at으로 초기화 (이미 설정되어 있으면 스킵)
-- (security_capability 테이블에는 created_at 컬럼이 없으므로 현재 시간으로 설정)
UPDATE security_capability 
SET updated_at = COALESCE(updated_at, CURRENT_TIMESTAMP)
WHERE updated_at IS NULL;

