//! Item icons — the `bitmap` a base record names, extracted from the user's
//! own `resources/Items.arc` and decoded from Grim Dawn's `TEX`-wrapped DDS to
//! an RGBA PNG the webview can show.
//!
//! Same RD-3 legal floor as the codex: read from the licensed install at
//! runtime, decoded in memory, cached to the app's own data dir — never
//! bundled, never shipped. Every file-open passes the codex's `shelf_guard`.

use std::collections::HashMap;

use crate::error::LedgerError;

const ARC_MAGIC: u32 = 0x0043_5241; // "ARC\0"
const ARC_RECORD_HEADER_SIZE: usize = 44;
const ARC_PART_HEADER_SIZE: usize = 12;
/// GD's texture wrapper: a 12-byte `TEX\x02` header in front of a DDS blob.
const TEX_HEADER_LEN: usize = 12;

fn le_u32(data: &[u8], pos: usize) -> Result<u32, LedgerError> {
    data.get(pos..pos + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| LedgerError::CodexShelfMissing {
            detail: format!("Items.arc truncated at offset {pos}"),
        })
}

/// One opened `Items.arc`: a filename → entry index, decompressed on demand.
/// The archive is large (~180 MB of textures); we index once and pull only the
/// handful of bitmaps the hoard actually references.
pub struct IconShelf {
    data: Vec<u8>,
    record_table_offset: usize,
    /// filename → (data offset, compressed, decompressed, parts, first_part)
    entries: HashMap<String, (usize, usize, usize, usize, usize)>,
}

impl IconShelf {
    pub fn open(data: Vec<u8>) -> Result<Self, LedgerError> {
        let magic = le_u32(&data, 0)?;
        let version = le_u32(&data, 4)?;
        if magic != ARC_MAGIC || version != 3 {
            return Err(LedgerError::CodexShelfMissing {
                detail: format!("not an ARC v3 archive (magic {magic:#x}, version {version})"),
            });
        }
        let file_entries = le_u32(&data, 8)? as usize;
        let record_table_size = le_u32(&data, 16)? as usize;
        let string_table_size = le_u32(&data, 20)? as usize;
        let record_table_offset = le_u32(&data, 24)? as usize;
        let string_table_at = record_table_offset + record_table_size;
        let headers_at = record_table_offset + record_table_size + string_table_size;

        let mut entries = HashMap::with_capacity(file_entries);
        for i in 0..file_entries {
            let at = headers_at + i * ARC_RECORD_HEADER_SIZE;
            let offset = le_u32(&data, at + 4)? as usize;
            let compressed = le_u32(&data, at + 8)? as usize;
            let decompressed = le_u32(&data, at + 12)? as usize;
            let parts = le_u32(&data, at + 28)? as usize;
            let first_part = le_u32(&data, at + 32)? as usize;
            let name_len = le_u32(&data, at + 36)? as usize;
            let name_off = le_u32(&data, at + 40)? as usize;
            let name_bytes = data
                .get(string_table_at + name_off..string_table_at + name_off + name_len)
                .unwrap_or_default();
            let name = String::from_utf8_lossy(name_bytes)
                .to_ascii_lowercase()
                .replace('\\', "/");
            entries.insert(name, (offset, compressed, decompressed, parts, first_part));
        }
        Ok(Self {
            data,
            record_table_offset,
            entries,
        })
    }

    /// The raw bytes of one archived file (LZ4 parts reassembled), or `None`
    /// when the archive holds no such name.
    fn raw_file(&self, name: &str) -> Result<Option<Vec<u8>>, LedgerError> {
        let key = name.to_ascii_lowercase().replace('\\', "/");
        let Some(&(offset, compressed, decompressed, parts, first_part)) = self.entries.get(&key)
        else {
            return Ok(None);
        };
        if parts <= 1 && compressed == decompressed {
            return Ok(Some(
                self.data
                    .get(offset..offset + compressed)
                    .unwrap_or_default()
                    .to_vec(),
            ));
        }
        let mut out = Vec::with_capacity(decompressed);
        for p in 0..parts {
            let part_at = self.record_table_offset + (first_part + p) * ARC_PART_HEADER_SIZE;
            let p_off = le_u32(&self.data, part_at)? as usize;
            let p_comp = le_u32(&self.data, part_at + 4)? as usize;
            let p_dec = le_u32(&self.data, part_at + 8)? as usize;
            let chunk = self.data.get(p_off..p_off + p_comp).unwrap_or_default();
            if p_comp == p_dec {
                out.extend_from_slice(chunk);
            } else {
                out.extend_from_slice(&lz4_flex::block::decompress(chunk, p_dec).map_err(|e| {
                    LedgerError::CodexShelfMissing {
                        detail: format!("Items.arc part failed to decompress: {e}"),
                    }
                })?);
            }
        }
        Ok(Some(out))
    }

