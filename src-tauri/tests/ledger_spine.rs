//! The spine's own criteria (3.5A + Phase 4's location contract): the
//! enumeration loop over a fixture root, the flagged-not-fatal path, the
//! three command projections, and the 100%-location search contract.
//!
//! These tests build fixture trees (copying the committed byte fixtures into
//! a temp root), which is why they live here — `ledger.rs` itself is swept
//! by the RD-2 read-only gate and never writes.

use std::fs;
use std::path::{Path, PathBuf};

use smugglers_ledger_lib::ledger::{
    assemble_hoard, character_sheets, search_hoard, warehouse_sheet,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

/// A root with two healthy hands, one deliberately corrupt hand, and the
/// shared transfer stash.
fn build_fixture_root(dir: &Path) {
    fs::create_dir_all(dir.join("main/_Alpha")).unwrap();
    fs::create_dir_all(dir.join("main/_Bravo")).unwrap();
    fs::create_dir_all(dir.join("main/_Corrupt")).unwrap();
    fs::copy(
        fixture("player-v8-inv11.gdc"),
        dir.join("main/_Alpha/player.gdc"),
    )
    .unwrap();
    fs::copy(
        fixture("player-v8-inv11.gdc"),
        dir.join("main/_Bravo/player.gdc"),
    )
    .unwrap();
    fs::copy(
        fixture("player-corrupt.gdc"),
        dir.join("main/_Corrupt/player.gdc"),
    )
    .unwrap();
    fs::copy(fixture("transfer-v11.gst"), dir.join("transfer.gst")).unwrap();
}

/// Site 1: the enumeration loop — every `save/main/_<Name>/player.gdc` plus
/// the root-level `.gst`, aggregated into one state and served by all three
/// command projections.
#[test]
fn it_should_enumerate_the_root_and_serve_all_three_commands() {
    let root = tempfile::tempdir().unwrap();
    build_fixture_root(root.path());

    let hoard = assemble_hoard(root.path());

    // list_characters
    let sheets = character_sheets(&hoard);
    assert_eq!(sheets.len(), 3, "every hand is on the rail, healthy or not");
    let healthy: Vec<_> = sheets.iter().filter(|s| s.flagged.is_none()).collect();
    assert_eq!(healthy.len(), 2);
    assert_eq!(healthy[0].name, "Fixture");
    assert_eq!(healthy[0].equipment.len(), 12);
    assert_eq!(healthy[0].weapon_set_1.len(), 2);
    assert_eq!(healthy[0].weapon_set_2.len(), 2);

    // list_stash
    let warehouse = warehouse_sheet(&hoard);
    assert_eq!(warehouse.tabs.len(), 2);
    assert_eq!(warehouse.tabs[0].width, 10);
    assert_eq!(warehouse.tabs[0].height, 19);

    // search_ledger
    let hits = search_hoard(&hoard, "ectoplasm");
    assert!(
        !hits.is_empty(),
        "the fixture stash and personal stashes carry ectoplasm records"
    );
}

/// Site 2: the flagged-not-fatal path — one corrupt file among healthy ones
/// flags its own hand and sinks nothing else.
#[test]
fn it_should_flag_the_corrupt_hand_without_sinking_the_fleet() {
    let root = tempfile::tempdir().unwrap();
    build_fixture_root(root.path());

    let hoard = assemble_hoard(root.path());
    let sheets = character_sheets(&hoard);

    let corrupt: Vec<_> = sheets.iter().filter(|s| s.flagged.is_some()).collect();
    assert_eq!(corrupt.len(), 1, "exactly the one corrupt file is flagged");
    assert_eq!(
        corrupt[0].name, "Corrupt",
        "the flagged hand keeps its directory identity so the investor knows WHICH save broke"
    );
    assert!(
        corrupt[0]
            .flagged
            .as_deref()
            .unwrap()
            .contains("the ledger won't turn"),
        "the flag carries the voiced error, not a stack trace"
    );

    // And the fleet sailed: both healthy hands parsed fully.
    assert_eq!(sheets.iter().filter(|s| s.flagged.is_none()).count(), 2);
    assert!(warehouse_sheet(&hoard).flagged.is_none());
}

/// The location-string contract: 100% of results carry a location naming
/// character-or-stash and slot/tab — asserted against a fixture aggregate
/// covering every container kind the hoard has.
#[test]
fn it_should_put_a_location_on_every_single_search_result() {
    let root = tempfile::tempdir().unwrap();
    build_fixture_root(root.path());
    let hoard = assemble_hoard(root.path());

    // "records" matches every fixture item by record path — the widest net
    // the search can cast: bags, equipment, personal stash, shared stash.
    let hits = search_hoard(&hoard, "records");
    // 2 healthy hands × (1 bag item + 1 equipped head + 1 personal-stash
    // item) + 3 shared-stash items = 9.
    assert_eq!(
        hits.len(),
        9,
        "every container kind is represented in the net"
    );
    for hit in &hits {
        assert!(
            !hit.location.is_empty(),
            "the contract is absolute: {hit:?} has no location"
        );
        assert!(
            hit.location.contains("—"),
            "a location names its owner AND its place: {}",
            hit.location
        );
    }
    // Every container kind produced its own location vocabulary.
    let all = hits
        .iter()
        .map(|h| h.location.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("BAGS, BAG 1"), "bag locations present:\n{all}");
    assert!(
        all.contains("EQUIPPED, HEAD"),
        "equipment locations present:\n{all}"
    );
    assert!(
        all.contains("PERSONAL STASH, TAB 1"),
        "personal stash present:\n{all}"
    );
    assert!(
        all.contains("SHARED STASH — TAB"),
        "shared stash present:\n{all}"
    );
}

/// Unresolved items stay searchable by the same string the UI shows for them
/// (the 4C ruling): with no codex at all, the raw record path matches.
#[test]
fn it_should_find_unresolved_contraband_by_its_raw_record_path() {
    let root = tempfile::tempdir().unwrap();
    build_fixture_root(root.path());
    let hoard = assemble_hoard(root.path()); // no codex: nothing resolved

    let hits = search_hoard(&hoard, "compa_ectoplasm");
    assert!(!hits.is_empty());
    for hit in &hits {
        assert!(hit.name.is_none(), "nothing is resolved without a codex");
        assert!(hit.record_path.contains("compa_ectoplasm"));
        assert!(!hit.location.is_empty());
    }
}
