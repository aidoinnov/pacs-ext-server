#!/usr/bin/env bash
set -euo pipefail

# Clean build artifacts for this workspace. Optional flags:
#   --all       also purge cargo caches (~/.cargo/registry, ~/.cargo/git) and rustup toolchains not in use
#   --targets   comma-separated list of relative subproject paths to clean (default: detect Cargo workspaces)
#   --dry-run   show what would be removed without deleting

DRY_RUN=false
ALL=false
TARGETS=""

for arg in "$@"; do
  case "$arg" in
    --all) ALL=true ;;
    --dry-run) DRY_RUN=true ;;
    --targets=*) TARGETS="${arg#*=}" ;;
    *) echo "Unknown option: $arg" && exit 2 ;;
  esac
done

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

remove_path() {
  local path="$1"
  if [[ -e "$path" ]]; then
    if $DRY_RUN; then
      echo "[dry-run] rm -rf $path"
    else
      rm -rf "$path"
      echo "removed: $path"
    fi
  fi
}

echo "Cleaning workspace targets..."

# If specific targets provided, clean those; else try cargo clean at workspace root and known subprojects
if [[ -n "$TARGETS" ]]; then
  IFS=',' read -r -a arr <<< "$TARGETS"
  for rel in "${arr[@]}"; do
    remove_path "$ROOT_DIR/$rel/target"
  done
else
  # Generic cargo clean at repo root if Cargo.toml exists
  if [[ -f "$ROOT_DIR/Cargo.toml" ]]; then
    if $DRY_RUN; then
      echo "[dry-run] cargo clean (root)"
    else
      (cd "$ROOT_DIR" && cargo clean || true)
    fi
  fi
  # Also clean pacs-server sub-crate target explicitly
  remove_path "$ROOT_DIR/pacs-server/target"
fi

echo "Pruning incremental caches (debug)..."
remove_path "$ROOT_DIR/target/debug/incremental"
remove_path "$ROOT_DIR/pacs-server/target/debug/incremental"

if $ALL; then
  echo "--all specified: purging cargo caches (~/.cargo)"
  remove_path "$HOME/.cargo/registry/cache"
  remove_path "$HOME/.cargo/registry/index"
  remove_path "$HOME/.cargo/git"
  echo "Note: rustup toolchains are under ~/.rustup; manage with 'rustup toolchain list/uninstall'"
fi

echo "Done."



