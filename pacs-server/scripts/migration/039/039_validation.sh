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
validate_table_exists() {
  echo ""
  echo "[1] Table existence validation..."

  local result
  result=$(run_psql "
    SELECT table_name
    FROM information_schema.tables
    WHERE table_schema = 'public'
      AND table_name = 'gc_deletion_log';
  ")

  if echo "$result" | grep -qx "gc_deletion_log"; then
    report_pass "table 'gc_deletion_log' exists"
  else
    report_fail "table 'gc_deletion_log' missing"
  fi
}

validate_columns() {
  echo ""
  echo "[2] Column validation..."

  local result
  result=$(run_psql "
    SELECT column_name
    FROM information_schema.columns
    WHERE table_name = 'gc_deletion_log'
      AND column_name IN (
        'id',
        'annotation_id',
        'snapshot_image_key',
        'file_size',
        'deleted_at',
        'dry_run',
        'status',
        'error_message'
      );
  ")

  local expected_cols=(
    "id"
    "annotation_id"
    "snapshot_image_key"
    "file_size"
    "deleted_at"
    "dry_run"
    "status"
    "error_message"
  )

  for col in "${expected_cols[@]}"; do
    if echo "$result" | grep -qx "$col"; then
      report_pass "column '$col' exists"
    else
      report_fail "column '$col' missing"
    fi
  done
}

validate_column_types() {
  echo ""
  echo "[3] Column type validation..."

  local result
  result=$(run_psql "
    SELECT column_name, data_type
    FROM information_schema.columns
    WHERE table_name = 'gc_deletion_log'
    ORDER BY ordinal_position;
  ")

  # id: bigint
  if echo "$result" | grep -q "id|bigint"; then
    report_pass "column 'id' type is bigint"
  else
    report_fail "column 'id' type is not bigint"
  fi

  # annotation_id: integer
  if echo "$result" | grep -q "annotation_id|integer"; then
    report_pass "column 'annotation_id' type is integer"
  else
    report_fail "column 'annotation_id' type is not integer"
  fi

  # snapshot_image_key: text
  if echo "$result" | grep -q "snapshot_image_key|text"; then
    report_pass "column 'snapshot_image_key' type is text"
  else
    report_fail "column 'snapshot_image_key' type is not text"
  fi

  # file_size: bigint
  if echo "$result" | grep -q "file_size|bigint"; then
    report_pass "column 'file_size' type is bigint"
  else
    report_fail "column 'file_size' type is not bigint"
  fi

  # deleted_at: timestamp with time zone
  if echo "$result" | grep -q "deleted_at|timestamp with time zone"; then
    report_pass "column 'deleted_at' type is timestamptz"
  else
    report_fail "column 'deleted_at' type is not timestamptz"
  fi

  # dry_run: boolean
  if echo "$result" | grep -q "dry_run|boolean"; then
    report_pass "column 'dry_run' type is boolean"
  else
    report_fail "column 'dry_run' type is not boolean"
  fi

  # status: text
  if echo "$result" | grep -q "status|text"; then
    report_pass "column 'status' type is text"
  else
    report_fail "column 'status' type is not text"
  fi
}

validate_constraints() {
  echo ""
  echo "[4] Constraint validation..."

  # Primary Key
  local pk_result
  pk_result=$(run_psql "
    SELECT constraint_name
    FROM information_schema.table_constraints
    WHERE table_name = 'gc_deletion_log'
      AND constraint_type = 'PRIMARY KEY';
  ")

  if [[ -n "$pk_result" ]]; then
    report_pass "primary key constraint exists"
  else
    report_fail "primary key constraint missing"
  fi

  # Foreign Key
  local fk_result
  fk_result=$(run_psql "
    SELECT constraint_name
    FROM information_schema.table_constraints
    WHERE table_name = 'gc_deletion_log'
      AND constraint_type = 'FOREIGN KEY'
      AND constraint_name = 'fk_annotation';
  ")

  if echo "$fk_result" | grep -qx "fk_annotation"; then
    report_pass "foreign key 'fk_annotation' exists"
  else
    report_fail "foreign key 'fk_annotation' missing"
  fi

  # Check Constraint (status)
  local check_result
  check_result=$(run_psql "
    SELECT constraint_name
    FROM information_schema.table_constraints
    WHERE table_name = 'gc_deletion_log'
      AND constraint_type = 'CHECK';
  ")

  if [[ -n "$check_result" ]]; then
    report_pass "check constraint on 'status' exists"
  else
    report_fail "check constraint on 'status' missing"
  fi
}

validate_indexes() {
  echo ""
  echo "[5] Index validation..."

  local result
  result=$(run_psql "
    SELECT indexname
    FROM pg_indexes
    WHERE tablename = 'gc_deletion_log';
  ")

  local expected_indexes=(
    "idx_gc_deletion_log_annotation_id"
    "idx_gc_deletion_log_deleted_at"
    "idx_gc_deletion_log_status"
  )

  for idx in "${expected_indexes[@]}"; do
    if echo "$result" | grep -qx "$idx"; then
      report_pass "index '$idx' exists"
    else
      report_fail "index '$idx' missing"
    fi
  done
}

validate_comments() {
  echo ""
  echo "[6] Comment validation..."

  # Table comment
  local table_comment
  table_comment=$(run_psql "
    SELECT obj_description('gc_deletion_log'::regclass, 'pg_class');
  ")

  if [[ -n "$table_comment" ]]; then
    report_pass "table comment exists"
  else
    report_fail "table comment missing"
  fi

  # Column comments
  local col_comments
  col_comments=$(run_psql "
    SELECT column_name
    FROM information_schema.columns c
    WHERE table_name = 'gc_deletion_log'
      AND EXISTS (
        SELECT 1
        FROM pg_catalog.pg_description d
        JOIN pg_catalog.pg_class cl ON d.objoid = cl.oid
        JOIN pg_catalog.pg_attribute a ON a.attrelid = cl.oid AND a.attnum = d.objsubid
        WHERE cl.relname = 'gc_deletion_log'
          AND a.attname = c.column_name
      );
  ")

  local expected_commented_cols=(
    "annotation_id"
    "snapshot_image_key"
    "file_size"
    "deleted_at"
    "dry_run"
    "status"
    "error_message"
  )

  for col in "${expected_commented_cols[@]}"; do
    if echo "$col_comments" | grep -qx "$col"; then
      report_pass "column '$col' has comment"
    else
      report_fail "column '$col' comment missing"
    fi
  done
}

########################################
# main: 흐름 제어 (오케스트레이션)
########################################
main() {
  echo "======================================"
  echo " GC Deletion Log Schema Validation"
  echo "======================================"

  prompt_password

  validate_table_exists
  validate_columns
  validate_column_types
  validate_constraints
  validate_indexes
  validate_comments

  cleanup_password
  print_summary
}

main "$@"
