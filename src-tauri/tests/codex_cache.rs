//! The codex cache criterion, both directions (Phase 3, 2 sites):
//!
//! 1. hash unchanged → served from cache with NO re-read of `database.arz`
//!    — proven with a discriminating probe: a phantom record that exists
//!    ONLY in the cache resolves to its cached name; a shelf walk would have
//!    returned nothing for it.
//! 2. hash changed → the cache is invalidated and the shelves are actually
//!    re-read — the same phantom record comes back unresolved, and a real
//!    record resolves to the NEW database's value.
//!
//! The tests forge a minimal-but-valid `.arz` + `Text_EN.arc` pair, which is
//! why they live here: the codex itself never encodes either format.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use smugglers_ledger_lib::codex::{database_identity, Codex};

const RECORD: &str = "records/items/test_sabre.dbr";
const PHANTOM: &str = "records/items/phantom.dbr";

/// A minimal valid ARZ: header, one LZ4-compressed record, record table,
/// string table — the exact layout `codex.rs` walks.
fn forge_tiny_arz(name_tag: &str, classification: &str) -> Vec<u8> {
    let strings = vec![
        RECORD.to_string(),
        "itemNameTag".to_string(),
        name_tag.to_string(),
        "itemClassification".to_string(),
        classification.to_string(),
        "Class".to_string(),
        "ArmorProtective_Head".to_string(),
    ];
    let idx = |s: &str| strings.iter().position(|x| x == s).unwrap() as u32;

    // Record body: three string fields (type 2, count 1).
    let mut body = Vec::new();
    for (field, value) in [
        ("itemNameTag", name_tag),
        ("itemClassification", classification),
        ("Class", "ArmorProtective_Head"),
    ] {
        body.extend_from_slice(&2u16.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&idx(field).to_le_bytes());
        body.extend_from_slice(&idx(value).to_le_bytes());
    }
    let compressed = lz4_flex::block::compress(&body);

    // Layout: [24-byte header][record data][record table][string table]
    let record_data_at = 24usize;
    let record_table_at = record_data_at + compressed.len();

    let mut record_table = Vec::new();
    record_table.extend_from_slice(&idx(RECORD).to_le_bytes()); // filename idx
    record_table.extend_from_slice(&0u32.to_le_bytes()); // type string, empty
    record_table.extend_from_slice(&0u32.to_le_bytes()); // offset (data at offset+24)
    record_table.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    record_table.extend_from_slice(&(body.len() as u32).to_le_bytes());
    record_table.extend_from_slice(&0u32.to_le_bytes()); // unknown
    record_table.extend_from_slice(&0u32.to_le_bytes()); // unknown2

    let string_table_at = record_table_at + record_table.len();
    let mut string_table = Vec::new();
    string_table.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    for s in &strings {
        string_table.extend_from_slice(&(s.len() as u32).to_le_bytes());
        string_table.extend_from_slice(s.as_bytes());
    }

    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // unknown
    out.extend_from_slice(&3u16.to_le_bytes()); // version
    out.extend_from_slice(&(record_table_at as u32).to_le_bytes());
    out.extend_from_slice(&(record_table.len() as u32).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes()); // one record
    out.extend_from_slice(&(string_table_at as u32).to_le_bytes());
    out.extend_from_slice(&(string_table.len() as u32).to_le_bytes());
    out.extend_from_slice(&compressed);
    out.extend_from_slice(&record_table);
    out.extend_from_slice(&string_table);
    out
}

/// A minimal valid ARC v3 with one plain-stored localization file.
fn forge_tiny_arc(lines: &str) -> Vec<u8> {
    let contents = lines.as_bytes();
    let name = b"tags.txt";
    let header_len = 28usize;
    let record_table_at = header_len + contents.len();

    let mut out = Vec::new();
    out.extend_from_slice(&0x0043_5241u32.to_le_bytes()); // "ARC\0"
    out.extend_from_slice(&3u32.to_le_bytes()); // version
    out.extend_from_slice(&1u32.to_le_bytes()); // file entries
    out.extend_from_slice(&1u32.to_le_bytes()); // data records
    out.extend_from_slice(&0u32.to_le_bytes()); // record table size (no parts)
    out.extend_from_slice(&(name.len() as u32).to_le_bytes()); // string table size
    out.extend_from_slice(&(record_table_at as u32).to_le_bytes());
    out.extend_from_slice(contents);
    out.extend_from_slice(name); // string table
                                 // record header (44 bytes)
    out.extend_from_slice(&1u32.to_le_bytes()); // entry_type 1 = plain
    out.extend_from_slice(&(header_len as u32).to_le_bytes()); // offset
    out.extend_from_slice(&(contents.len() as u32).to_le_bytes()); // compressed
    out.extend_from_slice(&(contents.len() as u32).to_le_bytes()); // decompressed
    out.extend_from_slice(&0u32.to_le_bytes()); // adler
    out.extend_from_slice(&0i64.to_le_bytes()); // filetime
    out.extend_from_slice(&0u32.to_le_bytes()); // parts
    out.extend_from_slice(&0u32.to_le_bytes()); // first part
    out.extend_from_slice(&(name.len() as u32).to_le_bytes()); // string len
    out.extend_from_slice(&0u32.to_le_bytes()); // string offset
    out
}

