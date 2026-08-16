//! The Smuggler's Ledger — entry library.
//!
//! One searchable overview of everything the investor owns across the whole
//! of Cairn. The Ledger turns the cipher on the hoard at startup, posts a
//! watch on the save root, and keeps the ledger current the moment Grim Dawn
//! writes to disk. It reads a save to know it and never for any other reason
//! — no refresh button exists because a ledger that needs to be told to
//! update itself isn't one you'd trust.

pub mod cipher;
pub mod codex;
pub mod contraband;
pub mod discovery;
pub mod error;
#[cfg(test)]
mod fixtures;
pub mod icons;
pub mod ledger;
pub mod manifest;
pub mod warehouse;
pub mod watch;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::LedgerError;
use crate::ledger::LedgerState;

/// The live watch on the chosen root — re-armed whenever the root changes.
#[derive(Default)]
pub struct WatchState {
    inner: Mutex<Option<watch::SaveWatch>>,
}

/// Turn the ledger, then (re)arm the watch on whichever root was chosen and
/// tell the frontend the page has been turned.
fn turn_and_post_watch(app: &AppHandle, root_override: Option<PathBuf>) -> Result<(), LedgerError> {
    let state = app.state::<LedgerState>();
    let stage_handle = app.clone();
    let result = ledger::turn_the_ledger(&state, root_override, move || {
        // Each stage store (parsed-raw, then resolved) turns the page.
        let _ = stage_handle.emit("ledger-turned", ());
    });

    let chosen = state
        .inner
        .lock()
        .expect("ledger state poisoned")
        .hoard
        .as_ref()
        .map(|h| h.root.clone());

    if let Some(root) = chosen {
        let handle = app.clone();
        let posted = watch::watch_the_hoard(&root, move || {
            // A save write settled (one event per burst): re-turn and announce.
            if turn_and_post_watch_quiet(&handle).is_ok() {
                let _ = handle.emit("save-changed", ());
            }
        })?;
        *app.state::<WatchState>()
            .inner
            .lock()
            .expect("watch state poisoned") = Some(posted);
    }

    result
}

/// Re-parse without re-arming the watch — the watch that fired stays posted.
fn turn_and_post_watch_quiet(app: &AppHandle) -> Result<(), LedgerError> {
    let state = app.state::<LedgerState>();
    let stage_handle = app.clone();
    ledger::turn_the_ledger(&state, current_root(app), move || {
        let _ = stage_handle.emit("ledger-turned", ());
    })
}

fn current_root(app: &AppHandle) -> Option<PathBuf> {
    app.state::<LedgerState>()
        .inner
        .lock()
        .expect("ledger state poisoned")
        .hoard
        .as_ref()
        .map(|h| h.root.clone())
}

/// Re-point the ledger at another discovered root, or at a manually picked
/// folder (the fallback, never the front door — RD-4).
#[tauri::command]
fn switch_root(app: AppHandle, path: PathBuf) -> Result<(), LedgerError> {
    turn_and_post_watch(&app, Some(path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // THE SHIPMENT's second half — the updater checks GitHub Releases for
        // a newer edition and verifies the minisign seal before install;
        // process drives the relaunch. The flow lives frontend-side in
        // src/shipment/; these are bare registration.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(LedgerState::default())
        .manage(WatchState::default())
        .manage(icons::IconState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            // The first turn runs off the main thread: the window paints
            // immediately with 4D's "Turning the cipher on the hoard…" state
            // while discovery and the parse work.
            std::thread::spawn(move || match turn_and_post_watch(&handle, None) {
                Ok(()) => log::info!("the ledger is turned — the hoard is on the page"),
                Err(err) => log::warn!("the ledger could not turn at startup: {err}"),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ledger::list_characters,
            ledger::list_stash,
            ledger::search_ledger,
            ledger::ledger_overview,
            ledger::item_icon,
            switch_root,
        ])
        .run(tauri::generate_context!())
        .expect("the ledger's window could not open");
}
