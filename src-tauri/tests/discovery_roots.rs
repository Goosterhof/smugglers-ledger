//! Discovery against synthetic fixture directory trees (Phase 1B criterion,
//! 4 sites): the Steam Cloud shape, the Documents shape, freshest-recursive
//! scoring across multiple candidates, and the zero-candidate fallback.
//!
//! These tests CREATE files, which is why they live here and not inside
//! `discovery.rs` — that module is swept by the RD-2 read-only gate.

use std::fs;
use std::path::Path;

use filetime::{set_file_mtime, FileTime};
use smugglers_ledger_lib::discovery::discover_roots;

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"synthetic").unwrap();
}

fn age(path: &Path, secs_ago: i64) {
    let now = FileTime::now().unix_seconds();
    set_file_mtime(path, FileTime::from_unix_time(now - secs_ago, 0)).unwrap();
}

/// Site 1: the Steam Cloud shape — `userdata/*/219990/remote/save/` with the
/// `.gst` files at the root and characters one level down in `save/main/`.
#[test]
fn it_should_find_the_steam_cloud_shape() {
    let steam = tempfile::tempdir().unwrap();
    let save = steam.path().join("userdata/54202139/219990/remote/save");
    touch(&save.join("transfer.gst"));
    touch(&save.join("main/_Spinny/player.gdc"));

    let roots = discover_roots(Some(steam.path()), None);
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].path, save);
    assert_eq!(
        roots[0].save_count, 2,
        "the recursive walk counts the nested player.gdc a flat scan misses"
    );
}

/// Site 2: the Documents shape (the labelled assumption — synthetic until 5B
/// confirms it on real ground), nesting identically under `save/main/`.
#[test]
fn it_should_find_the_documents_shape() {
    let documents = tempfile::tempdir().unwrap();
    let save = documents.path().join("My Games/Grim Dawn/save");
    touch(&save.join("transfer.gst"));
    touch(&save.join("main/_Warder/player.gdc"));

    let roots = discover_roots(None, Some(documents.path()));
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].path, save);
    assert_eq!(roots[0].save_count, 2);
}

/// Site 3: multiple candidates scored by the freshest file-modified time
/// found RECURSIVELY — the stale root's fresh-looking `.gst` must not beat a
/// rival whose freshness hides one level down in `save/main/`.
#[test]
fn it_should_score_candidates_by_freshest_recursive_mtime() {
    let steam = tempfile::tempdir().unwrap();
    let stale = steam.path().join("userdata/11111111/219990/remote/save");
    let fresh = steam.path().join("userdata/99999999/219990/remote/save");

    // The stale profile's root-level stash is NEWER than the fresh profile's
    // root-level stash — only recursion into save/main/ reveals the truth.
    touch(&stale.join("transfer.gst"));
    age(&stale.join("transfer.gst"), 3_600);
    touch(&stale.join("main/_Old/player.gdc"));
    age(&stale.join("main/_Old/player.gdc"), 86_400);

    touch(&fresh.join("transfer.gst"));
    age(&fresh.join("transfer.gst"), 7_200);
    touch(&fresh.join("main/_Active/player.gdc"));
    age(&fresh.join("main/_Active/player.gdc"), 60); // freshest file anywhere

    let roots = discover_roots(Some(steam.path()), None);
    assert_eq!(
        roots.len(),
        2,
        "the losing root stays visible, not discarded"
    );
    assert_eq!(
        roots[0].path, fresh,
        "freshest-wins is decided by the recursive walk, not the root's own files"
    );
    assert_eq!(roots[1].path, stale);
}

/// Site 4: zero candidates — the empty result is the manual picker's front
/// door (the fallback, never the first screen).
#[test]
fn it_should_return_nothing_when_no_root_holds_saves() {
    let steam = tempfile::tempdir().unwrap();
    let documents = tempfile::tempdir().unwrap();
    // A userdata tree for another game entirely, and one Grim Dawn save dir
    // holding no save files at all.
    touch(&steam.path().join("userdata/54202139/440/remote/stuff.bin"));
    fs::create_dir_all(steam.path().join("userdata/54202139/219990/remote/save")).unwrap();

    let roots = discover_roots(Some(steam.path()), Some(documents.path()));
    assert!(
        roots.is_empty(),
        "no saves anywhere → the manual picker's case"
    );
}