fn forge_install(dir: &Path, name_tag_value: &str) {
    fs::create_dir_all(dir.join("database")).unwrap();
    fs::create_dir_all(dir.join("gdx1/database")).unwrap();
    fs::create_dir_all(dir.join("gdx2/database")).unwrap();
    fs::create_dir_all(dir.join("gdx3/database")).unwrap();
    fs::create_dir_all(dir.join("resources")).unwrap();
    fs::write(
        dir.join("database/database.arz"),
        forge_tiny_arz("tagTestSabre", "Epic"),
    )
    .unwrap();
    // The three expansions: minimal valid shelves with no extra records.
    fs::write(
        dir.join("gdx1/database/GDX1.arz"),
        forge_tiny_arz("tagUnused1", "Common"),
    )
    .unwrap();
    fs::write(
        dir.join("gdx2/database/GDX2.arz"),
        forge_tiny_arz("tagUnused2", "Common"),
    )
    .unwrap();
    fs::write(
        dir.join("gdx3/database/GDX3.arz"),
        forge_tiny_arz("tagUnused3", "Common"),
    )
    .unwrap();
    fs::write(
        dir.join("resources/Text_EN.arc"),
        forge_tiny_arc(&format!("tagTestSabre={name_tag_value}\n")),
    )
    .unwrap();
}

fn worklist() -> HashSet<String> {
    [RECORD.to_string(), PHANTOM.to_string()]
        .into_iter()
        .collect()
}

/// Direction 1: unchanged hash → cache serves, no shelf is re-read.
#[test]
fn it_should_serve_from_cache_without_rereading_the_shelves_when_the_hash_matches() {
    let install = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    forge_install(install.path(), "Test Sabre");

    // First resolve: cold — walks the shelves, writes the cache.
    let mut codex = Codex::open(install.path(), cache.path()).unwrap();
    let resolved = codex.resolve(&worklist()).unwrap();
    assert_eq!(resolved[RECORD].name.as_deref(), Some("Test Sabre"));
    assert_eq!(resolved[RECORD].tier, 3, "Epic → tier 3 on the ink ramp");
    assert_eq!(
        resolved[RECORD].slot_class.as_deref(),
        Some("ArmorProtective_Head")
    );
    assert!(
        resolved[PHANTOM].name.is_none(),
        "no shelf holds the phantom"
    );

    // Plant the discriminating probe: rewrite the cache's phantom entry with
    // a name only the cache could serve. The hash stays untouched.
    let cache_file = cache.path().join("codex-cache.json");
    let mut cached: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_file).unwrap()).unwrap();
    cached["entries"][PHANTOM]["name"] = serde_json::Value::String("From The Cache".into());
    fs::write(&cache_file, serde_json::to_vec(&cached).unwrap()).unwrap();

    // Second open: warm. If the codex re-read database.arz, the phantom would
    // come back unresolved — the cached marker proves no shelf was consulted.
    let mut warm = Codex::open(install.path(), cache.path()).unwrap();
    assert!(
        warm.cache_answers(worklist().iter()),
        "every path answerable from cache"
    );
    let served = warm.resolve(&worklist()).unwrap();
    assert_eq!(
        served[PHANTOM].name.as_deref(),
        Some("From The Cache"),
        "the marker only the cache holds came back — database.arz was never re-read"
    );
}

/// Direction 2: changed hash → the cache is discarded and the shelves are
/// actually re-resolved.
#[test]
fn it_should_invalidate_and_reresolve_when_the_database_hash_changes() {
    let install = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    forge_install(install.path(), "Test Sabre");

    // Cold resolve, then plant the same phantom marker.
    let mut codex = Codex::open(install.path(), cache.path()).unwrap();
    codex.resolve(&worklist()).unwrap();
    let cache_file = cache.path().join("codex-cache.json");
    let mut cached: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_file).unwrap()).unwrap();
    cached["entries"][PHANTOM]["name"] = serde_json::Value::String("From The Cache".into());
    fs::write(&cache_file, serde_json::to_vec(&cached).unwrap()).unwrap();

    // The game patches: the database changes on disk (a renamed sabre), so
    // the install's identity hash changes with it.
    let old_identity = database_identity(install.path()).unwrap();
    forge_install(install.path(), "Renamed Sabre");
    fs::write(
        install.path().join("database/database.arz"),
        forge_tiny_arz("tagTestSabre", "Legendary"),
    )
    .unwrap();
    let new_identity = database_identity(install.path()).unwrap();
    assert_ne!(old_identity, new_identity, "the patch changed the identity");

    // Re-open: the stale cache must be discarded wholesale.
    let mut patched = Codex::open(install.path(), cache.path()).unwrap();
    let resolved = patched.resolve(&worklist()).unwrap();
    assert_eq!(
        resolved[PHANTOM].name, None,
        "the planted marker is gone — the stale cache was not trusted"
    );
    assert_eq!(
        resolved[RECORD].name.as_deref(),
        Some("Renamed Sabre"),
        "the fresh resolve read the NEW database's value"
    );
    assert_eq!(
        resolved[RECORD].tier, 4,
        "Legendary → tier 4: re-read, not remembered"
    );
}
