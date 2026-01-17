-- 테스트 데이터 삽입
BEGIN;

-- Foreign Key 제약 임시 비활성화 (테스트 목적)
ALTER TABLE gc_deletion_log DROP CONSTRAINT IF EXISTS fk_annotation;

-- 실제 존재하는 annotation 찾기 또는 테스트 ID 사용
DO $$
DECLARE
    test_annotation_id INTEGER := 999999; -- 테스트용 임시 ID
BEGIN
    -- 첫 번째 annotation ID 가져오기 시도
    SELECT id INTO test_annotation_id
    FROM annotation_annotation
    ORDER BY id
    LIMIT 1;

    -- annotation이 없으면 테스트용 ID 사용
    IF test_annotation_id IS NULL THEN
        test_annotation_id := 999999;
        RAISE NOTICE '⚠️  No annotation found. Using test ID: %', test_annotation_id;
    ELSE
        RAISE NOTICE '✅ Found existing annotation ID: %', test_annotation_id;
    END IF;

    -- 1. 정상 삽입
    INSERT INTO gc_deletion_log (
        annotation_id, snapshot_image_key, file_size,
        dry_run, status, error_message
    )
    VALUES (test_annotation_id, 'test-key-1', 1024, true, 'success', NULL);

    -- 2. 실패 케이스 삽입
    INSERT INTO gc_deletion_log (
        annotation_id, snapshot_image_key, file_size,
        dry_run, status, error_message
    )
    VALUES (test_annotation_id, 'test-key-2', 2048, false, 'failed', 'S3 delete error');

    -- 3. 스킵 케이스 삽입
    INSERT INTO gc_deletion_log (
        annotation_id, snapshot_image_key, file_size,
        dry_run, status, error_message
    )
    VALUES (test_annotation_id, 'test-key-3', NULL, true, 'skipped', NULL);

    RAISE NOTICE '✅ Inserted 3 test records';
END $$;

-- 검증
SELECT '✅ Success count:' as label, COUNT(*) as count FROM gc_deletion_log WHERE status = 'success';
SELECT '❌ Failed count:' as label, COUNT(*) as count FROM gc_deletion_log WHERE status = 'failed';
SELECT '⏭️  Skipped count:' as label, COUNT(*) as count FROM gc_deletion_log WHERE status = 'skipped';

-- 전체 데이터 확인
SELECT '📋 All test records:' as label;
SELECT id, annotation_id, snapshot_image_key, file_size, dry_run, status, error_message, deleted_at
FROM gc_deletion_log
ORDER BY id DESC
LIMIT 10;

ROLLBACK;