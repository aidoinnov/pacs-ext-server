#!/usr/bin/env bash

# Simple log watcher for pacs-ext-server backend.log
# Usage:
#   ./watch_backend_logs.sh              # watch full log
#   ./watch_backend_logs.sh "viewer"      # filter lines containing 'viewer'
#   ./watch_backend_logs.sh "ERROR|500"   # filter by regex (eg. ERROR or 500)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/backend.log"

if [[ ! -f "$LOG_FILE" ]]; then
  echo "[watch_backend_logs] Log file not found: $LOG_FILE" >&2
  echo "Make sure your server is writing to backend.log in the repo root." >&2
  exit 1
fi

FILTER="${1-}"

echo "========================================="
echo "📜 Watching backend log: $LOG_FILE"
if [[ -n "$FILTER" ]]; then
  echo "🔍 Filter: $FILTER"
else
  echo "🔍 Filter: (none, showing all lines)"
fi
echo "Press Ctrl+C to stop."
echo "========================================="

if [[ -n "$FILTER" ]]; then
  # --line-buffered so output appears immediately
  tail -F "$LOG_FILE" | grep --line-buffered -E "$FILTER"
else
  tail -F "$LOG_FILE"
fi

