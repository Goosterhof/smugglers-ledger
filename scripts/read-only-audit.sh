#!/usr/bin/env bash
# ============================================================================
# RD-2 CONTAINMENT GATE — Read-only, always.
# ============================================================================
# The Ledger opens save files for reading and never for writing. This sweep
# greps the five modules that touch save paths for the full forbidden set:
#
#   OpenOptions builders calling .write( / .append( / .create( / .truncate(
#   File::create   fs::write   fs::remove_file   fs::rename   set_len
#
# Zero matches required. `codex.rs` is EXCLUDED for one stated reason — it
# must write its resolve cache to app_data_dir() — and carries the
# countervailing rule instead: it never opens any path under a discovered
# save root, asserted by its own test (shelf_guard, src/codex.rs).
#
# Run from the repo root. Exit 0 = the promise holds. Exit 1 = a write-mode
# operation reached a module that touches save paths.
# ============================================================================
set -euo pipefail

cd "$(dirname "$0")/.."

SWEPT=(
  src-tauri/src/discovery.rs
  src-tauri/src/manifest.rs
  src-tauri/src/warehouse.rs
  src-tauri/src/ledger.rs
  src-tauri/src/watch.rs
)

FORBIDDEN='\.write\(|\.append\(|\.create\(|\.truncate\(|File::create|fs::write|fs::remove_file|fs::rename|set_len'

status=0
for file in "${SWEPT[@]}"; do
  if [ ! -f "$file" ]; then
    echo "RD-2 SWEEP BROKEN: $file is missing — the sweep list no longer matches the codebase" >&2
    status=1
    continue
  fi
  if matches=$(grep -nE "$FORBIDDEN" "$file"); then
    echo "RD-2 VIOLATION in $file:" >&2
    echo "$matches" >&2
    status=1
  fi
done

if [ "$status" -eq 0 ]; then
  echo "RD-2 read-only audit: clean. The Ledger reads saves and never writes them."
fi
exit "$status"
