//! Contraband — the item record itself, and the two v11 scars it carries.
//!
//! Every bag slot, equipment slot, and stash cell holds this same shape, and
//! this shape is exactly where the two most dangerous 1.2-era deltas live:
//!
//! 1. **v11 items** carry four extra trailing `int32`s after `stackCount`,
//!    before the grid coordinates. Miss them and the second item in any bag
//!    desyncs — the spike's tell was a string-length read of `1073741824`,
//!    which is the float `2.0` misread as a length prefix.
//! 2. **v11 stash tabs** carry a 20-byte trailer (five `int32`s, observed
//!    all-zero) after the item array — on shared AND personal stashes.
//!
//! The v11 gate lives HERE, in the shared reader, taking `block_version` as an
//! explicit parameter — `manifest.rs` and `warehouse.rs` both call the same
//! gated reader, so the traps can never be fixed in one caller and left broken
//! in the other (the 2A ruling).

use serde::{Deserialize, Serialize};

use crate::cipher::Cipher;
use crate::error::LedgerError;

/// Block versions at or above this carry the four trailing int32s on items
/// and the 20-byte trailer on stash tabs.
pub const V11_DELTAS_FROM: u32 = 11;

/// One item as the save stores it: record paths and seeds, never names.
/// Names are the codex's business (3A).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Contraband {
    pub base_name: String,
    pub prefix_name: String,
    pub suffix_name: String,
    pub modifier_name: String,
    pub transmute_name: String,
    pub seed: u32,
    pub component_name: String,
    pub relic_bonus: String,
    pub component_seed: u32,
    pub augment_name: String,
    pub unknown: u32,
    pub augment_seed: u32,
    pub var1: u32,
    pub stack_count: u32,
}

/// An item plus its grid position inside a bag or stash tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedContraband {
    #[serde(flatten)]
    pub item: Contraband,
    pub x: u32,
    pub y: u32,
}

/// One stash tab: its OWN parsed grid geometry plus its items. The frontend
/// renders cells from these dimensions, never from an assumed fixed size (4C).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StashTab {
    pub width: u32,
    pub height: u32,
    pub items: Vec<PlacedContraband>,
}

/// Read one item record. When `block_version >= 11` the four trailing int32s
/// are consumed and discarded — no named field, because neither reference nor
/// spike names their meaning, and guessing a wrong semantic is worse than
/// admitting it's unknown (the 2A ruling). Grid position is NOT read here —
/// its type differs per container (int in bags, float in stash tabs, a single
/// attached byte on equipment), so each caller reads its own.
pub fn parse_item(cipher: &mut Cipher, block_version: u32) -> Result<Contraband, LedgerError> {
    let item = Contraband {
        base_name: cipher.read_str()?,
        prefix_name: cipher.read_str()?,
        suffix_name: cipher.read_str()?,
        modifier_name: cipher.read_str()?,
        transmute_name: cipher.read_str()?,
        seed: cipher.read_u32()?,
        component_name: cipher.read_str()?,
        relic_bonus: cipher.read_str()?,
        component_seed: cipher.read_u32()?,
        augment_name: cipher.read_str()?,
        unknown: cipher.read_u32()?,
        augment_seed: cipher.read_u32()?,
        var1: cipher.read_u32()?,
        stack_count: cipher.read_u32()?,
    };
    if block_version >= V11_DELTAS_FROM {
        for _ in 0..4 {
            cipher.read_u32()?;
        }
    }
    Ok(item)
}

