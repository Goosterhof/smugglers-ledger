//! The manifest — one character's full loadout, walked from one `player.gdc`.
//!
//! This module reads ONE file: header (name/level/class/hardcore), the
//! character-info block (iron, difficulty — loot-filter tail consumed raw to
//! block end, version-agnostic), the bio block (attributes), the inventory
//! block (bags, 12 equipment slots, both weapon sets), and the personal
//! stash. The loop across all ten characters belongs to `ledger.rs` (3.5A),
//! never to this module.
//!
//! Every block gates its shape on ITS OWN version byte, never the file's:
//! the file version stays **8** while inventory and stash report **11**, and
//! a parser gating on file version reads the 2018-era layout and desyncs
//! (delta #1 — the single most important lesson of the spike).

use serde::Serialize;

use crate::cipher::Cipher;
use crate::contraband::{parse_item, parse_stash_tab, Contraband, PlacedContraband, StashTab};
use crate::error::LedgerError;

const GDC_MAGIC: u32 = 0x5843_4447; // "GDCX"
const EQUIPMENT_SLOTS: usize = 12;
const WEAPON_SET_SLOTS: usize = 2;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestHeader {
    pub name: String,
    pub sex: u8,
    pub class_tag: String,
    pub level: u32,
    pub hardcore: bool,
    pub expansion: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterInfo {
    pub version: u32,
    pub difficulty: u8,
    pub greatest_difficulty: u8,
    pub iron: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bio {
    pub level: u32,
    pub experience: u32,
    pub attribute_points_unspent: u32,
    pub skill_points_unspent: u32,
    pub devotion_points_unspent: u32,
    pub total_devotion_unlocked: u32,
    pub physique: f32,
    pub cunning: f32,
    pub spirit: f32,
    pub health: f32,
    pub energy: f32,
}

/// An equipment or weapon slot: the item plus the game's own attached flag.
/// An empty slot is an item whose `base_name` is the empty string — exactly
/// as the game stores it (an unclaimed seat is an empty chair).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DressedSlot {
    pub item: Contraband,
    pub attached: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub version: u32,
    pub sacks: Vec<Vec<PlacedContraband>>,
    pub use_alternate: bool,
    pub equipment: Vec<DressedSlot>,
    pub weapon_set_1: Vec<DressedSlot>,
    pub weapon_set_2: Vec<DressedSlot>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub header: ManifestHeader,
    pub file_version: u32,
    pub info: CharacterInfo,
    pub bio: Bio,
    pub inventory: Inventory,
    pub personal_stash: Vec<StashTab>,
}

fn expect_block(cipher: &mut Cipher, want: u32) -> Result<usize, LedgerError> {
    let (block, end) = cipher.block_start()?;
    if block != want {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("expected block {want}, got {block}"),
        });
    }
    Ok(end)
}

fn version_corridor(version: u32, floor: u32, ceiling: u32, what: &str) -> Result<(), LedgerError> {
    if !(floor..=ceiling).contains(&version) {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("implausible {what} version {version} — a different game version?"),
        });
    }
    Ok(())
}

/// Walk one `player.gdc` byte stream into a full manifest. Parsing stops
/// after block 4 — the later blocks (skills, factions, quests…) are not the
/// Ledger's business and stay unread.
pub fn parse_player(data: &[u8]) -> Result<Manifest, LedgerError> {
    let mut cipher = Cipher::turn(data)?;
    cipher.expect_u32(GDC_MAGIC, "GDCX magic")?;
    cipher.expect_u32(2, "header version")?;
    let header = ManifestHeader {
        name: cipher.read_wstr()?,
        sex: cipher.read_byte()?,
        class_tag: cipher.read_str()?,
        level: cipher.read_u32()?,
        hardcore: cipher.read_byte()? != 0,
        expansion: cipher.read_byte()?,
    };
    let checksum = cipher.next_u32()?;
    if checksum != 0 {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("header checksum marker expected 0, got {checksum}"),
        });
    }
    let file_version = cipher.read_u32()?;
    version_corridor(file_version, 6, 12, "gdc file")?;
    for _ in 0..16 {
        cipher.read_byte()?; // uid
    }

    let info = parse_character_info(&mut cipher)?;
    let bio = parse_bio(&mut cipher)?;
    let inventory = parse_inventory(&mut cipher)?;
    let personal_stash = parse_personal_stash(&mut cipher)?;

    Ok(Manifest {
        header,
        file_version,
        info,
        bio,
        inventory,
        personal_stash,
    })
}

