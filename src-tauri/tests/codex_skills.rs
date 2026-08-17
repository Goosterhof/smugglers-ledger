//! The skill-graft criterion, end-to-end: an item record's skill fields —
//! "+N to <Skill>" grants (int-typed levels), mastery and all-skills grants,
//! and the Monster Infrequent `modifiedSkillName`/`modifierSkillName` pairs —
//! resolve into readable, searchable lines. Skill names come from the skill
//! record's own `skillDisplayName` tag, chasing `buffSkillName` indirection
//! when an aura hides its name one record deeper.
//!
//! The test forges a multi-record `.arz` + `Text_EN.arc` pair, which is why
//! it lives here: the codex itself never encodes either format.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use smugglers_ledger_lib::codex::Codex;

const ITEM: &str = "records/items/gearweapons/mi_scorcher.dbr";
const SKILL_BLADE_ARC: &str = "records/skills/playerclass01/blade_arc.dbr";
const SKILL_AURA: &str = "records/skills/playerclass03/aura_parent.dbr";
const SKILL_AURA_BUFF: &str = "records/skills/playerclass03/aura_buff.dbr";
const MASTERY: &str = "records/skills/playerclass01/mastery.dbr";
const MODIFIER: &str = "records/skills/itemskillsgdx2/modifiers/scorcher_mod.dbr";

