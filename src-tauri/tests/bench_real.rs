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

    // Stats — how many records resolved stat lines, and a sample.
    let with_stats = resolved.values().filter(|r| !r.stats.is_empty()).count();
    println!(
        "STAT COVERAGE: {with_stats} of {} resolved records carry stat lines",
        resolved.len()
    );
    let mut samples = 0;
    for (path, r) in &resolved {
        if r.stats.len() >= 3 && samples < 4 {
            println!("  {path}:");
            for line in &r.stats {
                println!("    {} {}", line.magnitude, line.label);
            }
            samples += 1;
        }
    }

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

/// [BENCH] Icons — extract and decode a real item bitmap from Items.arc.
#[test]
#[ignore = "bench only — needs the real install"]
fn bench_decode_a_real_icon() {
    use smugglers_ledger_lib::icons::IconShelf;
    let install = install_root();
    let data = std::fs::read(install.join("resources/Items.arc")).expect("read Items.arc");
    let shelf = IconShelf::open(data).expect("open Items.arc");
    // A known base medal bitmap (from the record dump).
    let bitmap = "chests/breakables/boozecrate01_dif.tex";
    let png = shelf
        .icon_png(bitmap)
        .expect("icon_png ok")
        .expect("bitmap present");
    println!("decoded {bitmap}: {} PNG bytes", png.len());
    assert_eq!(&png[1..4], b"PNG", "output is a PNG");
    // Decode the PNG back to confirm it's a real, non-empty image.
    let img = image::load_from_memory(&png).expect("valid PNG");
    println!("  dimensions: {}x{}", img.width(), img.height());
    assert!(img.width() >= 16 && img.height() >= 16, "a real icon");
}

/// [BENCH] THE TRADES — every mastery panel in the install reads: ten trees,
/// named, with their skills placed at the game's own coordinates and their
/// connections derived from the game's own connector runs. Reports the shape
/// of what it read; asserts only what must never regress.
#[test]
#[ignore = "bench only — needs the real install"]
fn bench_read_the_trades_from_the_real_install() {
    let install = install_root();
    let cache = std::env::temp_dir().join("smugglers-bench-trades");
    let _ = std::fs::remove_dir_all(&cache);
    let mut codex = Codex::open(&install, &cache).expect("open the codex");

    let cold = Instant::now();
    let trees = codex.trades().expect("read the trades");
    let cold_ms = cold.elapsed().as_millis();
    let warm = Instant::now();
    let again = Codex::open(&install, &cache)
        .expect("reopen")
        .trades()
        .expect("read the trades from cache");
    let warm_ms = warm.elapsed().as_millis();

    println!(
        "trades: {} masteries — cold {cold_ms} ms, warm {warm_ms} ms",
        trees.len()
    );
    assert_eq!(again.len(), trees.len(), "the cache serves the same trees");

    let mut total_nodes = 0;
    let mut unnamed = 0;
    let mut connected = 0;
    let mut iconless = 0;
    for tree in &trees {
        let roots = tree.nodes.iter().filter(|n| n.parent.is_none()).count();
        let linked = tree.nodes.len() - roots;
        total_nodes += tree.nodes.len();
        connected += linked;
        unnamed += tree
            .nodes
            .iter()
            .filter(|n| n.name.starts_with("records/"))
            .count();
        iconless += tree.nodes.iter().filter(|n| n.icon.is_none()).count();
        println!(
            "  {:>2} {:<14} {:>2} skills — {roots} roots, {linked} linked, bar {} (max {})",
            tree.class_index,
            tree.name,
            tree.nodes.len(),
            tree.bar_record,
            tree.bar_max_level,
        );
        assert!(!tree.name.is_empty(), "every mastery is named");
        assert!(!tree.nodes.is_empty(), "every mastery has skills");
        assert!(
            tree.bar_record.contains("_classtraining_"),
            "the bar is the mastery record a save carries a level for"
        );
        assert!(
            tree.nodes.iter().all(|n| (1..=9).contains(&n.tier)),
            "every skill sits in one of the nine columns"
        );
    }
    println!(
        "  {total_nodes} skills — {connected} on a drawn line, {unnamed} unnamed, {iconless} without an icon"
    );
    assert!(
        trees.len() >= 9,
        "the base game alone ships six, the expansions the rest"
    );
    assert_eq!(unnamed, 0, "every skill resolves a display name");
    assert!(
        connected * 2 > total_nodes,
        "most skills hang off something — a tree with no lines is a list"
    );

    // One of Soldier's own rows, checked against the panel by hand.
    let soldier = trees.iter().find(|t| t.name == "Soldier").expect("Soldier");
    let cadence = soldier
        .nodes
        .iter()
        .find(|n| n.record.ends_with("/cadence1.dbr"))
        .expect("Cadence");
    println!(
        "  Cadence — tier {} (mastery {}), max {}, ultimate {}, icon {:?}",
        cadence.tier, cadence.unlock_level, cadence.max_level, cadence.ultimate_level, cadence.icon
    );
    assert_eq!(cadence.tier, 1);
    assert_eq!(cadence.unlock_level, 1);
    assert_eq!(cadence.parent, None, "Cadence is the root of its own row");
    let discord = soldier
        .nodes
        .iter()
        .find(|n| n.record.ends_with("/cadence1b.dbr"))
        .expect("Discord");
    assert_eq!(
        discord.parent.as_deref(),
        Some(cadence.record.as_str()),
        "the transmuter hangs off Cadence"
    );
    println!("  Discord — {:?}", discord.conversion);
}