/// Read one stash tab (block 0): width, height, item array with float grid
/// coordinates, then — v11 — the 20-byte trailer consumed raw to block end.
/// Shared by BOTH consumers of the trailer logic: `warehouse.rs` for the
/// transfer stash and `manifest.rs` for personal character stashes.
pub fn parse_stash_tab(cipher: &mut Cipher, block_version: u32) -> Result<StashTab, LedgerError> {
    let (block, end) = cipher.block_start()?;
    if block != 0 {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("expected stash tab block 0, got {block}"),
        });
    }
    let width = cipher.read_u32()?;
    let height = cipher.read_u32()?;
    let count = cipher.read_u32()?;
    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let item = parse_item(cipher, block_version)?;
        let x = cipher.read_f32()? as u32;
        let y = cipher.read_f32()? as u32;
        items.push(PlacedContraband { item, x, y });
    }
    // v11 trailer (five int32s, observed all-zero) — cipher-safe raw consume,
    // per the 1A skip ruling. On pre-v11 tabs this is a no-op.
    cipher.consume_to(end)?;
    cipher.block_end(end)?;
    Ok(StashTab {
        width,
        height,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn it_should_skip_the_four_v11_trailing_ints_and_read_the_true_grid_coordinates() {
        // fixtures/item-v11-trap.bin: a hand-constructed v11 item whose four
        // trailing int32s are [7, 9, 7, 9] — the known-wrong x/y a parser
        // reads if it misses the skip — followed by the true coordinates
        // x=3, y=5 (as floats, stash-tab style, appended by the forge).
        let bytes = fixtures::bytes("item-v11-trap.bin");
        let mut cipher = Cipher::turn(&bytes).unwrap();
        let item = parse_item(&mut cipher, 11).expect("v11 item should parse");
        let x = cipher.read_f32().unwrap() as u32;
        let y = cipher.read_f32().unwrap() as u32;
        assert_eq!(item.stack_count, 12);
        assert_eq!(
            (x, y),
            (3, 5),
            "the true coordinates follow the skipped span"
        );
    }

    #[test]
    fn it_should_read_the_known_wrong_coordinates_when_the_skip_is_missed() {
        // The same fixture parsed as v5 proves the trap is real: without the
        // skip, the four trailing ints masquerade as x/y (and two leftovers).
        let bytes = fixtures::bytes("item-v11-trap.bin");
        let mut cipher = Cipher::turn(&bytes).unwrap();
        let item = parse_item(&mut cipher, 5).expect("field set itself parses either way");
        assert_eq!(item.stack_count, 12);
        let wrong_x = cipher.read_u32().unwrap();
        let wrong_y = cipher.read_u32().unwrap();
        assert_eq!(
            (wrong_x, wrong_y),
            (7, 9),
            "missing the skip serves the trap values as coordinates"
        );
    }

    #[test]
    fn it_should_round_trip_the_full_field_set_of_a_real_transfer_stash_item() {
        // fixtures/item-v11-real.bin: the full field set of a real item
        // captured from the investor's transfer.gst on 2026-08-16 and
        // re-encrypted under a fresh seed (anonymised: no surrounding save
        // data, no original seed). Expected values printed by the forge at
        // capture time.
        let bytes = fixtures::bytes("item-v11-real.bin");
        let expected = fixtures::real_item_expectation();
        let mut cipher = Cipher::turn(&bytes).unwrap();
        let item = parse_item(&mut cipher, 11).expect("real capture should parse");
        let x = cipher.read_f32().unwrap() as u32;
        let y = cipher.read_f32().unwrap() as u32;
        assert_eq!(item.base_name, expected.item.base_name);
        assert_eq!(item.prefix_name, expected.item.prefix_name);
        assert_eq!(item.suffix_name, expected.item.suffix_name);
        assert_eq!(item.component_name, expected.item.component_name);
        assert_eq!(item.augment_name, expected.item.augment_name);
        assert_eq!(item.seed, expected.item.seed);
        assert_eq!(item.stack_count, expected.item.stack_count);
        assert_eq!((x, y), (expected.x, expected.y));
    }

    #[test]
    fn it_should_consume_the_20_byte_trailer_on_a_v11_stash_tab() {
        // fixtures/stash-tab-v11.bin: one encrypted v11 tab block — width 10,
        // height 19, two items, the five-int32 all-zero trailer, block end.
        // parse_stash_tab consuming the trailer exactly is what block_end
        // asserts: a missed or half-eaten trailer fails the position check.
        let bytes = fixtures::bytes("stash-tab-v11.bin");
        let mut cipher = Cipher::turn(&bytes).unwrap();
        let tab = parse_stash_tab(&mut cipher, 11).expect("v11 tab should parse");
        assert_eq!((tab.width, tab.height), (10, 19));
        assert_eq!(tab.items.len(), 2);
        assert_eq!(
            cipher.remaining(),
            0,
            "trailer fully consumed, nothing left"
        );
    }
}