fn parse_character_info(cipher: &mut Cipher) -> Result<CharacterInfo, LedgerError> {
    let end = expect_block(cipher, 1)?;
    let version = cipher.read_u32()?;
    let _is_in_main_quest = cipher.read_byte()?;
    let _has_been_in_game = cipher.read_byte()?;
    let difficulty = cipher.read_byte()?;
    let greatest_difficulty = cipher.read_byte()?;
    let iron = cipher.read_u32()?;
    let _greatest_survival_difficulty = cipher.read_byte()?;
    let _current_tribute = cipher.read_u32()?;
    let _compass_state = cipher.read_byte()?;
    if (2..=4).contains(&version) {
        let _always_show_loot = cipher.read_u32()?;
    }
    let _skill_window_show_help = cipher.read_byte()?;
    let _weapon_swap_active = cipher.read_byte()?;
    let _weapon_swap_enabled = cipher.read_byte()?;
    let _texture = cipher.read_str()?;
    // Loot-filter tail: layout varies by version (the old refs read 39 fixed
    // bytes, gd-edit reads a count-prefixed array) — raw-consuming to block
    // end sidesteps the dispute, version-agnostically (the 1A skip ruling).
    cipher.consume_to(end)?;
    cipher.block_end(end)?;
    Ok(CharacterInfo {
        version,
        difficulty,
        greatest_difficulty,
        iron,
    })
}

fn parse_bio(cipher: &mut Cipher) -> Result<Bio, LedgerError> {
    let end = expect_block(cipher, 2)?;
    let version = cipher.read_u32()?;
    version_corridor(version, 7, 10, "bio")?;
    let bio = Bio {
        level: cipher.read_u32()?,
        experience: cipher.read_u32()?,
        attribute_points_unspent: cipher.read_u32()?,
        skill_points_unspent: cipher.read_u32()?,
        devotion_points_unspent: cipher.read_u32()?,
        total_devotion_unlocked: cipher.read_u32()?,
        physique: cipher.read_f32()?,
        cunning: cipher.read_f32()?,
        spirit: cipher.read_f32()?,
        health: cipher.read_f32()?,
        energy: cipher.read_f32()?,
    };
    cipher.block_end(end)?;
    Ok(bio)
}

fn parse_dressed_slots(
    cipher: &mut Cipher,
    count: usize,
    version: u32,
) -> Result<Vec<DressedSlot>, LedgerError> {
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        let item = parse_item(cipher, version)?;
        let attached = cipher.read_byte()? != 0;
        slots.push(DressedSlot { item, attached });
    }
    Ok(slots)
}

fn parse_inventory(cipher: &mut Cipher) -> Result<Inventory, LedgerError> {
    let end = expect_block(cipher, 3)?;
    // Gate on THIS block's version — the file version upstairs stays 8 while
    // this reports 11 (delta #1). parse_item receives it explicitly.
    let version = cipher.read_u32()?;
    version_corridor(version, 4, 12, "inventory")?;
    let mut inventory = Inventory {
        version,
        sacks: Vec::new(),
        use_alternate: false,
        equipment: Vec::new(),
        weapon_set_1: Vec::new(),
        weapon_set_2: Vec::new(),
    };
    if cipher.read_byte()? != 0 {
        let num_bags = cipher.read_u32()?;
        let _focused = cipher.read_u32()?;
        let _selected = cipher.read_u32()?;
        for _ in 0..num_bags {
            let sack_end = expect_block(cipher, 0)?;
            let _temp_bool = cipher.read_byte()?;
            let count = cipher.read_u32()?;
            let mut sack = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let item = parse_item(cipher, version)?;
                let x = cipher.read_u32()?; // ints in bags, floats in stashes
                let y = cipher.read_u32()?;
                sack.push(PlacedContraband { item, x, y });
            }
            cipher.consume_to(sack_end)?; // tolerate per-sack trailer fields
            cipher.block_end(sack_end)?;
            inventory.sacks.push(sack);
        }
        inventory.use_alternate = cipher.read_byte()? != 0;
        inventory.equipment = parse_dressed_slots(cipher, EQUIPMENT_SLOTS, version)?;
        let _alternate1 = cipher.read_byte()?;
        inventory.weapon_set_1 = parse_dressed_slots(cipher, WEAPON_SET_SLOTS, version)?;
        let _alternate2 = cipher.read_byte()?;
        inventory.weapon_set_2 = parse_dressed_slots(cipher, WEAPON_SET_SLOTS, version)?;
    }
    cipher.block_end(end)?;
    Ok(inventory)
}

