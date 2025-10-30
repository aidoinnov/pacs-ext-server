#!/usr/bin/env bash
set -euo pipefail

DB_URL="${1:-${APP_DATABASE_URL:-${DATABASE_URL:-}}}"
if [ -z "${DB_URL}" ]; then
  echo "Usage: $0 [DATABASE_URL]  (or set APP_DATABASE_URL/DATABASE_URL)" >&2
  exit 1
fi

PSQL="psql -v ON_ERROR_STOP=1 -X -d \"${DB_URL}\""
BASE_DIR="$(cd "$(dirname "$0")" && pwd)"
SQL_DIR="${BASE_DIR}/sql"
OUT_DIR="${BASE_DIR}/results"
mkdir -p "${OUT_DIR}"

run_block() {
  local label="$1" sql_file="$2" out_file="$3"
  echo "===== ${label} =====" | tee "${out_file}"
  # Use psql with timing captured inside SQL
  eval ${PSQL} -f "${sql_file}" | tee -a "${out_file}"
}

# BEFORE: drop indexes, then bench
echo "[1/3] Dropping core indexes (simulating BEFORE)" | tee "${OUT_DIR}/index_bench_before.txt"
eval ${PSQL} -f "${SQL_DIR}/drop_core_indexes.sql" | tee -a "${OUT_DIR}/index_bench_before.txt"
run_block "BEFORE: EXPLAIN ANALYZE" "${SQL_DIR}/bench_core.sql" "${OUT_DIR}/index_bench_before.txt"

# AFTER: recreate indexes from migration 018, then bench
echo "[2/3] Creating core indexes (AFTER)" | tee "${OUT_DIR}/index_bench_after.txt"
eval ${PSQL} -f "${PWD}/migrations/018_core_indices.sql" | tee -a "${OUT_DIR}/index_bench_after.txt"
run_block "AFTER: EXPLAIN ANALYZE" "${SQL_DIR}/bench_core.sql" "${OUT_DIR}/index_bench_after.txt"

# DIFF: naive diff for visibility
echo "[3/3] Diff BEFORE vs AFTER (plans and timings)"
diff -u "${OUT_DIR}/index_bench_before.txt" "${OUT_DIR}/index_bench_after.txt" > "${OUT_DIR}/index_bench_diff.txt" || true

echo
echo "Outputs:"
echo "  BEFORE: ${OUT_DIR}/index_bench_before.txt"
echo "  AFTER : ${OUT_DIR}/index_bench_after.txt"
echo "  DIFF  : ${OUT_DIR}/index_bench_diff.txt"