    /// Extract the named bitmap and decode it to a PNG. The `.tex` is a 12-byte
    /// header in front of a DDS; we strip the header, parse the DDS, decode
    /// (any BCn) to RGBA, and re-encode as PNG for the webview.
    pub fn icon_png(&self, bitmap: &str) -> Result<Option<Vec<u8>>, LedgerError> {
        let Some(raw) = self.raw_file(bitmap)? else {
            return Ok(None);
        };
        if raw.len() <= TEX_HEADER_LEN || &raw[..3] != b"TEX" {
            return Ok(None);
        }
        decode_tex_to_png(&raw[TEX_HEADER_LEN..]).map(Some)
    }
}

/// A DDS blob → RGBA → PNG bytes. Separated so the bench can feed it a fixture.
fn decode_tex_to_png(dds_bytes: &[u8]) -> Result<Vec<u8>, LedgerError> {
    // GD's TEX-wrapped DDS carries a nonstandard magic — bytes "DDS\x52" where
    // a standard DDS is "DDS\x20" (a space). The rest of the header is
    // conventional, so normalise the magic and parse it as the DDS it is.
    let mut blob = dds_bytes.to_vec();
    if blob.len() >= 4 && &blob[..3] == b"DDS" {
        blob[3] = b' ';
    }
    let dds = image_dds::ddsfile::Dds::read(blob.as_slice()).map_err(|e| {
        LedgerError::CodexShelfMissing {
            detail: format!("icon DDS parse failed: {e}"),
        }
    })?;
    let rgba = image_dds::image_from_dds(&dds, 0).map_err(|e| LedgerError::CodexShelfMissing {
        detail: format!("icon DDS decode failed: {e}"),
    })?;
    let mut png: Vec<u8> = Vec::new();
    rgba.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| LedgerError::CodexShelfMissing {
            detail: format!("icon PNG encode failed: {e}"),
        })?;
    Ok(png)
}

// ---------------------------------------------------------------------------
// The icon cabinet — managed state: lazily-opened Items.arc shelves + a decoded
// PNG cache. Most bitmaps live in the base Items.arc; expansions open only if a
// bitmap isn't found in the shelves already loaded.
// ---------------------------------------------------------------------------

use std::path::Path;
use std::sync::Mutex;

/// The Items.arc shelves inside a Grim Dawn install, base first.
const ITEMS_ARC_SHELVES: [&str; 4] = [
    "resources/Items.arc",
    "gdx1/resources/Items.arc",
    "gdx2/resources/Items.arc",
    "gdx3/resources/Items.arc",
];

#[derive(Default)]
pub struct IconState {
    inner: Mutex<IconInner>,
}

#[derive(Default)]
struct IconInner {
    /// Shelves opened so far (index into ITEMS_ARC_SHELVES for the next open).
    shelves: Vec<IconShelf>,
    opened: usize,
    /// bitmap → decoded PNG (`None` = searched every shelf, genuinely absent).
    cache: std::collections::HashMap<String, Option<Vec<u8>>>,
}

impl IconState {
    /// The decoded PNG for one bitmap, or `None` if no shelf holds it. Opens
    /// more Items.arc shelves on demand; caches every answer (hit and miss).
    pub fn icon_png(&self, install_root: &Path, bitmap: &str) -> Option<Vec<u8>> {
        let mut inner = self.inner.lock().ok()?;
        if let Some(cached) = inner.cache.get(bitmap) {
            return cached.clone();
        }
        // Search already-open shelves, then open more until found or exhausted.
        loop {
            for shelf in &inner.shelves {
                if let Ok(Some(png)) = shelf.icon_png(bitmap) {
                    inner.cache.insert(bitmap.to_string(), Some(png.clone()));
                    return Some(png);
                }
            }
            if inner.opened >= ITEMS_ARC_SHELVES.len() {
                inner.cache.insert(bitmap.to_string(), None);
                return None;
            }
            let next = install_root.join(ITEMS_ARC_SHELVES[inner.opened]);
            inner.opened += 1;
            if let Ok(data) = std::fs::read(&next) {
                if let Ok(shelf) = IconShelf::open(data) {
                    inner.shelves.push(shelf);
                }
            }
        }
    }
}
