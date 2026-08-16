# CLAUDE.md — The Smuggler's Ledger

*The lab journal for `gadgets/smugglers-ledger/`. Read this before picking up the scalpel.*

## What This Is

**The Smuggler's Ledger** is a Tauri v2 Windows desktop gadget that decrypts Grim Dawn's own save files, walks every character and the shared transfer stash, resolves display names and rarity from the user's local game install, and gives one searchable overview of every item across the whole of Cairn. Read-only, offline-capable, zero-config on a second machine.

The domain speaks Cairn, not inventory-app: the Ledger **turns the cipher** on a save (never "decrypts a file"); a resolved item is **named**, an unresolved one is **contraband still in the crate**; the shared stash is **the warehouse**; a character's loadout is **the manifest**; search results carry **a location, not a match score** — `SPINNY — BAGS, BAG 1, CELL 2,3`.

- **Experiment log:** `zmuuzn/documents/experiment-logs/00065-the-smugglers-ledger.md`
- **Format record (canonical):** `zmuuzn/.claude/memory/grim-dawn-save-format.md` — cipher constants, block structures, the four 1.2-era deltas. Ratified against two distinct real datasets (2026-08-15 + 2026-08-16). Cite as fact; the public references are stale.
- **Design system:** `zmuuzn/documents/design-systems/00012-system-smugglers-ledger.md` — **GASLAMP DOUBLE-ENTRY**. Litmus for every UI decision: *would a gaslamp double-entry ledger do this?* No pills, no glow, no spinner, no refresh button, radius 0, ONE shadow.
- **Wireframe:** `zmuuzn/documents/wireframes/00038-wire-ledger-overview.md`

## The Architecture Contract (Investor-Ratified 2026-08-16)

- **RD-1 — Rust does everything.** All parsing (cipher, block walking, `.arz`/`.arc` reading) lives in `src-tauri/`. No sidecars, no Python, no shelling out.
- **RD-2 — Read-only, always.** No write-mode file operation anywhere the code touches a save path. Gated by `scripts/read-only-audit.sh` (in the Sentinel), which sweeps `discovery.rs`, `manifest.rs`, `warehouse.rs`, `ledger.rs`, `watch.rs` for the full forbidden set. `codex.rs` is excluded for ONE reason (it writes its resolve cache to `app_data_dir()`) and carries the countervailing rule: every file it opens passes `shelf_guard`, which refuses any path outside the install root and the cache dir — asserted by its own test.
- **RD-3 — Names come from the user's own install, never bundled.** Gated by `scripts/legal-floor-audit.sh` (in the Sentinel): no `.arz`/`.arc`/`.dbr`/`.tex` tracked, no record paths in the shipped bundle, fixtures capped at 64 KiB.
- **RD-4 — Discovery, not configuration.** `discovery.rs` scans every Steam profile's `userdata/*/219990/remote/save/` plus `Documents/My Games/Grim Dawn/save/`, scores candidates by freshest recursive mtime, surfaces the chosen root with every other root as a switch. Manual picker (`tauri-plugin-dialog`) is the fallback, never the front door.
- **RD-5 — File-watched, not polled.** `watch.rs` debounces save-write bursts behind `SAVE_WRITE_DEBOUNCE_MS = 1_500` and fires exactly one re-parse per burst. There is no refresh button anywhere.
- **RD-6 — Vue 3 + UnoCSS**, `uno.config.ts` carries the full `--sl-*` token system and the mandatory `prefers-reduced-motion` floor.

## The Module Map (src-tauri/src/)

