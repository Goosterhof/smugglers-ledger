//! The cipher — the XOR stream every save file hides behind.
//!
//! Ported verbatim from the lab's proven Python spike (`gd_spike.py`,
//! ratified 2026-08-15 and re-ratified 2026-08-16 against two distinct real
//! datasets), NOT re-derived from the stale public references. The canonical
//! format record is `.claude/memory/grim-dawn-save-format.md` in the parent
//! laboratory.
//!
//! Mechanism module: plain name by design — "cipher" is the plan's own verb.
//! The Ledger *turns the cipher* on a save; this is the thing that turns.

use crate::error::LedgerError;

const KEY_SALT: u32 = 0x5555_5555;
const KEY_MULTIPLIER: u32 = 39_916_801;
/// A decrypted length prefix larger than this is not a string — it is the
/// structure telling us we desynced (the spike's own tell was 1073741824,
/// which is the float 2.0 misread as a length).
const MAX_PLAUSIBLE_STRING: u32 = 4096;

/// A decrypting cursor over one save file's bytes.
///
/// The rolling key evolves from the RAW (encrypted) bytes only, which is what
/// makes [`Cipher::consume_to`] safe: any span can be consumed byte-by-byte
/// without interpreting it, as long as it hides no nested block or
/// [`Cipher::next_u32`] field (those advance without a key update).
#[derive(Debug)]
pub struct Cipher<'a> {
    /// The raw (encrypted) bytes of one save file.
    data: &'a [u8],
    pos: usize,
    key: u32,
    table: [u32; 256],
}

impl<'a> Cipher<'a> {
    /// Read the 4-byte seed and build the 256-entry rolling key table.
    pub fn turn(data: &'a [u8]) -> Result<Self, LedgerError> {
        if data.len() < 4 {
            return Err(LedgerError::CipherWontTurn {
                detail: "file too short to carry a cipher seed".into(),
            });
        }
        let key = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) ^ KEY_SALT;
        let mut table = [0u32; 256];
        let mut k = key;
        for slot in &mut table {
            k = k.rotate_right(1).wrapping_mul(KEY_MULTIPLIER);
            *slot = k;
        }
        Ok(Self {
            data,
            pos: 4,
            key,
            table,
        })
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn raw4(&mut self) -> Result<[u8; 4], LedgerError> {
        if self.pos + 4 > self.data.len() {
            return Err(self.desync("ran off the end of the file mid-word"));
        }
        let raw = [
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ];
        self.pos += 4;
        Ok(raw)
    }

    /// Decrypt a u32 WITHOUT updating the key — block-length fields and
    /// end-of-block markers only.
    pub fn next_u32(&mut self) -> Result<u32, LedgerError> {
        let raw = self.raw4()?;
        Ok(u32::from_le_bytes(raw) ^ self.key)
    }

    /// Decrypt a u32 and roll the key over its raw bytes.
    pub fn read_u32(&mut self) -> Result<u32, LedgerError> {
        let raw = self.raw4()?;
        let value = u32::from_le_bytes(raw) ^ self.key;
        for b in raw {
            self.key ^= self.table[b as usize];
        }
        Ok(value)
    }

    /// Decrypt one byte and roll the key over it.
    pub fn read_byte(&mut self) -> Result<u8, LedgerError> {
        if self.pos >= self.data.len() {
            return Err(self.desync("ran off the end of the file mid-byte"));
        }
        let raw = self.data[self.pos];
        self.pos += 1;
        let value = raw ^ (self.key as u8);
        self.key ^= self.table[raw as usize];
        Ok(value)
    }

    pub fn read_f32(&mut self) -> Result<f32, LedgerError> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Length-prefixed ASCII string.
    pub fn read_str(&mut self) -> Result<String, LedgerError> {
        let len = self.read_u32()?;
        if len > MAX_PLAUSIBLE_STRING {
            return Err(self.desync(&format!("string length {len} is not a string")));
        }
        let mut out = String::with_capacity(len as usize);
        for _ in 0..len {
            out.push(self.read_byte()? as char);
        }
        Ok(out)
    }

