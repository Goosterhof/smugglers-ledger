# The Smuggler's Ledger

*A back room in Cairn at two in the morning. One oil lamp, one open folio, one man who knows exactly where everything is.*

The Smuggler's Ledger is a read-only Windows desktop companion for **Grim Dawn**. It finds your saves by itself, turns the cipher on every character and the shared transfer stash, resolves display names and rarity from your own local game install, and answers the one question the game refuses to: **where is my Ectoplasm?**

- Type a name, read the WHEREABOUTS column: `SPINNY — BAGS, BAG 1, CELL 2,3`.
- Alt-tab out, play, alt-tab back — the page has already turned. No refresh button exists.
- Install it on a second machine and it finds that machine's own saves and install. No path typed, no config edited, no README consulted beyond "run the installer."

## The promises

- **Read-only, always.** The Ledger opens save files for reading and never for anything else — enforced by an automated sweep in CI, not by intent.
- **Nothing of the game's ships with the Ledger.** Names and rarities are resolved from *your* install at runtime and cached for *your* machine. The binary carries zero extracted game data — also enforced in CI.
- **Offline-capable.** No network access, no CDN, no telemetry. The fonts are in the box.

## Development

```sh
npm install
npm run tauri dev      # the bench (WSL2: set SMUGGLERS_STEAM_ROOT / SMUGGLERS_GAME_ROOT)
npm run tauri build    # the .msi / .nsis installers (Windows host)
```

The full lab journal — architecture contract, module map, format deltas, gates — is in [`CLAUDE.md`](CLAUDE.md). The save-format research record lives in the parent laboratory.

Part of the [Zmuuzn laboratory](https://github.com/Goosterhof/zmuuzn). Not affiliated with Crate Entertainment; Grim Dawn is theirs, and the Ledger never touches what's theirs beyond reading your own licensed files on your own disk.