/// [BENCH] The skill icons decode — they are NOT block-compressed like the
/// item icons, and the flat-surface path is what makes them readable.
#[test]
#[ignore = "bench only — needs the real install"]
fn bench_decode_a_real_skill_icon() {
    use smugglers_ledger_lib::icons::{Cabinet, IconState};
    let install = install_root();
    let icons = IconState::default();
    // Both flat depths, one from each row of the same panel: Cadence's icon is
    // 32-bit BGRA, Blitz's is 24-bit BGR with no alpha channel at all.
    for bitmap in [
        "ui/skills/icons/class01/skillicon_cadence1_up.tex",
        "ui/skills/icons/class01/skillicon_blitz1_up.tex",
    ] {
        let png = icons
            .icon_png(&install, Cabinet::Ui, bitmap)
            .unwrap_or_else(|| panic!("{bitmap} decodes"));
        let img = image::load_from_memory(&png).expect("valid PNG");
        println!(
            "decoded {bitmap}: {} PNG bytes, {}x{}",
            png.len(),
            img.width(),
            img.height()
        );
        assert!(img.width() >= 16 && img.height() >= 16);
    }
}

/// [BENCH] The build overlay — every real hand's allocated skills read out of
/// block 8, and the gear they are wearing grafts ranks on top.
#[test]
#[ignore = "bench only — needs the real save set AND the real install"]
fn bench_read_the_real_builds() {
    let mut hoard = assemble_hoard(&save_root());
    let records = distinct_records(&hoard);
    let cache = std::env::temp_dir().join("smugglers-bench-builds");
    let mut codex = Codex::open(&install_root(), &cache).expect("open the codex");
    hoard.resolved = codex.resolve(&records).expect("resolve");

    let builds = smugglers_ledger_lib::ledger::hand_builds(&hoard);
    let mut with_skills = 0;
    for build in &builds {
        let bars: Vec<String> = build
            .allocated
            .iter()
            .filter(|r| r.record.contains("_classtraining_"))
            .map(|r| {
                format!(
                    "{}={}",
                    r.record.rsplit('/').next().unwrap_or_default(),
                    r.level
                )
            })
            .collect();
        println!(
            "{:<22} lvl {:>2} [{}] — {} skills, {} gear grafts, {} mastery grafts, +{} to all — bars {}",
            build.hand,
            build.level,
            build.class_tag,
            build.allocated.len(),
            build.granted.len(),
            build.mastery_granted.len(),
            build.all_granted,
            bars.join(" ")
        );
        if !build.allocated.is_empty() {
            with_skills += 1;
        }
        // A hand with no class tag has not picked a mastery yet — it still
        // carries the engine's own default skills, and no bar.
        if !build.class_tag.is_empty() {
            assert!(
                !bars.is_empty(),
                "a hand that picked a mastery bought a bar"
            );
        }
    }
    assert_eq!(
        with_skills,
        builds.len(),
        "every hand that parses reads its skills block"
    );
}
