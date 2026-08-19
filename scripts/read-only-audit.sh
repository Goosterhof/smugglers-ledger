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
# Zero matches required. Every module outside the sweep is EXCLUDED by name,
# with its reason, in EXCLUDED below — and the sweep fails if a module exists
# that neither list claims, so a new save-touching module can never appear
# unswept in silence. Notable exclusions: `codex.rs` must write its resolve
# cache to app_data_dir() and carries the countervailing rule instead (it
# never opens any path under a discovered save root, asserted by its own
# shelf_guard test); `fixtures.rs` is the one module that ENCRYPTS and writes
# — it is the test-only Scribe that forges fixture saves, #[cfg(test)]-gated
# in lib.rs so it never ships in the binary.
#
# Run from the repo root. Exit 0 = the promise holds. Exit 1 = a write-mode
# operation reached a module that touches save paths, or a module exists
# that the sweep does not classify.
# ============================================================================
set -euo pipefail

cd "$(dirname "$0")/.."

SWEPT=(
  src-tauri/src/discovery.rs
  # trades.rs opens nothing — it reads shelves the codex already holds. Swept
  # anyway: the sweep is free here and keeps it that way.
  src-tauri/src/trades.rs
  src-tauri/src/manifest.rs
  src-tauri/src/warehouse.rs
  src-tauri/src/ledger.rs
  src-tauri/src/watch.rs
)

# Every non-swept module, each with its stated reason:
EXCLUDED=(
  src-tauri/src/cipher.rs     # pure byte transform — no filesystem I/O at all
  src-tauri/src/contraband.rs # pure parser over in-memory bytes — no filesystem I/O
  src-tauri/src/codex.rs      # writes ONLY its resolve cache to app_data_dir(); shelf_guard test asserts it never opens a save-root path
  src-tauri/src/icons.rs      # reads Items.arc from the game install and decodes textures in memory; never writes, never opens a save-root path
  src-tauri/src/error.rs      # error types only
  src-tauri/src/fixtures.rs   # test-only Scribe: encrypts + writes FIXTURES, #[cfg(test)]-gated in lib.rs, never in the shipped binary
  src-tauri/src/lib.rs        # module wiring
  src-tauri/src/main.rs       # Tauri entry point
)

# Completeness: SWEPT ∪ EXCLUDED must be exactly the modules on disk.
actual=$(find src-tauri/src -maxdepth 1 -name '*.rs' | sort)
claimed=$(printf '%s\n' "${SWEPT[@]}" "${EXCLUDED[@]}" | sort)
if [ "$actual" != "$claimed" ]; then
  echo "RD-2 SWEEP INCOMPLETE: modules on disk do not match SWEPT + EXCLUDED." >&2
  echo "Classify every new module — swept, or excluded WITH a reason:" >&2
  diff <(echo "$claimed") <(echo "$actual") >&2 || true
  exit 1
fi

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
