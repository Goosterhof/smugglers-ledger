//! The [BENCH] criteria — run manually on the Scientist's bench against the
//! REAL save set and game install, never in the Sentinel (which must never
//! see a save file). Each test is `#[ignore]` and env-gated:
//!
//! ```sh
//! SMUGGLERS_BENCH_SAVE_ROOT="/mnt/c/.../Steam/userdata/<id>/219990/remote/save" \
//! SMUGGLERS_BENCH_INSTALL_ROOT="/mnt/c/.../steamapps/common/Grim Dawn" \
//! cargo test --release --test bench_real -- --ignored --nocapture
//! ```
//!
//! Output transcripts are recorded in the experiment log's Build Progress.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use smugglers_ledger_lib::codex::Codex;
use smugglers_ledger_lib::ledger::{assemble_hoard, distinct_records, search_hoard};
use smugglers_ledger_lib::manifest::parse_player;
use smugglers_ledger_lib::warehouse::parse_stash;

fn save_root() -> PathBuf {
    PathBuf::from(
        std::env::var("SMUGGLERS_BENCH_SAVE_ROOT").expect("set SMUGGLERS_BENCH_SAVE_ROOT"),
    )
}

fn install_root() -> PathBuf {
    PathBuf::from(
        std::env::var("SMUGGLERS_BENCH_INSTALL_ROOT").expect("set SMUGGLERS_BENCH_INSTALL_ROOT"),
    )
}

