//! The icons — the `bitmap` a record names, extracted from the user's own
//! archives and decoded from Grim Dawn's `TEX`-wrapped DDS to an RGBA PNG the
//! webview can show. Two cabinets: item icons come out of `Items.arc`, the
//! skill panel's icons out of `UI.arc` (see [`Cabinet`]).
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
    if let Some(png) = decode_flat_surface(&blob) {
        return Ok(png);
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

/// The skill-panel icons are not block-compressed. GD writes them as a plain
/// surface with an EMPTY pixel format — no fourcc, no channel masks — which
/// `image_dds` refuses, rightly: there is no format there to read. The header
/// still states the surface size and the bit depth, and the payload behind it
/// is exactly `width × height × depth/8` bytes in D3D's channel order. Decode
/// that directly, or hand the blob back to the BCn path untouched.
///
/// Both depths are in the wild and BOTH must be read: Cadence's icon is 32-bit
/// BGRA, Blitz's — one row down the same panel — is 24-bit BGR with no alpha
/// channel at all. Reading only the first leaves a lettered blank in the tree.
fn decode_flat_surface(blob: &[u8]) -> Option<Vec<u8>> {
    /// 4 bytes of magic + the 124-byte header.
    const HEADER_LEN: usize = 128;
    /// Where the pixel-format block sits inside that header.
    const PIXEL_FORMAT: usize = 76;
    let word = |at: usize| -> Option<u32> {
        blob.get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if word(PIXEL_FORMAT + 8)? != 0 {
        return None; // a fourcc means a real, compressed format — not ours
    }
    let depth = match word(PIXEL_FORMAT + 12)? {
        32 => 4usize,
        24 => 3usize,
        _ => return None,
    };
    let height = word(12)? as usize;
    let width = word(16)? as usize;
    let pixels = blob.get(HEADER_LEN..HEADER_LEN + width * height * depth)?;
    // BGR(A) on disk (D3D's A8R8G8B8 / R8G8B8 order), RGBA in a PNG. A surface
    // with no alpha channel is opaque, which is what a 24-bit icon means.
    let mut rgba = Vec::with_capacity(width * height * 4);
    for pixel in pixels.chunks_exact(depth) {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        rgba.push(if depth == 4 { pixel[3] } else { 0xFF });
    }
    let image: image::RgbaImage = image::ImageBuffer::from_raw(width as u32, height as u32, rgba)?;
    let mut png: Vec<u8> = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(png)
}

// ---------------------------------------------------------------------------
// The icon cabinets — managed state: lazily-opened archive shelves + a decoded
// PNG cache, one set per cabinet. Most bitmaps live in the base archive;
// expansions open only if a bitmap isn't found in the shelves already loaded.
// ---------------------------------------------------------------------------

use std::path::Path;
use std::sync::Mutex;

/// Which archive family a bitmap comes out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cabinet {
    /// `Items.arc` — the item icons the docket shows.
    Items,
    /// `UI.arc` — the skill panel's own icons, for THE TRADES.
    Ui,
}

const ITEMS_ARC_SHELVES: [&str; 4] = [
    "resources/Items.arc",
    "gdx1/resources/Items.arc",
    "gdx2/resources/Items.arc",
    "gdx3/resources/Items.arc",
];

const UI_ARC_SHELVES: [&str; 4] = [
    "resources/UI.arc",
    "gdx1/resources/UI.arc",
    "gdx2/resources/UI.arc",
    "gdx3/resources/UI.arc",
];

impl Cabinet {
    fn shelves(self) -> &'static [&'static str] {
        match self {
            Cabinet::Items => &ITEMS_ARC_SHELVES,
            Cabinet::Ui => &UI_ARC_SHELVES,
        }
    }

    /// The archive's own key for a bitmap the records name. A skill record
    /// spells its icon `ui/skills/icons/class01/skillicon_cadence1_up.tex`;
    /// UI.arc files the very same texture WITHOUT that leading `ui/`. Getting
    /// this wrong reads as "the install has no such icon", which is a lie.
    fn key(self, bitmap: &str) -> String {
        match self {
            Cabinet::Items => bitmap.to_string(),
            Cabinet::Ui => bitmap.strip_prefix("ui/").unwrap_or(bitmap).to_string(),
        }
    }
}

#[derive(Default)]
pub struct IconState {
    inner: Mutex<std::collections::HashMap<Cabinet, IconInner>>,
}

#[derive(Default)]
struct IconInner {
    /// Shelves opened so far (index into the cabinet's list for the next open).
    shelves: Vec<IconShelf>,
    opened: usize,
    /// bitmap → decoded PNG (`None` = searched every shelf, genuinely absent).
    cache: std::collections::HashMap<String, Option<Vec<u8>>>,
}

impl IconState {
    /// The decoded PNG for one bitmap, or `None` if no shelf in that cabinet
    /// holds it. Opens more shelves on demand; caches every answer (hit and
    /// miss) so a missing icon is asked for exactly once.
    pub fn icon_png(&self, install_root: &Path, cabinet: Cabinet, bitmap: &str) -> Option<Vec<u8>> {
        let mut cabinets = self.inner.lock().ok()?;
        let inner = cabinets.entry(cabinet).or_default();
        let key = cabinet.key(bitmap);
        if let Some(cached) = inner.cache.get(&key) {
            return cached.clone();
        }
        // Search already-open shelves, then open more until found or exhausted.
        loop {
            for shelf in &inner.shelves {
                if let Ok(Some(png)) = shelf.icon_png(&key) {
                    inner.cache.insert(key, Some(png.clone()));
                    return Some(png);
                }
            }
            if inner.opened >= cabinet.shelves().len() {
                inner.cache.insert(key, None);
                return None;
            }
            let next = install_root.join(cabinet.shelves()[inner.opened]);
            inner.opened += 1;
            if let Ok(data) = std::fs::read(&next) {
                if let Ok(shelf) = IconShelf::open(data) {
                    inner.shelves.push(shelf);
                }
            }
        }
    }
}
