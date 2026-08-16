//! The watch criterion (5A): modifying a save file on disk while the watch
//! is posted triggers exactly ONE re-parse — not zero and not several —
//! within the `SAVE_WRITE_DEBOUNCE_MS` (1,500 ms) window the constant names.
//! The criterion tests the constant, not an undefined "window".
//!
//! These tests write files, which is why they live here — `watch.rs` itself
//! is swept by the RD-2 read-only gate.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use smugglers_ledger_lib::watch::{settle_loop, watch_the_hoard, SAVE_WRITE_DEBOUNCE_MS};

#[test]
fn it_should_name_the_constant_the_criterion_tests() {
    assert_eq!(SAVE_WRITE_DEBOUNCE_MS, 1_500);
}

/// The debounce core, driven synthetically: a burst of five events inside
/// the window fires the callback exactly once; a second burst fires it again.
#[test]
fn it_should_settle_each_burst_into_exactly_one_firing() {
    let (tx, rx) = mpsc::channel();
    let fired = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&fired);
    let handle = std::thread::spawn(move || {
        settle_loop(&rx, Duration::from_millis(80), move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
    });

    // Burst one: five writes in quick succession.
    for _ in 0..5 {
        tx.send(std::path::PathBuf::from("player.gdc")).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(fired.load(Ordering::SeqCst), 1, "one burst, one firing");

    // Burst two: the loop re-arms.
    tx.send(std::path::PathBuf::from("transfer.gst")).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(fired.load(Ordering::SeqCst), 2, "a new burst fires anew");

    drop(tx);
    handle.join().unwrap();
}

/// The full criterion against the real filesystem and the REAL constant: a
/// fixture save modified twice in close succession while the watch is
/// posted → exactly one settling inside the named window (plus scheduling
/// slack), not zero and not several.
#[test]
fn it_should_reparse_exactly_once_per_write_burst_within_the_named_window() {
    let root = tempfile::tempdir().unwrap();
    let save = root.path().join("transfer.gst");
    std::fs::write(&save, b"before").unwrap();

    let fired = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&fired);
    let _watch = watch_the_hoard(root.path(), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })
    .expect("the watch should post");

    // Give the watcher a beat to arm, then burst: two writes plus a touch of
    // a non-save file (which must not count as a save event at all).
    std::thread::sleep(Duration::from_millis(300));
    std::fs::write(&save, b"the game wrote a checkpoint").unwrap();
    std::thread::sleep(Duration::from_millis(120));
    std::fs::write(&save, b"and touched the file again").unwrap();
    std::fs::write(root.path().join("notes.txt"), b"not a save").unwrap();

    // Inside the window: nothing has fired yet (debounced, not per-event).
    std::thread::sleep(Duration::from_millis(SAVE_WRITE_DEBOUNCE_MS / 2));
    assert_eq!(
        fired.load(Ordering::SeqCst),
        0,
        "the burst is still settling — firing early would mean re-parsing per event"
    );

    // Past the window (plus slack): exactly one re-parse.
    std::thread::sleep(Duration::from_millis(SAVE_WRITE_DEBOUNCE_MS + 700));
    assert_eq!(
        fired.load(Ordering::SeqCst),
        1,
        "exactly one re-parse per burst — not zero, not several"
    );
}
