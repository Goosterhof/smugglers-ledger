//! The Ledger's failure vocabulary — every error crosses the Tauri bridge as
//! a serializable value with enough voice for 4D's flagged states to build on.
//!
//! Mechanism module: plain name by design (see the 1A naming ruling).

use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize, Clone)]
#[serde(tag = "kind", content = "detail")]
pub enum LedgerError {
    /// The save's structure desynced or the file is from a different game
    /// version — the flagged-character state, never fatal to the fleet (1C).
    #[error("the ledger won't turn: {detail}")]
    CipherWontTurn { detail: String },

    /// Filesystem trouble reading a save or install path.
    #[error("could not read {path}: {detail}")]
    Unreadable { path: String, detail: String },

    /// No save root could be discovered anywhere on this machine (RD-4's
    /// manual-picker front door).
    #[error("no hoard found on this machine")]
    NoHoardFound,

    /// The game install (database.arz and the text shelves) is missing or
    /// unreadable — the raw-path-mode state.
    #[error("the codex has no shelf: {detail}")]
    CodexShelfMissing { detail: String },
}

impl LedgerError {
    pub fn unreadable(path: &std::path::Path, err: &std::io::Error) -> Self {
        LedgerError::Unreadable {
            path: path.display().to_string(),
            detail: err.to_string(),
        }
    }
}
