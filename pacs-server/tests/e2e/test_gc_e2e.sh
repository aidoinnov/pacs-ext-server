#!/bin/bash
# GC Runner E2E 테스트
# 실제 DB에 테스트 데이터를 생성하고 GC Runner를 실행하여 검증

set -e

echo "🧪 GC Runner E2E Test"
echo "====================="

# 환경 변수 설정
export DATABASE_URL="${DATABASE_URL:-postgresql://aido@localhost:5432/pacs_db}"
export S3_BUCKET="${S3_BUCKET:-test-bucket}"
export S3_REGION="${S3_REGION:-ap-northeast-2}"
export S3_ACCESS_KEY="${S3_ACCESS_KEY:-test-key}"
export S3_SECRET_KEY="${S3_SECRET_KEY:-test-secret}"

# DB 이름
DB_NAME="pacs_db"

GC_RUNNER="./target/debug/gc_runner"

# 색상 코드
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 빌드 확인
if [ ! -f "$GC_RUNNER" ]; then
    echo "❌ gc_runner binary not found. Building..."
    cargo build --bin gc_runner
fi

# 테스트 데이터 정리 함수
cleanup_test_data() {
    echo ""
    echo "🧹 Cleaning up test data..."
    psql -d $DB_NAME -c "DELETE FROM gc_deletion_log WHERE annotation_id >= 90000;" 2>/dev/null || true
    psql -d $DB_NAME -c "DELETE FROM annotation_annotation WHERE id >= 90000;" 2>/dev/null || true
    psql -d $DB_NAME -c "DELETE FROM security_project WHERE id = 99999;" 2>/dev/null || true
    psql -d $DB_NAME -c "DELETE FROM security_user WHERE id = 99999;" 2>/dev/null || true
    echo "✅ Cleanup completed"
}

# 테스트용 project와 user 생성
setup_test_fixtures() {
    echo ""
    echo "📦 Setting up test fixtures..."

    # 테스트용 user 생성
    psql -d $DB_NAME <<EOF
INSERT INTO security_user (id, keycloak_id, username, email, created_at)
OVERRIDING SYSTEM VALUE VALUES (
    99999, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 'test-gc-user', 'test-gc@example.com', NOW()
) ON CONFLICT (id) DO NOTHING;
EOF

    # 테스트용 project 생성
    psql -d $DB_NAME <<EOF
INSERT INTO security_project (id, name, description, is_active, created_at)
OVERRIDING SYSTEM VALUE VALUES (
    99999, 'test-gc-project', 'Test project for GC E2E', true, NOW()
) ON CONFLICT (id) DO NOTHING;
EOF

    echo "✅ Test fixtures created"
}

# 테스트 데이터 생성 함수
create_test_data() {
    local status=$1
    local days_ago=$2
    local annotation_id=$3
    local snapshot_key=$4

    local created_at=$(date -u -v-${days_ago}d +"%Y-%m-%d %H:%M:%S")

    # snapshot_uploaded_at 값 결정
    local uploaded_at="NULL"
    if [ "$status" = "completed" ]; then
        uploaded_at="'${created_at}'"
    fi

    psql -d $DB_NAME <<EOF
INSERT INTO annotation_annotation (
    id, project_id, user_id, study_uid, series_uid, instance_uid,
    tool_name, data, is_shared, created_at, updated_at,
    snapshot_image_key, snapshot_status, snapshot_uploaded_at
) OVERRIDING SYSTEM VALUE VALUES (
    ${annotation_id}, 99999, 99999, 'test-study-${annotation_id}', 'test-series', 'test-instance',
    'test-tool', '{}', false, '${created_at}', '${created_at}',
    ${snapshot_key}, '${status}', ${uploaded_at}
) ON CONFLICT (id) DO UPDATE SET
    snapshot_status = '${status}',
    snapshot_image_key = ${snapshot_key},
    created_at = '${created_at}',
    updated_at = '${created_at}',
    snapshot_uploaded_at = ${uploaded_at};
EOF
}

# 테스트 시작
echo ""
echo "📋 Test Setup"
echo "-------------"
cleanup_test_data
setup_test_fixtures

echo ""
echo "📊 Creating test data..."

# 시나리오 1: PENDING 3일 이상 (타임아웃 대상)
echo "  - Creating PENDING annotation (4 days old) - ID: 90001"
create_test_data "pending" 4 90001 "'snapshots/99999/90001/test-90001.png'"

# 시나리오 2: PENDING 3일 미만 (타임아웃 대상 아님)
echo "  - Creating PENDING annotation (2 days old) - ID: 90002"
create_test_data "pending" 2 90002 "'snapshots/99999/90002/test-90002.png'"

# 시나리오 3: FAILED 7일 이상 (정리 대상)
echo "  - Creating FAILED annotation (8 days old) - ID: 90003"
create_test_data "failed" 8 90003 "'snapshots/99999/90003/test-90003.png'"

