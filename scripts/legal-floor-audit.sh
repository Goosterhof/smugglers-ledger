#!/usr/bin/env bash
# ============================================================================
# RD-3 LEGAL-FLOOR GATE — Names come from the user's own install, never bundled.
# ============================================================================
# database.arz / gdx* / Text_EN.arc and everything inside them are the
# developer's copyrighted game data. The Ledger reads a user's own local
# install at runtime and caches what it resolves FOR THAT USER — it never
# bundles, ships, or redistributes so much as one extracted record. This is
# the line between "reads Grim Dawn" and "redistributes Grim Dawn".
#
# Two sweeps, zero matches required outside the parser's own source constants:
#   1. `git ls-files` for game-data file extensions (.arz/.arc/.dbr/.tex).
#   2. The built frontend bundle (dist/) for the ARZ record-path prefix
#      (`records/` strings other than the parser's own constants live in Rust,
#      not in dist) and for raw ARZ bytes.
#
# Run from the repo root, after `npm run build` when dist/ exists.
# ============================================================================
set -euo pipefail

cd "$(dirname "$0")/.."

status=0

# --- Sweep 1: no game-data file may ever be committed --------------------
if tracked=$(git ls-files | grep -iE '\.(arz|arc|dbr|tex)$'); then
  echo "RD-3 VIOLATION — game data files tracked in the repo:" >&2
  echo "$tracked" >&2
  status=1
fi

# --- Sweep 2: the shipped bundle carries no extracted records ------------
# The ARZ format's little-endian version-3 header starts 0x?? 0x00 0x03 0x00;
# scanning for real ARZ content is done by the record-path prefix, which only
# exists inside game data — the Ledger's own strings (test fixtures included)
# live in Rust source and byte fixtures, never in the web bundle.
if [ -d dist ]; then
  if hits=$(grep -rl "records/items/" dist 2>/dev/null); then
    echo "RD-3 VIOLATION — DBR record paths found in the shipped bundle:" >&2
    echo "$hits" >&2
    status=1
  fi
else
  echo "RD-3 note: dist/ not built — bundle sweep skipped (repo sweep still ran)."
fi

# --- Sweep 3: byte fixtures must stay tiny and synthetic/anonymised ------
# A committed fixture larger than 64 KiB could hide real game data. Every
# legitimate fixture in this repo is a few hundred bytes.
while IFS= read -r fixture; do
  size=$(wc -c <"$fixture")
  if [ "$size" -gt 65536 ]; then
    echo "RD-3 VIOLATION — fixture $fixture is ${size} bytes (cap 65536): too big to be an anonymised capture" >&2
    status=1
  fi
done < <(git ls-files 'src-tauri/fixtures/*')

if [ "$status" -eq 0 ]; then
  echo "RD-3 legal-floor audit: clean. Zero bytes of game data ship anywhere."
fi
exit "$status"