fn parse_personal_stash(cipher: &mut Cipher) -> Result<Vec<StashTab>, LedgerError> {
    let end = expect_block(cipher, 4)?;
    let version = cipher.read_u32()?;
    version_corridor(version, 5, 12, "personal stash")?;
    let tab_count = cipher.read_u32()?;
    let mut tabs = Vec::with_capacity(tab_count as usize);
    for _ in 0..tab_count {
        tabs.push(parse_stash_tab(cipher, version)?);
    }
    cipher.block_end(end)?;
    Ok(tabs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    fn fixture_manifest() -> Manifest {
        let bytes = fixtures::bytes("player-v8-inv11.gdc");
        parse_player(&bytes).expect("the fixture player should parse")
    }

    #[test]
    fn it_should_gate_block_layout_per_block_not_per_file() {
        // The committed fixture carries FILE version 8 with an inventory
        // block reporting version 11 — asserted directly, not inferred from
        // the parse merely succeeding.
        let manifest = fixture_manifest();
        assert_eq!(manifest.file_version, 8, "file version stays 8");
        assert_eq!(
            manifest.inventory.version, 11,
            "inventory block reports its own version 11"
        );
        // And the v11 layout rules were USED: the bag item's four trap ints
        // [7,9,7,9] were skipped, so the true grid coordinates surface.
        let placed = &manifest.inventory.sacks[0][0];
        assert_eq!(
            (placed.x, placed.y),
            (2, 3),
            "v11 skip applied — a v5-gated parse would read x=7, y=9"
        );
    }

    #[test]
    fn it_should_read_header_info_and_bio_from_the_fixture() {
        let manifest = fixture_manifest();
        assert_eq!(manifest.header.name, "Fixture");
        assert_eq!(manifest.header.level, 42);
        assert!(!manifest.header.hardcore);
        assert_eq!(manifest.info.iron, 92_077);
        assert_eq!(manifest.bio.experience, 123_456);
        assert_eq!(manifest.bio.health, 2500.0);
    }

    #[test]
    fn it_should_dress_twelve_equipment_slots_and_both_weapon_sets() {
        let manifest = fixture_manifest();
        assert_eq!(manifest.inventory.equipment.len(), 12);
        assert_eq!(manifest.inventory.weapon_set_1.len(), 2);
        assert_eq!(manifest.inventory.weapon_set_2.len(), 2);
        assert!(manifest.inventory.equipment[0].attached);
        assert!(
            manifest.inventory.equipment[1].item.base_name.is_empty(),
            "an empty slot is an empty string, not a missing entry"
        );
    }

    #[test]
    fn it_should_consume_the_v11_trailer_on_the_personal_stash_consumer_path() {
        // The second, independent consumer of the shared trailer logic
        // (warehouse.rs asserts the shared-stash side). block_end's position
        // check fails if the 20-byte trailer is missed or half-eaten.
        let manifest = fixture_manifest();
        assert_eq!(manifest.personal_stash.len(), 1);
        let tab = &manifest.personal_stash[0];
        assert_eq!((tab.width, tab.height), (8, 16));
        assert_eq!(tab.items.len(), 1);
        assert_eq!((tab.items[0].x, tab.items[0].y), (4, 2));
    }

    #[test]
    fn it_should_refuse_a_corrupt_save_with_a_flaggable_error_not_a_panic() {
        let bytes = fixtures::bytes("player-corrupt.gdc");
        let err = parse_player(&bytes).unwrap_err();
        assert!(matches!(err, LedgerError::CipherWontTurn { .. }));
    }
}
