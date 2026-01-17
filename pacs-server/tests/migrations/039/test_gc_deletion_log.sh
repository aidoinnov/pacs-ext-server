#!/bin/bash
# DB 마이그레이션 테스트

set -e
export PGPASSWORD="PacsExtension2024"

echo "🔍 Testing migration 039_create_gc_deletion_log.sql"

# 1. 마이그레이션 실행
# sqlx migrate run

# 2. 테이블 존재 확인
psql -h "localhost" -p "5456" -U pacs_extension_admin -d pacs_extension -c "\d gc_deletion_log" || {
    echo "❌ Table gc_deletion_log not found"
    exit 1
}

# 3. 컬럼 확인
psql -h "localhost" -p "5456" -U pacs_extension_admin -d pacs_extension -c "
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'gc_deletion_log'
ORDER BY ordinal_position;
" || exit 1

# 4. 인덱스 확인
psql -h "localhost" -p "5456" -U pacs_extension_admin -d pacs_extension -c "
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'gc_deletion_log';
" || exit 1

# 5. 제약 조건 확인
psql -h "localhost" -p "5456" -U pacs_extension_admin -d pacs_extension -c "
SELECT conname, contype, pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conrelid = 'gc_deletion_log'::regclass;
" || exit 1

echo "✅ Migration test passed"