| Module | Register | What it does |
|---|---|---|
| `cipher.rs` | mechanism | XOR stream + 256-entry rolling key table, typed read primitives. Ported verbatim from the proven spike, NOT the stale references |
| `discovery.rs` | mechanism | Root auto-discovery, freshest-recursive-wins scoring, Steam registry / libraryfolders.vdf lookup |
| `watch.rs` | mechanism | Debounced save watcher (`notify`), `settle_loop` testable core |
| `error.rs` | mechanism | `LedgerError`, serializable across the Tauri bridge |
| `contraband.rs` | domain | THE item reader — both v11 traps closed here, in one place, gated on the **block** version passed in |
| `manifest.rs` | domain | One `player.gdc`: header, info (loot-filter tail raw-consumed), bio, inventory, personal stash |
| `warehouse.rs` | domain | One `transfer.gst`: tabs with their OWN parsed grid geometry |
| `codex.rs` | domain | `.arz` + `Text_EN.arc` readers, tag→name resolution, hash-keyed per-machine cache |
| `ledger.rs` | domain | The spine: enumeration loop, aggregate managed state, `list_characters` / `list_stash` / `search_ledger` / `ledger_overview` commands, two-stage cold turn |
| `fixtures.rs` | test-only | `#[cfg(test)]` — fixture loader + the Scribe (the cipher's inverse; the app itself never encrypts). Forge: `cargo test -- --ignored forge_fixtures` |

**The four 1.2-era deltas live in the format record — never "fix" a parser by trusting `AaronHutchinson/Grim-Dawn-Save-Decryption` or `gd-edit` verbatim:** versions are per-BLOCK not per-file (file stays v8 while inventory reports v11); v11 items carry 4 trailing int32s before x/y; v11 stash tabs carry a 20-byte trailer (shared AND personal); the transfer expansion byte reads 7 not 3.

## The Rarity Mapping (Open Question #1 — SETTLED 2026-08-16)

Enumerated on the bench across the full real save set (1,196 distinct records, 10 characters + 10-tab stash). The distinct `itemClassification` values are exactly six, mapped ordinally in `codex.rs::classification_tier`:

| Classification | Tier | Ink |
|---|---|---|
| Common | 0 | `--sl-tier-0` |
| Magical | 1 | `--sl-tier-1` |
| Rare | 2 | `--sl-tier-2` |
| Epic | 3 | `--sl-tier-3` |
| Legendary | 4 | `--sl-tier-4` |
| Quest | 5 | `--sl-tier-5` |

Unknown values render tier-0. The `RarityMark` glyph (`· ◦ ◇ ◆ ✦ ✷`) is the primary carrier; colour is reinforcement (WCAG 1.4.1).

## Containment Protocols (the gates on disk)

```sh
# Frontend (repo root)
npm run typecheck        # vue-tsc, strict
npm run lint             # oxlint --type-aware (war-room canonical config)
npm test                 # vitest — the six 4D voiced states are GATED here
npm run format:check     # oxfmt
npm run build            # vue-tsc + vite build

# Rust (src-tauri/)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test               # every [GATE] criterion on committed fixtures

# The two floors (repo root)
bash scripts/read-only-audit.sh    # RD-2
bash scripts/legal-floor-audit.sh  # RD-3 (bundle sweep needs dist/ built)
```

The Sentinel (`.github/workflows/sentinel.yml`) runs all of the above on every push/PR to `main`. Merge policy per Decision 025: **merge commits only**.

### The bench suite (never in CI)

`src-tauri/tests/bench_real.rs` — `#[ignore]`d, env-gated. The Sentinel never sees a real save:

```sh
SMUGGLERS_BENCH_SAVE_ROOT="/mnt/c/Program Files (x86)/Steam/userdata/<id>/219990/remote/save" \
SMUGGLERS_BENCH_INSTALL_ROOT="/mnt/c/Program Files (x86)/Steam/steamapps/common/Grim Dawn" \
cargo test --release --test bench_real -- --ignored --nocapture --test-threads 1
```

Bench record (2026-08-16, this bench): 10/10 characters parse clean; transfer.gst 10 tabs / 500 items; COLD full-database resolve **2.17 s** (criterion ceiling 60 s); WARM parse+resolve **≈118 ms** (criterion ceiling 2 s); 1,195/1,196 records named — the one unresolved (`records/items/lootaffixes/prefixunique/a001.dbr`, an affix with no display tag) surfaces as its raw path, searchable.

## Design System Notes for Builders

- **The two grounds:** the shell is soot (`--sl-soot`), the sheet is folio cream. The folio is a **local alias remap** on `.sl-folio` (lives in `LedgerSheet.vue` and nowhere else). **The trap:** var()-indirected token maps bake at `:root` — every indirected alias must be RE-DECLARED under `.sl-folio`, never merely inherited.
- **THE WET INK** is the signature interaction and the ONLY reaction surface: new/changed rows mount at 2× pitch, full ink, +0.4px tracking, settle over `--sl-dur-ink` (1500 ms). No quill, no hand, no page turn, no toast, no spinner — ever.
- **`--sl-lamp` is forbidden as text on paper (1.92:1); `--sl-wax` is forbidden as text on soot (2.26:1).** Use `--sl-wax-lit` on soot.
- **`--sl-ink-soft` alpha is 0.68, not 0.62** — 0.62 measured 4.24:1 and failed AA. Do not "tidy" it.
- Every vertical dimension on the sheet snaps to `--sl-pitch` (26 px) or the ruling breaks.
- Reduced motion: the CSS preflight floor in `uno.config.ts` covers the whole surface (no canvas/RAF exists here). Every state is a *place* — a printed time, a dog-eared tab, a completed rule.

## Known Limitations (ships broken-shaped, by design)

- **No auto-update.** The installer is hand-delivered; a new version is a new `.msi`/`.nsis` handed over again.
- **Windows-only.** Discovery reads the Windows Steam registry; no macOS/Linux path exists or is planned. (`SMUGGLERS_STEAM_ROOT` / `SMUGGLERS_GAME_ROOT` env overrides serve the WSL2 bench.)
- **No item icons.** Items render as named text with rarity ink — `Items.arc` `.tex` decoding is Project Phase 2.
- **Four of the five `.gst` families ship unread.** `formulas`, `reagents`, `transmutes`, `potions` sit at the save root, parsed by nothing.
- **One root at a time.** Freshest root by default with a manual switch — no merged multi-profile view.
- **English text tables only.** The codex reads `Text_EN.arc`; a non-English install resolves nothing until a localization pass.
- **The Documents save layout is still a labelled assumption** — exercised against synthetic fixture trees only until the brother test (log Phase 5B) confirms it on real ground.
- **Affix names resolve via `lootRandomizerName`; item style prefixes (`itemStyleTag`, e.g. "Mythical") are not yet composed into display names.**

## Distribution

`npm run tauri build` on a Windows host produces the `.msi`/`.nsis` installers (bundle targets in `tauri.conf.json`, window 1180×780 / min 960×640 per wireframe #00038). Hand-delivered, like the Cube.
