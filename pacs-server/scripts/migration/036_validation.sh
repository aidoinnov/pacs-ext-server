#!/bin/bash
set -euo pipefail

########################################
# config: 환경 설정
########################################
HOST="localhost"
PORT="5456"
USER="pacs_extension_admin"
DB="pacs_extension"

########################################
# reporter: 결과 집계 및 출력
########################################
PASS_CNT=0
FAIL_CNT=0

report_pass() {
  echo "✅ PASS - $1"
  PASS_CNT=$((PASS_CNT + 1))
}

report_fail() {
  echo "❌ FAIL - $1"
  FAIL_CNT=$((FAIL_CNT + 1))
}

print_summary() {
  echo ""
  echo "======================================"
  echo " Validation Summary"
  echo "======================================"
  echo "✅ PASS: $PASS_CNT"
  echo "❌ FAIL: $FAIL_CNT"

  if [[ $FAIL_CNT -eq 0 ]]; then
    echo ""
    echo "🎉 ALL CHECKS PASSED"
    exit 0
  else
    echo ""
    echo "🚨 SOME CHECKS FAILED"
    exit 1
  fi
}

########################################
# auth: 인증 책임
########################################
prompt_password() {
  echo -n "🔐 PostgreSQL password for user '$USER': "
  read -s PGPASSWORD
  echo ""
  export PGPASSWORD
}

cleanup_password() {
  unset PGPASSWORD
}

########################################
# db: DB 접근 책임
########################################
run_psql() {
  local sql="$1"
  psql -h "$HOST" -p "$PORT" -U "$USER" -d "$DB" -At -c "$sql"
}

########################################
# validators: 검증 로직
########################################
validate_columns() {
  echo ""
  echo "[1] Column validation..."

  local result
  result=$(run_psql "
    SELECT column_name
    FROM information_schema.columns
    WHERE table_name = 'annotation_annotation'
      AND column_name IN (
        'snapshot_image_key',
        'snapshot_status',
        'snapshot_uploaded_at'
      );
  ")

  for col in snapshot_image_key snapshot_status snapshot_uploaded_at; do
    if echo "$result" | grep -qx "$col"; then
      report_pass "column '$col' exists"
    else
      report_fail "column '$col' missing"
    fi
  done
}

validate_enum() {
  echo ""
  echo "[2] ENUM snapshot_upload_status validation..."

  local result
  result=$(run_psql "
    SELECT enumlabel
    FROM pg_enum
    WHERE enumtypid = 'snapshot_upload_status'::regtype
    ORDER BY enumsortorder;
  ")

  local expected=("pending" "uploading" "completed" "failed")
  for enum in "${expected[@]}"; do
    if echo "$result" | grep -qx "$enum"; then
      report_pass "enum '$enum' exists"
    else
      report_fail "enum '$enum' missing"
    fi
  done
}

validate_indexes() {
  echo ""
  echo "[3] Snapshot index validation..."

  local result
  result=$(run_psql "
    SELECT indexname
    FROM pg_indexes
    WHERE tablename = 'annotation_annotation'
      AND indexname LIKE '%snapshot%';
  ")

  if [[ -n "$result" ]]; then
    report_pass "snapshot related index exists"
  else
    report_fail "no snapshot related index found"
  fi
}

########################################
# main: 흐름 제어 (오케스트레이션)
########################################
main() {
  echo "======================================"
  echo " Snapshot Schema Validation Started"
  echo "======================================"

  prompt_password

  validate_columns
  validate_enum
  validate_indexes

  cleanup_password
  print_summary
}

main "$@"