/// One forged DBR field, in the arz encoding's three value types.
enum Field<'a> {
    Str(&'a str, &'a str),
    Int(&'a str, i32),
    Float(&'a str, f32),
}

fn intern(strings: &mut Vec<String>, s: &str) -> u32 {
    if let Some(i) = strings.iter().position(|x| x == s) {
        return i as u32;
    }
    strings.push(s.to_string());
    (strings.len() - 1) as u32
}

/// A minimal valid multi-record ARZ: header, LZ4-compressed record bodies,
/// record table, string table — the exact layout `codex.rs` walks.
fn forge_arz(records: &[(&str, Vec<Field>)]) -> Vec<u8> {
    let mut strings: Vec<String> = Vec::new();
    let mut bodies: Vec<(u32, Vec<u8>, usize)> = Vec::new();
    for (path, fields) in records {
        let path_idx = intern(&mut strings, path);
        let mut body = Vec::new();
        for field in fields {
            match field {
                Field::Str(name, value) => {
                    let name_idx = intern(&mut strings, name);
                    let value_idx = intern(&mut strings, value);
                    body.extend_from_slice(&2u16.to_le_bytes());
                    body.extend_from_slice(&1u16.to_le_bytes());
                    body.extend_from_slice(&name_idx.to_le_bytes());
                    body.extend_from_slice(&value_idx.to_le_bytes());
                }
                Field::Int(name, value) => {
                    let name_idx = intern(&mut strings, name);
                    body.extend_from_slice(&0u16.to_le_bytes());
                    body.extend_from_slice(&1u16.to_le_bytes());
                    body.extend_from_slice(&name_idx.to_le_bytes());
                    body.extend_from_slice(&value.to_le_bytes());
                }
                Field::Float(name, value) => {
                    let name_idx = intern(&mut strings, name);
                    body.extend_from_slice(&1u16.to_le_bytes());
                    body.extend_from_slice(&1u16.to_le_bytes());
                    body.extend_from_slice(&name_idx.to_le_bytes());
                    body.extend_from_slice(&value.to_bits().to_le_bytes());
                }
            }
        }
        let compressed = lz4_flex::block::compress(&body);
        let decompressed = body.len();
        bodies.push((path_idx, compressed, decompressed));
    }

    // Layout: [24-byte header][record bodies][record table][string table];
    // a record's table offset is relative such that data sits at offset + 24.
    let mut record_data = Vec::new();
    let mut record_table = Vec::new();
    for (path_idx, compressed, decompressed) in &bodies {
        record_table.extend_from_slice(&path_idx.to_le_bytes());
        record_table.extend_from_slice(&0u32.to_le_bytes()); // type string, empty
        record_table.extend_from_slice(&(record_data.len() as u32).to_le_bytes());
        record_table.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        record_table.extend_from_slice(&(*decompressed as u32).to_le_bytes());
        record_table.extend_from_slice(&0u32.to_le_bytes()); // unknown
        record_table.extend_from_slice(&0u32.to_le_bytes()); // unknown2
        record_data.extend_from_slice(compressed);
    }

    let record_table_at = 24 + record_data.len();
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
    out.extend_from_slice(&(bodies.len() as u32).to_le_bytes());
    out.extend_from_slice(&(string_table_at as u32).to_le_bytes());
    out.extend_from_slice(&(string_table.len() as u32).to_le_bytes());
    out.extend_from_slice(&record_data);
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

fn forge_install(dir: &Path) {
    fs::create_dir_all(dir.join("database")).unwrap();
    fs::create_dir_all(dir.join("gdx1/database")).unwrap();
    fs::create_dir_all(dir.join("gdx2/database")).unwrap();
    fs::create_dir_all(dir.join("gdx3/database")).unwrap();
    fs::create_dir_all(dir.join("resources")).unwrap();

    // The Monster Infrequent under test: two plus-skill grants (one hiding
    // behind buff indirection), a mastery grant, an all-skills grant, and a
    // modifier pair pointing the modifier record's stats at Blade Arc.
    let main = forge_arz(&[
        (
            ITEM,
            vec![
                Field::Str("itemNameTag", "tagScorcher"),
                Field::Str("itemClassification", "Rare"),
                Field::Str("augmentSkillName1", SKILL_BLADE_ARC),
                Field::Int("augmentSkillLevel1", 2),
                Field::Str("augmentSkillName2", SKILL_AURA),
                Field::Int("augmentSkillLevel2", 1),
                Field::Str("augmentMasteryName1", MASTERY),
                Field::Int("augmentMasteryLevel1", 1),
                Field::Int("augmentAllLevel", 1),
                Field::Str("modifiedSkillName1", SKILL_BLADE_ARC),
                Field::Str("modifierSkillName1", MODIFIER),
            ],
        ),
        (
            SKILL_BLADE_ARC,
            vec![Field::Str("skillDisplayName", "tagBladeArc")],
        ),
        // The aura parent carries no display name of its own — the name lives
        // one record deeper, behind buffSkillName.
        (
            SKILL_AURA,
            vec![Field::Str("buffSkillName", SKILL_AURA_BUFF)],
        ),
        (
            SKILL_AURA_BUFF,
            vec![Field::Str("skillDisplayName", "tagAura")],
        ),
        (MASTERY, vec![Field::Str("skillDisplayName", "tagSoldier")]),
        (MODIFIER, vec![Field::Float("offensiveFireModifier", 70.0)]),
    ]);
    fs::write(dir.join("database/database.arz"), main).unwrap();

    // The three expansions: minimal valid shelves with one dummy record each.
    for (shelf, dummy) in [
        ("gdx1/database/GDX1.arz", "records/gdx1_dummy.dbr"),
        ("gdx2/database/GDX2.arz", "records/gdx2_dummy.dbr"),
        ("gdx3/database/GDX3.arz", "records/gdx3_dummy.dbr"),
    ] {
        fs::write(
            dir.join(shelf),
            forge_arz(&[(dummy, vec![Field::Str("itemNameTag", "tagUnused")])]),
        )
        .unwrap();
    }

    fs::write(
        dir.join("resources/Text_EN.arc"),
        forge_tiny_arc(
            "tagScorcher=Vilgazor's Scorcher\n\
             tagBladeArc=Blade Arc\n\
             tagAura=Mogdrogen's Pact\n\
             tagSoldier=Soldier\n",
        ),
    )
    .unwrap();
}

#[test]
fn it_should_resolve_the_skill_grafts_into_readable_searchable_lines() {
    let install = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    forge_install(install.path());

    let mut codex = Codex::open(install.path(), cache.path()).unwrap();
    let worklist: HashSet<String> = [ITEM.to_string()].into_iter().collect();
    let resolved = codex.resolve(&worklist).unwrap();

    let record = &resolved[ITEM];
    assert_eq!(record.name.as_deref(), Some("Vilgazor's Scorcher"));
    let rendered: Vec<String> = record
        .skills
        .iter()
        .map(|l| format!("{} {}", l.magnitude, l.label))
        .collect();

    // The plus-skill grant, its int-typed level read as an int.
    assert!(
        rendered.contains(&"+2 to Blade Arc".to_string()),
        "grant missing from {rendered:?}"
    );
    // The aura grant, its name chased through buffSkillName indirection.
    assert!(
        rendered.contains(&"+1 to Mogdrogen's Pact".to_string()),
        "buff-indirected grant missing from {rendered:?}"
    );
    // Mastery and all-skills grants.
    assert!(rendered.contains(&"+1 to All Skills in Soldier".to_string()));
    assert!(rendered.contains(&"+1 to All Skills".to_string()));
    // The Monster Infrequent case: the modifier record's own stats, suffixed
    // onto the skill they modify — the line the skill search matches.
    assert!(
        rendered.contains(&"+70% Fire Damage to Blade Arc".to_string()),
        "modifier line missing from {rendered:?}"
    );
}
