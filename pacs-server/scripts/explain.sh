#!/usr/bin/env bash
# Usage: APP_DATABASE_URL=postgres://... ./scripts/explain.sh "<SQL>" [outfile]
set -euo pipefail

if [ -z "${APP_DATABASE_URL:-}" ]; then
  echo "APP_DATABASE_URL is not set" >&2
  exit 1
fi

SQL=${1:-}
if [ -z "$SQL" ]; then
  echo "Provide SQL to EXPLAIN, e.g.: SELECT * FROM project_data_study LIMIT 1;" >&2
  exit 1
fi

TS=$(date +%Y%m%d-%H%M%S)
OUTFILE=${2:-"scripts/results/explain_${TS}.txt"}
mkdir -p scripts/results

# Run EXPLAIN ANALYZE with psql
PGURL="$APP_DATABASE_URL"
{
  echo "-- EXPLAIN captured at $TS"
  echo "$SQL" | sed 's/^/-- SQL: /'
  echo
  PGPASSWORD=$(echo "$PGURL" | sed -n 's#postgres://[^:]*:\([^@]*\)@.*#\1#p') \
  psql "$PGURL" -v ON_ERROR_STOP=1 -X -q -c "EXPLAIN (ANALYZE, BUFFERS, TIMING, VERBOSE) $SQL"
} | tee "$OUTFILE"

echo "Saved: $OUTFILE"
