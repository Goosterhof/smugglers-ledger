//! Test-support only: fixture loading and the Scribe (the cipher's inverse).
//!
//! The Ledger itself NEVER encrypts and NEVER writes a save — the Scribe
//! exists solely under `#[cfg(test)]` so the committed byte fixtures in
//! `fixtures/` can be forged once (on the bench, against the real save set)
//! and regression-checked forever in CI without a byte of private save data
//! in the repo. See `fixtures/README.md` for the forge protocol.

use std::path::PathBuf;

use crate::contraband::PlacedContraband;

const KEY_SALT: u32 = 0x5555_5555;
const KEY_MULTIPLIER: u32 = 39_916_801;

pub fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

pub fn bytes(name: &str) -> Vec<u8> {
    let path = dir().join(name);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture {} unreadable ({e}) — run the forge: \
             `cargo test --release -- --ignored forge_fixtures`",
            path.display()
        )
    })
}

/// The expected field values for `item-v11-real.bin`, written by the forge at
/// capture time (`item-v11-real.expected.json`) so the round-trip asserts
/// against the real item's values, not against the parser's own output.
pub fn real_item_expectation() -> PlacedContraband {
    let raw = bytes("item-v11-real.expected.json");
    serde_json::from_slice(&raw).expect("expectation JSON should deserialize")
}

/// The cipher's inverse — encrypts plaintext into the exact byte stream
/// `Cipher` decrypts. Key evolution mirrors the reader: the rolling key is
/// updated from the RAW (encrypted) bytes, so the Scribe updates its key from
/// the bytes it emits.
pub struct Scribe {
    buf: Vec<u8>,
    key: u32,
    table: [u32; 256],
}

