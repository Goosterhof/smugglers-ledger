//! Discovery — finding the saves without being told (RD-4).
//!
//! The Ledger scans every `userdata/<steamid>/219990/remote/save/` under the
//! Steam install (the registry names the install root on Windows; discovery
//! never guesses first) and the `Documents/My Games/Grim Dawn/save/` layout
//! (unverified on real ground — a labelled assumption exercised against
//! synthetic fixture trees until 5B confirms it on the brother's machine).
//!
//! Character saves live one level below the root — `save/main/_<Name>/
//! player.gdc` — with only the `.gst` files at the root itself, so every
//! candidate root is scored by the most recent file-modified timestamp of any
//! `.gdc`/`.gst` found **recursively**: a flat scan finds zero character
//! saves and would score every real root by its `.gst` files alone.
//!
//! The freshest root wins and becomes the watched directory; every other
//! candidate stays visible in the UI as a one-click switch — freshest-wins is
//! a default, not a verdict. Zero candidates → the manual folder picker
//! (`tauri-plugin-dialog`), the fallback, never the front door.
//!
//! Mechanism module: plain name by design. Read-only always (RD-2): this
//! module opens directories and file metadata, never file contents.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;

const GRIM_DAWN_STEAM_APP_ID: &str = "219990";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateRoot {
    /// The save root itself.
    pub path: PathBuf,
    /// Number of `.gdc`/`.gst` files found recursively under the root.
    pub save_count: usize,
    /// The freshest file-modified time found recursively — the score.
    #[serde(skip)]
    pub freshest: SystemTime,
    /// Milliseconds since the epoch of `freshest`, for the frontend.
    pub freshest_epoch_ms: u64,
}

/// Every save root discovery could find, freshest first. `roots[0]` is the
/// chosen one; the rest are the switch (RD-4's "never silently invisible").
pub fn discover_roots(
    steam_root: Option<&Path>,
    documents_dir: Option<&Path>,
) -> Vec<CandidateRoot> {
    let mut candidates = Vec::new();
    if let Some(steam) = steam_root {
        candidates.extend(steam_cloud_candidates(steam));
    }
    if let Some(documents) = documents_dir {
        let docs_root = documents.join("My Games").join("Grim Dawn").join("save");
        if docs_root.is_dir() {
            candidates.push(docs_root);
        }
    }
    let mut scored: Vec<CandidateRoot> = candidates.iter().filter_map(|c| score_root(c)).collect();
    scored.sort_by(|a, b| b.freshest.cmp(&a.freshest));
    scored
}

/// `userdata/<every steamid>/219990/remote/save/` — ALL profiles, not just
/// the current login: a second Steam profile's hoard must never be silently
/// invisible (the 1B ruling).
fn steam_cloud_candidates(steam_root: &Path) -> Vec<PathBuf> {
    let userdata = steam_root.join("userdata");
    let Ok(entries) = std::fs::read_dir(&userdata) else {
        return Vec::new();
    };
    let mut roots: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| {
            e.path()
                .join(GRIM_DAWN_STEAM_APP_ID)
                .join("remote")
                .join("save")
        })
        .filter(|p| p.is_dir())
        .collect();
    roots.sort();
    roots
}

/// Score one candidate root: recurse, count saves, keep the freshest mtime.
/// Returns None when the tree holds no save files at all.
fn score_root(root: &Path) -> Option<CandidateRoot> {
    let mut save_count = 0usize;
    let mut freshest = SystemTime::UNIX_EPOCH;
    walk_for_saves(root, 0, &mut save_count, &mut freshest);
    if save_count == 0 {
        return None;
    }
    let freshest_epoch_ms = freshest
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Some(CandidateRoot {
        path: root.to_path_buf(),
        save_count,
        freshest,
        freshest_epoch_ms,
    })
}

fn walk_for_saves(dir: &Path, depth: u8, save_count: &mut usize, freshest: &mut SystemTime) {
    // save/main/_<Name>/ is two levels down; 6 leaves headroom without
    // letting a symlink cycle walk the disk.
    if depth > 6 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            walk_for_saves(&path, depth + 1, save_count, freshest);
        } else if is_save_file(&path) {
            *save_count += 1;
            if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                if modified > *freshest {
                    *freshest = modified;
                }
            }
        }
    }
}

fn is_save_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("gdc") | Some("gst")
    )
}

/// Enumerate `save/main/_<Name>/player.gdc` under a chosen root — the
/// character files the aggregation loop (3.5A) walks. Sorted for stable
/// ordering across runs.
pub fn character_saves(root: &Path) -> Vec<PathBuf> {
    let main = root.join("main");
    let Ok(entries) = std::fs::read_dir(&main) else {
        return Vec::new();
    };
    let mut saves: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path().join("player.gdc"))
        .filter(|p| p.is_file())
        .collect();
    saves.sort();
    saves
}

/// The shared transfer stash at the root itself, if present.
pub fn transfer_stash(root: &Path) -> Option<PathBuf> {
    let path = root.join("transfer.gst");
    path.is_file().then_some(path)
}

/// The Steam install root for this machine. On Windows the registry names
/// it (discovery reads, never guesses first), with the conventional
/// `Program Files (x86)\Steam` as fallback. `SMUGGLERS_STEAM_ROOT` overrides
/// everywhere — the bench's own knob.
pub fn steam_install_root() -> Option<PathBuf> {
    if let Ok(bench) = std::env::var("SMUGGLERS_STEAM_ROOT") {
        let p = PathBuf::from(bench);
        return p.is_dir().then_some(p);
    }
    platform_steam_root()
}

#[cfg(windows)]
fn platform_steam_root() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let from_registry = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
        .and_then(|k| k.get_value::<String, _>("SteamPath"))
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_dir());
    from_registry.or_else(|| {
        let conventional = PathBuf::from("C:\\Program Files (x86)\\Steam");
        conventional.is_dir().then_some(conventional)
    })
}

#[cfg(not(windows))]
fn platform_steam_root() -> Option<PathBuf> {
    // Windows-only gadget (Known Limitation) — no macOS/Linux path exists or
    // is planned. The bench override above still serves development.
    None
}

/// The game install (`steamapps/common/Grim Dawn`) for the codex — checks
/// the Steam root's own library plus every library named by
/// `libraryfolders.vdf`, because a second drive is the common case the
/// brother test will meet. `SMUGGLERS_GAME_ROOT` overrides for the bench.
pub fn game_install_root(steam_root: Option<&Path>) -> Option<PathBuf> {
    if let Ok(bench) = std::env::var("SMUGGLERS_GAME_ROOT") {
        let p = PathBuf::from(bench);
        return p.is_dir().then_some(p);
    }
    let steam = steam_root?;
    let mut libraries = vec![steam.to_path_buf()];
    libraries.extend(steam_library_folders(steam));
    libraries
        .into_iter()
        .map(|lib| lib.join("steamapps").join("common").join("Grim Dawn"))
        .find(|p| p.join("database").join("database.arz").is_file())
}

/// Minimal `libraryfolders.vdf` scan: every `"path"  "<dir>"` line names a
/// Steam library. A full VDF parser is more machinery than two quoted
/// strings deserve.
fn steam_library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    let Ok(contents) = std::fs::read_to_string(&vdf) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("\"path\"")?.trim();
            let path = rest.trim_matches('"');
            (!path.is_empty()).then(|| PathBuf::from(path.replace("\\\\", "\\")))
        })
        .filter(|p| p.is_dir())
        .collect()
}
