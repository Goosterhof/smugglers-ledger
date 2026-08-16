//! The warehouse — the shared transfer stash, walked tab by tab.
//!
//! `transfer.gst` is one block-18 file: version, mod string, expansion byte
//! (**7** on 1.2-era files — the stale references assert 3; we read and never
//! assert), then N tabs, each carrying its OWN grid geometry alongside its
//! items. The 4C frontend renders cell geometry from these parsed dimensions,
//! never from an assumed fixed tab size.
//!
//! Every block's shape is gated on THAT block's own version byte, never the
//! file's — the single most important lesson of the spike (delta #1).

use serde::Serialize;

use crate::cipher::Cipher;
use crate::contraband::{parse_stash_tab, StashTab};
use crate::error::LedgerError;

/// The plausibility corridor for stash block versions — outside it, the save
/// is from a game version the Ledger does not know, and says so.
const VERSION_FLOOR: u32 = 4;
const VERSION_CEILING: u32 = 15;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warehouse {
    pub version: u32,
    pub tabs: Vec<StashTab>,
}

/// Walk a `transfer.gst` byte stream into the shared warehouse.
pub fn parse_stash(data: &[u8]) -> Result<Warehouse, LedgerError> {
    let mut cipher = Cipher::turn(data)?;
    cipher.expect_u32(2, "stash magic")?;
    let (block, end) = cipher.block_start()?;
    if block != 18 {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("expected stash block 18, got {block}"),
        });
    }
    let version = cipher.read_u32()?;
    if !(VERSION_FLOOR..=VERSION_CEILING).contains(&version) {
        return Err(LedgerError::CipherWontTurn {
            detail: format!("implausible stash version {version} — a different game version?"),
        });
    }
    cipher.next_u32()?; // pad
    let _mod_name = cipher.read_str()?;
    let _expansion = cipher.read_byte()?; // 7 observed on 1.2 — read, never asserted
    let tab_count = cipher.read_u32()?;
    let mut tabs = Vec::with_capacity(tab_count as usize);
    for _ in 0..tab_count {
        tabs.push(parse_stash_tab(&mut cipher, version)?);
    }
    cipher.block_end(end)?;
    Ok(Warehouse { version, tabs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn it_should_walk_a_v11_stash_reading_each_tabs_own_grid_dimensions() {
        let bytes = fixtures::bytes("transfer-v11.gst");
        let warehouse = parse_stash(&bytes).expect("the fixture stash should parse");
        assert_eq!(warehouse.version, 11);
        assert_eq!(warehouse.tabs.len(), 2);
        // Each tab's geometry is ITS OWN, parsed — not an assumed constant.
        assert_eq!(
            (warehouse.tabs[0].width, warehouse.tabs[0].height),
            (10, 19)
        );
        assert_eq!((warehouse.tabs[1].width, warehouse.tabs[1].height), (8, 16));
        assert_eq!(warehouse.tabs[0].items.len(), 2);
        assert_eq!(warehouse.tabs[1].items.len(), 1);
    }

    #[test]
    fn it_should_consume_the_v11_trailer_on_the_shared_stash_consumer_path() {
        // Criterion: both consumers of the trailer logic asserted
        // independently. This is the shared-stash side; manifest.rs asserts
        // the personal-stash side. A missed trailer fails block_end's
        // position check, so a green parse IS the trailer assertion — and the
        // grid x/y below additionally pin that no item field bled into them.
        let bytes = fixtures::bytes("transfer-v11.gst");
        let warehouse = parse_stash(&bytes).unwrap();
        let first = &warehouse.tabs[0].items[0];
        let second = &warehouse.tabs[0].items[1];
        assert_eq!((first.x, first.y), (2, 3));
        assert_eq!((second.x, second.y), (3, 3));
    }

    #[test]
    fn it_should_flag_an_implausible_stash_version_instead_of_desyncing() {
        // Rewrite the fixture with a version outside the corridor.
        let mut s = fixtures::Scribe::new(0x0BAD_5EED);
        s.write_u32(2);
        let b = s.begin_block(18);
        s.write_u32(99); // implausible
        s.write_next_u32(0);
        s.write_str("");
        s.write_byte(7);
        s.write_u32(0);
        s.end_block(b);
        let err = parse_stash(&s.into_bytes()).unwrap_err();
        assert!(matches!(err, LedgerError::CipherWontTurn { .. }));
    }
}