impl Scribe {
    /// `seed_raw` is stored verbatim as the first four bytes, exactly as a
    /// real save stores its seed.
    pub fn new(seed_raw: u32) -> Self {
        let key = seed_raw ^ KEY_SALT;
        let mut table = [0u32; 256];
        let mut k = key;
        for slot in &mut table {
            k = k.rotate_right(1).wrapping_mul(KEY_MULTIPLIER);
            *slot = k;
        }
        Self {
            buf: seed_raw.to_le_bytes().to_vec(),
            key,
            table,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn write_u32(&mut self, value: u32) {
        let raw = (value ^ self.key).to_le_bytes();
        self.buf.extend_from_slice(&raw);
        for b in raw {
            self.key ^= self.table[b as usize];
        }
    }

    /// No key update — the mirror of `Cipher::next_u32` (block lengths and
    /// end-of-block markers).
    pub fn write_next_u32(&mut self, value: u32) {
        let raw = (value ^ self.key).to_le_bytes();
        self.buf.extend_from_slice(&raw);
    }

    pub fn write_byte(&mut self, value: u8) {
        let raw = value ^ (self.key as u8);
        self.buf.push(raw);
        self.key ^= self.table[raw as usize];
    }

    pub fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        for b in s.bytes() {
            self.write_byte(b);
        }
    }

    pub fn write_wstr(&mut self, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        self.write_u32(units.len() as u32);
        for unit in units {
            self.write_byte(unit as u8);
            self.write_byte((unit >> 8) as u8);
        }
    }

    /// Open a block: writes the id, reserves the length word (a `next_u32`,
    /// which never updates the key — that is what makes late patching sound),
    /// and returns a handle for `end_block`.
    pub fn begin_block(&mut self, id: u32) -> BlockHandle {
        self.write_u32(id);
        let len_offset = self.buf.len();
        let key_snapshot = self.key;
        self.buf.extend_from_slice(&[0, 0, 0, 0]);
        BlockHandle {
            len_offset,
            key_snapshot,
        }
    }

    /// Close a block: patch the reserved length and write the 0 terminator.
    pub fn end_block(&mut self, handle: BlockHandle) {
        let body_len = (self.buf.len() - handle.len_offset - 4) as u32;
        let raw = (body_len ^ handle.key_snapshot).to_le_bytes();
        self.buf[handle.len_offset..handle.len_offset + 4].copy_from_slice(&raw);
        self.write_next_u32(0);
    }

    /// One item record in v11 shape: the full field set, four trailing ints,
    /// no position (callers append their own, matching their container).
    #[allow(clippy::too_many_arguments)]
    pub fn write_item_v11(&mut self, base: &str, stack: u32, seed: u32, ext: [u32; 4]) {
        self.write_str(base); // baseName
        self.write_str(""); // prefixName
        self.write_str(""); // suffixName
        self.write_str(""); // modifierName
        self.write_str(""); // transmuteName
        self.write_u32(seed);
        self.write_str(""); // componentName
        self.write_str(""); // relicBonus
        self.write_u32(0); // componentSeed
        self.write_str(""); // augmentName
        self.write_u32(0); // unknown
        self.write_u32(0); // augmentSeed
        self.write_u32(0); // var1
        self.write_u32(stack);
        for v in ext {
            self.write_u32(v);
        }
    }
}

pub struct BlockHandle {
    len_offset: usize,
    key_snapshot: u32,
}

/// Build the synthetic `transfer-v11.gst` byte stream: block 18, version 11,
/// expansion byte 7, two tabs (10x19 and 8x16) each carrying items and the
/// five-int32 all-zero trailer.
pub fn forge_transfer_v11() -> Vec<u8> {
    let mut s = Scribe::new(0x1BAD_C0DE);
    s.write_u32(2); // stash magic
    let stash = s.begin_block(18);
    s.write_u32(11); // stash block version — the v11 gate's input
    s.write_next_u32(0); // pad
    s.write_str(""); // mod
    s.write_byte(7); // expansion status: 7 on 1.2-era files, never asserted
    s.write_u32(2); // tab count
    for (w, h, items) in [(10u32, 19u32, 2u32), (8, 16, 1)] {
        let tab = s.begin_block(0);
        s.write_u32(w);
        s.write_u32(h);
        s.write_u32(items);
        for i in 0..items {
            s.write_item_v11(
                "records/items/materia/compa_ectoplasm.dbr",
                6 + i,
                0xAAA0 + i,
                [0, 1, 0, 0],
            );
            s.write_f32((2 + i) as f32); // x
            s.write_f32(3.0); // y
        }
        for _ in 0..5 {
            s.write_u32(0); // the 20-byte v11 tab trailer
        }
        s.end_block(tab);
    }
    s.end_block(stash);
    s.into_bytes()
}

/// Build a synthetic `player.gdc` whose FILE version is 8 while its inventory
/// and personal-stash blocks report version 11 — the per-block gating trap
/// (delta #1) as a committed fixture.
pub fn forge_player_v8_inv11(name: &str, level: u32) -> Vec<u8> {
    let mut s = Scribe::new(0x00DD_BA11 ^ level);
    s.write_u32(0x5843_4447); // "GDCX"
    s.write_u32(2); // header version
    s.write_wstr(name);
    s.write_byte(1); // sex
    s.write_str("tagSkillClassName06"); // classTag
    s.write_u32(level);
    s.write_byte(0); // hardcore
    s.write_byte(7); // expansion
    s.write_next_u32(0); // header checksum marker
    s.write_u32(8); // FILE version stays 8 — the trap
    for _ in 0..16 {
        s.write_byte(0xAB); // uid
    }

    // block 1: character_info v5 with a loot-filter tail of raw bytes
    let b1 = s.begin_block(1);
    s.write_u32(5); // info version
    s.write_byte(1); // isInMainQuest
    s.write_byte(1); // hasBeenInGame
    s.write_byte(1); // difficulty
    s.write_byte(2); // greatestDifficulty
    s.write_u32(92_077); // money
    s.write_byte(0); // greatestSurvivalDifficulty
    s.write_u32(0); // currentTribute
    s.write_byte(3); // compassState
    s.write_byte(1); // skillWindowShowHelp
    s.write_byte(0); // weaponSwapActive
    s.write_byte(1); // weaponSwapEnabled
    s.write_str("creatures/pc/hero02.tex");
    for i in 0..39 {
        s.write_byte(i); // loot-filter tail — consumed raw by the parser
    }
    s.end_block(b1);

    // block 2: bio v8
    let b2 = s.begin_block(2);
    s.write_u32(8);
    s.write_u32(level); // level
    s.write_u32(123_456); // experience
    s.write_u32(0); // attributePointsUnspent
    s.write_u32(3); // skillPointsUnspent
    s.write_u32(1); // devotionPointsUnspent
    s.write_u32(20); // totalDevotionUnlocked
    s.write_f32(400.0); // physique
    s.write_f32(300.0); // cunning
    s.write_f32(350.0); // spirit
    s.write_f32(2500.0); // health
    s.write_f32(1200.0); // energy
    s.end_block(b2);

    // block 3: inventory v11 — one bag whose item carries the trap ints
    let b3 = s.begin_block(3);
    s.write_u32(11); // INVENTORY block version 11 — gates the deltas
    s.write_byte(1); // flag: inventory present
    s.write_u32(1); // numBags
    s.write_u32(0); // focused
    s.write_u32(0); // selected
    let sack = s.begin_block(0);
    s.write_byte(0); // tempBool
    s.write_u32(1); // item count
    s.write_item_v11(
        "records/items/gearhead/a07_head001.dbr",
        1,
        0xBEE5,
        [7, 9, 7, 9], // the known-wrong x/y if the skip is missed
    );
    s.write_u32(2); // true x — an int in bags
    s.write_u32(3); // true y
    s.end_block(sack);
    s.write_byte(0); // useAlternate
    for slot in 0..12u32 {
        // Slot 0 dressed, the rest empty — an empty slot is an item whose
        // baseName is the empty string, exactly as the game stores it.
        if slot == 0 {
            s.write_item_v11(
                "records/items/gearhead/a07_head001.dbr",
                1,
                0xCAFE,
                [0, 1, 0, 0],
            );
        } else {
            s.write_item_v11("", 0, 0, [0, 0, 0, 0]);
        }
        s.write_byte(u8::from(slot == 0)); // attached
    }
    s.write_byte(0); // alternate1
    for _ in 0..2 {
        s.write_item_v11("", 0, 0, [0, 0, 0, 0]);
        s.write_byte(0);
    }
    s.write_byte(0); // alternate2
    for _ in 0..2 {
        s.write_item_v11("", 0, 0, [0, 0, 0, 0]);
        s.write_byte(0);
    }
    s.end_block(b3);

    // block 4: personal stash v11 — one tab with the 20-byte trailer, the
    // second consumer of the shared trailer logic
    let b4 = s.begin_block(4);
    s.write_u32(11);
    s.write_u32(1); // tab count
    let tab = s.begin_block(0);
    s.write_u32(8); // width
    s.write_u32(16); // height
    s.write_u32(1); // item count
    s.write_item_v11(
        "records/items/materia/compa_ectoplasm.dbr",
        3,
        0xF00D,
        [0, 1, 0, 0],
    );
    s.write_f32(4.0); // x
    s.write_f32(2.0); // y
    for _ in 0..5 {
        s.write_u32(0); // trailer
    }
    s.end_block(tab);
    s.end_block(b4);

    s.into_bytes()
}

#[cfg(test)]
mod forge {
    use super::*;