/// [BENCH] Phase 1: every real `player.gdc` parses — header, info, bio,
/// inventory — and the real `transfer.gst` parses with per-tab geometry.
#[test]
#[ignore = "bench only — needs the real save set"]
fn bench_parse_the_real_save_set() {
    let root = save_root();
    let started = Instant::now();

    let stash_bytes = std::fs::read(root.join("transfer.gst")).unwrap();
    let stash = parse_stash(&stash_bytes).expect("transfer.gst should parse");
    let total: usize = stash.tabs.iter().map(|t| t.items.len()).sum();
    println!(
        "transfer.gst: v{}, {} tabs, {} items",
        stash.version,
        stash.tabs.len(),
        total
    );
    for (i, tab) in stash.tabs.iter().enumerate() {
        println!(
            "  tab {i} ({}x{}): {} items",
            tab.width,
            tab.height,
            tab.items.len()
        );
    }

    let mut ok = 0;
    let mut failed = 0;
    let mut dirs: Vec<_> = std::fs::read_dir(root.join("main"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for dir in dirs {
        let gdc = dir.join("player.gdc");
        if !gdc.is_file() {
            continue;
        }
        match std::fs::read(&gdc)
            .map_err(|e| e.to_string())
            .and_then(|b| parse_player(&b).map_err(|e| e.to_string()))
        {
            Ok(p) => {
                ok += 1;
                let bags: usize = p.inventory.sacks.iter().map(Vec::len).sum();
                let equipped = p
                    .inventory
                    .equipment
                    .iter()
                    .chain(&p.inventory.weapon_set_1)
                    .chain(&p.inventory.weapon_set_2)
                    .filter(|s| !s.item.base_name.is_empty())
                    .count();
                let personal: usize = p.personal_stash.iter().map(|t| t.items.len()).sum();
                println!(
                    "{}: level {} {} inv-v{} — bags {bags}, equipped {equipped}, personal stash {personal}, iron {}",
                    p.header.name,
                    p.header.level,
                    p.header.class_tag,
                    p.inventory.version,
                    p.info.iron
                );
            }
            Err(e) => {
                failed += 1;
                println!("{}: FAILED — {e}", dir.display());
            }
        }
    }
    println!(
        "RESULT: {ok} characters parsed clean, {failed} failed, in {:?}",
        started.elapsed()
    );
    assert_eq!(failed, 0, "every real save must parse");
}

/// [BENCH] Phase 3: the full-corpus resolve — display names for every
/// distinct base record, slot classes for every equippable, the
/// classification enumeration that settles the rarity mapping, and the
/// cold/warm timing numbers Phase 4's performance criterion wants.
#[test]
#[ignore = "bench only — needs the real save set AND the real install"]
fn bench_codex_full_corpus_resolve() {
    let root = save_root();
    let install = install_root();
    let cache = tempfile::tempdir().unwrap();

    let parse_started = Instant::now();
    let mut hoard = assemble_hoard(&root);
    let parse_elapsed = parse_started.elapsed();
    let records = distinct_records(&hoard);
    println!(
        "hoard assembled in {parse_elapsed:?}; {} distinct records",
        records.len()
    );

    // COLD: no cache exists — the full shelf walk.
    let cold_started = Instant::now();
    let mut codex = Codex::open(&install, cache.path()).unwrap();
    let resolved = codex.resolve(&records).unwrap();
    let cold_elapsed = cold_started.elapsed();
    println!(
        "COLD resolve: {cold_elapsed:?} for {} records",
        resolved.len()
    );

    let unresolved: BTreeSet<_> = resolved
        .iter()
        .filter(|(_, r)| r.name.is_none())
        .map(|(p, _)| p.clone())
        .collect();
    println!("unresolved: {} of {}", unresolved.len(), resolved.len());
    for path in &unresolved {
        println!("  UNRESOLVED (falls back to raw path in UI): {path}");
    }

    // The classification enumeration — Design System #00012 Open Question #1.
    let mut classifications: BTreeMap<String, usize> = BTreeMap::new();
    let mut slot_classes: BTreeSet<String> = BTreeSet::new();
    for r in resolved.values() {
        if let Some(c) = &r.classification {
            *classifications.entry(c.clone()).or_default() += 1;
        }
        if let Some(s) = &r.slot_class {
            slot_classes.insert(s.clone());
        }
    }
    println!("itemClassification enumeration (the rarity-mapping measurement):");
    for (c, n) in &classifications {
        println!("  {c}: {n}");
    }
    println!("equip-slot classes present: {slot_classes:?}");

    // WARM: a fresh Codex over the written cache — the 2-second promise's
    // half of the measurement (parse + cached resolve).
    let warm_started = Instant::now();
    let mut warm = Codex::open(&install, cache.path()).unwrap();
    assert!(
        warm.cache_answers(records.iter()),
        "warm cache answers everything"
    );
    let warm_resolved = warm.resolve(&records).unwrap();
    let warm_elapsed = warm_started.elapsed();
    println!(
        "WARM resolve: {warm_elapsed:?} for {} records (parse was {parse_elapsed:?}; warm total ≈ {:?})",
        warm_resolved.len(),
        parse_elapsed + warm_elapsed
    );

    hoard.resolved = resolved;
    let named = hoard.resolved.values().filter(|r| r.name.is_some()).count();
    println!("named: {named}/{}", hoard.resolved.len());
}

/// [BENCH] Phase 4: the search acceptance test — every "ectoplasm" stack
/// across ALL real locations, with a location on each, cross-referenced by
/// hand against the game's own UI.
#[test]
#[ignore = "bench only — needs the real save set AND the real install"]
fn bench_search_the_real_hoard_for_ectoplasm() {
    let root = save_root();
    let install = install_root();
    let cache = tempfile::tempdir().unwrap();

    let mut hoard = assemble_hoard(&root);
    let records: HashSet<String> = distinct_records(&hoard);
    let mut codex = Codex::open(&install, cache.path()).unwrap();
    hoard.resolved = codex.resolve(&records).unwrap();

    let hits = search_hoard(&hoard, "ectoplasm");
    println!("'ectoplasm' — {} stacks:", hits.len());
    let mut total = 0u32;
    for hit in &hits {
        total += hit.stack;
        println!(
            "  {} × {} — {}",
            hit.stack,
            hit.name.as_deref().unwrap_or(&hit.record_path),
            hit.location
        );
        assert!(!hit.location.is_empty());
    }
    println!("TOTAL IN HAND: {total}");
    assert!(
        !hits.is_empty(),
        "the investor's stash is known to hold ectoplasm"
    );
}