    /// Length-prefixed UTF-16LE string (the character name).
    pub fn read_wstr(&mut self) -> Result<String, LedgerError> {
        let len = self.read_u32()?;
        if len > MAX_PLAUSIBLE_STRING {
            return Err(self.desync(&format!("wstring length {len} is not a string")));
        }
        let mut units = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let lo = self.read_byte()? as u16;
            let hi = self.read_byte()? as u16;
            units.push(lo | (hi << 8));
        }
        Ok(String::from_utf16_lossy(&units))
    }

    /// Block header: decrypted block id + the absolute end offset of its body.
    pub fn block_start(&mut self) -> Result<(u32, usize), LedgerError> {
        let id = self.read_u32()?;
        let len = self.next_u32()? as usize;
        let end = self.pos + len;
        if end > self.data.len() {
            return Err(self.desync(&format!("block {id} claims to run past the file's end")));
        }
        Ok((id, end))
    }

    /// Verify we consumed the block exactly, then eat the 0 terminator.
    /// The position check is what catches every layout mistake.
    pub fn block_end(&mut self, end: usize) -> Result<(), LedgerError> {
        if self.pos != end {
            return Err(self.desync(&format!(
                "block end mismatch: at {} but the block ends at {end}",
                self.pos
            )));
        }
        let terminator = self.next_u32()?;
        if terminator != 0 {
            return Err(self.desync(&format!("expected end-of-block 0, got {terminator}")));
        }
        Ok(())
    }

    /// Raw-consume to a block boundary, keeping the key state correct.
    /// Only safe when the span contains no nested block or `next_u32` field —
    /// used for the v11 stash-tab trailer and the character-info loot-filter
    /// tail, and only those.
    pub fn consume_to(&mut self, end: usize) -> Result<(), LedgerError> {
        if end > self.data.len() {
            return Err(self.desync("consume_to target past the file's end"));
        }
        while self.pos < end {
            self.read_byte()?;
        }
        Ok(())
    }

    pub fn expect_u32(&mut self, want: u32, what: &str) -> Result<u32, LedgerError> {
        let got = self.read_u32()?;
        if got != want {
            return Err(self.desync(&format!("{what}: expected {want}, got {got}")));
        }
        Ok(got)
    }

    fn desync(&self, detail: &str) -> LedgerError {
        LedgerError::CipherWontTurn {
            detail: format!("{detail} (at byte {})", self.pos),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn it_should_reproduce_the_spikes_byte_for_byte_output_on_a_real_header_capture() {
        // fixtures/cipher-header.bin is the first 64 raw bytes of a real
        // transfer.gst — seed, magic, block header, version, pad, mod string,
        // expansion byte, tab count — captured 2026-08-16 and anonymised by
        // truncation (no item, name, or seed-derived data follows).
        // The expected values are the spike's own decoded output for the same
        // capture: byte-for-byte agreement with the proven Python implementation.
        // The capture is TRUNCATED at 64 bytes, so the block's length word
        // points past the fixture's end — the header is decoded with the raw
        // primitives (`read_u32` + `next_u32`), the same cipher operations
        // `block_start` composes, without its whole-file bounds check.
        let bytes = fixtures::bytes("cipher-header.bin");
        let mut cipher = Cipher::turn(&bytes).expect("the cipher should turn");
        cipher
            .expect_u32(2, "stash magic")
            .expect("magic should be 2");
        let block = cipher.read_u32().expect("block id");
        assert_eq!(block, 18, "transfer.gst carries block 18");
        let _length = cipher.next_u32().expect("block length word");
        let version = cipher.read_u32().expect("version");
        assert_eq!(version, 11, "the 2026-08-16 capture is a v11 stash");
        cipher.next_u32().expect("pad");
        let mod_name = cipher.read_str().expect("mod string");
        assert_eq!(mod_name, "", "vanilla save: empty mod string");
        let expansion = cipher.read_byte().expect("expansion byte");
        assert_eq!(
            expansion, 7,
            "1.2-era expansion byte is 7, not the 3 the stale refs assert"
        );
        let tabs = cipher.read_u32().expect("tab count");
        assert_eq!(tabs, 10, "the ratification-run stash had 10 tabs");
    }

    #[test]
    fn it_should_refuse_a_file_too_short_to_seed_the_key() {
        let err = Cipher::turn(&[0x01, 0x02]).unwrap_err();
        assert!(matches!(err, LedgerError::CipherWontTurn { .. }));
    }

    #[test]
    fn it_should_flag_an_implausible_string_length_as_a_desync_not_an_allocation() {
        // A stream whose first decrypted u32 after the seed is huge:
        // encrypt a length of 1_073_741_824 (the spike's own desync tell).
        let mut scribe = fixtures::Scribe::new(0xDEAD_BEEF);
        scribe.write_u32(1_073_741_824);
        let bytes = scribe.into_bytes();
        let mut cipher = Cipher::turn(&bytes).unwrap();
        let err = cipher.read_str().unwrap_err();
        assert!(matches!(err, LedgerError::CipherWontTurn { .. }));
    }
}