    /// Regenerate the committed synthetic fixtures. Run explicitly:
    /// `cargo test -- --ignored forge_fixtures`. The two real-capture
    /// fixtures (`cipher-header.bin`, `item-v11-real.bin` + expectation) are
    /// only regenerated when `SMUGGLERS_BENCH_SAVE_ROOT` points at a real
    /// save root — see `fixtures/README.md`.
    #[test]
    #[ignore = "fixture forge — writes fixtures/, run deliberately"]
    fn forge_fixtures() {
        let dir = super::dir();
        std::fs::create_dir_all(&dir).unwrap();

        // item-v11-trap.bin: seed, one v11 item (trap ints 7,9,7,9), true x/y
        let mut s = Scribe::new(0x7A9B_11ED);
        s.write_item_v11(
            "records/items/materia/compa_ectoplasm.dbr",
            12,
            0xACE,
            [7, 9, 7, 9],
        );
        s.write_f32(3.0);
        s.write_f32(5.0);
        std::fs::write(dir.join("item-v11-trap.bin"), s.into_bytes()).unwrap();

        // stash-tab-v11.bin: one bare tab block, 10x19, two items, trailer
        let mut s = Scribe::new(0x57A5_401D);
        let tab = s.begin_block(0);
        s.write_u32(10);
        s.write_u32(19);
        s.write_u32(2);
        for i in 0..2u32 {
            s.write_item_v11(
                "records/items/materia/compa_ectoplasm.dbr",
                4 + i,
                0xB0B + i,
                [0, 1, 0, 0],
            );
            s.write_f32(i as f32);
            s.write_f32(7.0);
        }
        for _ in 0..5 {
            s.write_u32(0);
        }
        s.end_block(tab);
        std::fs::write(dir.join("stash-tab-v11.bin"), s.into_bytes()).unwrap();

        std::fs::write(dir.join("transfer-v11.gst"), forge_transfer_v11()).unwrap();
        std::fs::write(
            dir.join("player-v8-inv11.gdc"),
            forge_player_v8_inv11("Fixture", 42),
        )
        .unwrap();

        // player-corrupt.gdc: a file the cipher cannot turn — flagged, never fatal
        std::fs::write(dir.join("player-corrupt.gdc"), [0x13u8; 64]).unwrap();

        // Real captures, bench only
        if let Ok(root) = std::env::var("SMUGGLERS_BENCH_SAVE_ROOT") {
            let real = std::fs::read(std::path::Path::new(&root).join("transfer.gst"))
                .expect("bench save root should carry transfer.gst");
            std::fs::write(dir.join("cipher-header.bin"), &real[..64]).unwrap();

            let stash = crate::warehouse::parse_stash(&real).expect("real stash should parse");
            let capture = stash.tabs[0].items[0].clone();
            let mut s = Scribe::new(0xFE11_0FF5);
            s.write_str(&capture.item.base_name);
            s.write_str(&capture.item.prefix_name);
            s.write_str(&capture.item.suffix_name);
            s.write_str(&capture.item.modifier_name);
            s.write_str(&capture.item.transmute_name);
            s.write_u32(capture.item.seed);
            s.write_str(&capture.item.component_name);
            s.write_str(&capture.item.relic_bonus);
            s.write_u32(capture.item.component_seed);
            s.write_str(&capture.item.augment_name);
            s.write_u32(capture.item.unknown);
            s.write_u32(capture.item.augment_seed);
            s.write_u32(capture.item.var1);
            s.write_u32(capture.item.stack_count);
            for v in [0u32, 1, 0, 0] {
                s.write_u32(v); // fresh v11 trailing ints — values are discarded by the parser
            }
            s.write_f32(capture.x as f32);
            s.write_f32(capture.y as f32);
            std::fs::write(dir.join("item-v11-real.bin"), s.into_bytes()).unwrap();
            std::fs::write(
                dir.join("item-v11-real.expected.json"),
                serde_json::to_vec_pretty(&capture).unwrap(),
            )
            .unwrap();
        }
    }
}
