#!/bin/bash
# DB 마이그레이션 테스트

set -e
export PGPASSWORD="PacsExtension2024"

psql -h localhost -p 5456 -U pacs_extension_admin -d pacs_extension -f tests/migrations/039/test_gc_log_insert.sql || {
    echo "❌ GC log insert test failed"
    exit 1
}

echo "✅ GC log insert test passed"