//! The watch — fresh on next glance (5A, RD-5).
//!
//! Grim Dawn writes saves on exit and at checkpoints, and a single checkpoint
//! touches `player.gdc` and can touch `transfer.gst` in close succession.
//! This module wraps the `notify` crate on the discovered save directory and
//! coalesces each write burst into exactly ONE re-parse: every save-file
//! event re-arms a quiet-period timer, and only the timer's expiry fires the
//! callback. The investor never clicks refresh — there is nothing to click.
//!
//! Read-only always (RD-2): this module watches paths and never opens one.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::LedgerError;

/// The debounce window: a burst of save writes inside this many milliseconds
/// is one settling event, not several. Named here and tested by name (the
/// Phase 5 criterion tests the constant, not an undefined "window").
pub const SAVE_WRITE_DEBOUNCE_MS: u64 = 1_500;

fn is_save_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("gdc") | Some("gst")
    )
}

/// A live watch on one save root. Dropping it stops the watch.
pub struct SaveWatch {
    _watcher: RecommendedWatcher,
}

/// Watch `root` recursively; call `on_settled` exactly once per write burst,
/// after the burst has been quiet for [`SAVE_WRITE_DEBOUNCE_MS`].
pub fn watch_the_hoard<F>(root: &Path, on_settled: F) -> Result<SaveWatch, LedgerError>
where
    F: Fn() + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event {
            for path in event.paths {
                if is_save_path(&path) {
                    let _ = tx.send(path);
                }
            }
        }
    })
    .map_err(|e| LedgerError::Unreadable {
        path: root.display().to_string(),
        detail: format!("the watch could not be posted: {e}"),
    })?;
    watcher
        .watch(root, RecursiveMode::Recursive)
        .map_err(|e| LedgerError::Unreadable {
            path: root.display().to_string(),
            detail: format!("the watch could not be posted: {e}"),
        })?;

    std::thread::spawn(move || {
        settle_loop(
            &rx,
            Duration::from_millis(SAVE_WRITE_DEBOUNCE_MS),
            on_settled,
        );
    });

    Ok(SaveWatch { _watcher: watcher })
}

/// The debounce core, separated from the filesystem so the test suite can
/// drive it with synthetic event streams: block for the first event of a
/// burst, then keep absorbing events until the channel has been quiet for a
/// full `window` — THEN fire, once.
pub fn settle_loop<F>(rx: &mpsc::Receiver<PathBuf>, window: Duration, on_settled: F)
where
    F: Fn(),
{
    while rx.recv().is_ok() {
        // A burst has begun. Absorb everything until it goes quiet.
        loop {
            match rx.recv_timeout(window) {
                Ok(_more_of_the_burst) => continue,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    on_settled();
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}