# 시나리오 4: FAILED 7일 미만 (정리 대상 아님)
echo "  - Creating FAILED annotation (5 days old) - ID: 90004"
create_test_data "failed" 5 90004 "'snapshots/99999/90004/test-90004.png'"

# 시나리오 5: COMPLETED (처리 대상 아님)
echo "  - Creating COMPLETED annotation (10 days old) - ID: 90005"
create_test_data "completed" 10 90005 "'snapshots/99999/90005/test-90005.png'"

echo ""
echo "✅ Test data created"

# 현재 상태 확인
echo ""
echo "📊 Current state before GC:"
echo "----------------------------"
psql -d $DB_NAME -c "
SELECT id, snapshot_status, snapshot_image_key, 
       DATE_PART('day', NOW() - created_at) as days_old
FROM annotation_annotation 
WHERE id >= 90000 
ORDER BY id;
"

# 테스트 1: Job A Dry-run
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test 1: Job A - PENDING Timeout (Dry-run)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
$GC_RUNNER timeout-pending --grace-days 3 --batch-size 100 --dry-run

echo ""
echo "📊 State after Job A Dry-run (should be unchanged):"
psql -d $DB_NAME -c "
SELECT id, snapshot_status, snapshot_image_key
FROM annotation_annotation 
WHERE id >= 90000 
ORDER BY id;
"

# 테스트 2: Job A 실제 실행
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test 2: Job A - PENDING Timeout (Actual)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
$GC_RUNNER timeout-pending --grace-days 3 --batch-size 100

echo ""
echo "📊 State after Job A (ID 90001 should be FAILED):"
psql -d $DB_NAME -c "
SELECT id, snapshot_status, snapshot_image_key
FROM annotation_annotation 
WHERE id >= 90000 
ORDER BY id;
"

# 검증 1: ID 90001이 FAILED로 변경되었는지 확인
RESULT=$(psql -d $DB_NAME -t -c "SELECT snapshot_status FROM annotation_annotation WHERE id = 90001;")
if [[ "$RESULT" == *"failed"* ]]; then
    echo -e "${GREEN}✅ Test 2 PASSED: ID 90001 changed to FAILED${NC}"
else
    echo -e "${RED}❌ Test 2 FAILED: ID 90001 status is $RESULT${NC}"
    exit 1
fi

# 검증 2: ID 90002는 변경되지 않았는지 확인
RESULT=$(psql -d $DB_NAME -t -c "SELECT snapshot_status FROM annotation_annotation WHERE id = 90002;")
if [[ "$RESULT" == *"pending"* ]]; then
    echo -e "${GREEN}✅ Test 2 PASSED: ID 90002 still PENDING (grace period not met)${NC}"
else
    echo -e "${RED}❌ Test 2 FAILED: ID 90002 status is $RESULT${NC}"
    exit 1
fi

# 테스트 3: Job B Dry-run
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test 3: Job B - FAILED Cleanup (Dry-run)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
$GC_RUNNER cleanup-failed --grace-days 7 --batch-size 100 --dry-run

# 테스트 4: Job B 실제 실행
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test 4: Job B - FAILED Cleanup (Actual)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
$GC_RUNNER cleanup-failed --grace-days 7 --batch-size 100

echo ""
echo "📊 State after Job B (ID 90003 snapshot_image_key should be NULL):"
psql -d $DB_NAME -c "
SELECT id, snapshot_status, snapshot_image_key
FROM annotation_annotation 
WHERE id >= 90000 
ORDER BY id;
"

# 검증 3: ID 90003의 snapshot_image_key가 NULL인지 확인
RESULT=$(psql -d $DB_NAME -t -c "SELECT snapshot_image_key FROM annotation_annotation WHERE id = 90003;")
if [[ -z "$RESULT" || "$RESULT" == " " ]]; then
    echo -e "${GREEN}✅ Test 4 PASSED: ID 90003 snapshot_image_key is NULL${NC}"
else
    echo -e "${RED}❌ Test 4 FAILED: ID 90003 snapshot_image_key is $RESULT${NC}"
    exit 1
fi

# 검증 4: ID 90004는 변경되지 않았는지 확인
RESULT=$(psql -d $DB_NAME -t -c "SELECT snapshot_image_key FROM annotation_annotation WHERE id = 90004;")
if [[ "$RESULT" == *"test-90004"* ]]; then
    echo -e "${GREEN}✅ Test 4 PASSED: ID 90004 snapshot_image_key still exists (grace period not met)${NC}"
else
    echo -e "${YELLOW}⚠️  Test 4 WARNING: ID 90004 snapshot_image_key is $RESULT${NC}"
fi

# GC 로그 확인
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 GC Deletion Log"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
psql -d $DB_NAME -c "
SELECT id, annotation_id, job_type, s3_key, success, error_message, created_at
FROM gc_deletion_log 
WHERE annotation_id >= 90000
ORDER BY created_at DESC;
"

# 정리
cleanup_test_data

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ All E2E tests PASSED!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

