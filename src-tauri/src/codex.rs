//! The codex — names from the user's own install, resolved once, cached per
//! machine (RD-3 made concrete).
//!
//! A parsed item is a record path, never a display string. This module reads
//! `database.arz` and the three expansion databases (`gdx1/GDX1.arz`,
//! `gdx2/GDX2.arz`, `gdx3/GDX3.arz`) from the user's own game install,
//! resolves each record path to its localization tag, and looks that tag up
//! in the game's own text tables — `resources/Text_EN.arc` and each
//! expansion's own copy, **ARC archives on disk, not tables inside the arz**.
//!
//! **The legal floor (RD-3):** everything this module reads is the
//! developer's copyrighted game data. It is read from the user's own licensed
//! install at runtime and cached to the app's local data dir for that user —
//! never bundled, never shipped, never synced. The RD-3 legal-floor audit
//! script (`scripts/legal-floor-audit.sh`) gates this in the Sentinel.
//!
//! **The countervailing RD-2 rule:** codex.rs is the one module allowed to
//! write (its resolve cache, to `app_data_dir`) and is therefore excluded
//! from the read-only sweep — in exchange, every file it opens passes the
//! [`shelf_guard`] choke point, which refuses any path outside the install
//! root or the cache dir. It can never open a path under a discovered save
//! root; its own test asserts the refusal.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::LedgerError;

/// The relative database + text-table shelves inside a Grim Dawn install.
const DATABASE_SHELVES: [&str; 4] = [
    "database/database.arz",
    "gdx1/database/GDX1.arz",
    "gdx2/database/GDX2.arz",
    "gdx3/database/GDX3.arz",
];
const TEXT_SHELVES: [&str; 4] = [
    "resources/Text_EN.arc",
    "gdx1/resources/Text_EN.arc",
    "gdx2/resources/Text_EN.arc",
    "gdx3/resources/Text_EN.arc",
];
const CACHE_FILE: &str = "codex-cache.json";
/// Identity sampling width: leading + trailing bytes of each database file.
const IDENTITY_SAMPLE: usize = 64 * 1024;

/// One resolved record: what the UI needs and nothing else.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRecord {
    /// Display name, localized. `None` = contraband still in the crate — the
    /// UI falls back to the raw record path, which stays searchable (4C).
    pub name: Option<String>,
    /// The game's own classification string (`Common`, `Epic`, …), verbatim.
    pub classification: Option<String>,
    /// Ordinal ink tier 0–5 derived from the classification (Design System
    /// #00012 Open Question #1 — enumerated on the bench, unknown → 0).
    pub tier: u8,
    /// The record's `Class` field — the equip-slot class the 4A slot filter
    /// reads (`ArmorProtective_Head`, `WeaponMelee_Axe`, …).
    pub slot_class: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CodexCache {
    /// Identity of the install's database files — a changed game patch
    /// changes this and invalidates every entry below.
    db_hash: String,
    entries: HashMap<String, ResolvedRecord>,
}

/// The choke point every codex file-open passes through: the path must live
/// under the install root or the cache dir. This is the mechanism behind the
/// RD-2 exclusion — codex.rs cannot open a save path even by accident.
fn shelf_guard(path: &Path, install_root: &Path, cache_dir: &Path) -> Result<(), LedgerError> {
    if path.starts_with(install_root) || path.starts_with(cache_dir) {
        return Ok(());
    }
    Err(LedgerError::CodexShelfMissing {
        detail: format!(
            "refused to open {} — the codex only reads its own shelves",
            path.display()
        ),
    })
}

fn guarded_read(
    path: &Path,
    install_root: &Path,
    cache_dir: &Path,
) -> Result<Vec<u8>, LedgerError> {
    shelf_guard(path, install_root, cache_dir)?;
    std::fs::read(path).map_err(|e| LedgerError::unreadable(path, &e))
}

/// The install's database identity: SHA-256 over each shelf's length, mtime,
/// and head+tail sample. Change-sensitive (a patch rewrites the files) while
/// staying far cheaper than hashing ~180 MB on every warm start — the warm
/// path must fit inside the 2-second window Phase 4's criterion names.
pub fn database_identity(install_root: &Path) -> Result<String, LedgerError> {
    let mut hasher = Sha256::new();
    for shelf in DATABASE_SHELVES {
        let path = install_root.join(shelf);
        let mut file =
            std::fs::File::open(&path).map_err(|e| LedgerError::unreadable(&path, &e))?;
        let meta = file
            .metadata()
            .map_err(|e| LedgerError::unreadable(&path, &e))?;
        hasher.update(meta.len().to_le_bytes());
        if let Ok(modified) = meta.modified() {
            if let Ok(since) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                hasher.update(since.as_secs().to_le_bytes());
            }
        }
        let mut head = vec![0u8; IDENTITY_SAMPLE.min(meta.len() as usize)];
        file.read_exact(&mut head)
            .map_err(|e| LedgerError::unreadable(&path, &e))?;
        hasher.update(&head);
        if meta.len() as usize > IDENTITY_SAMPLE {
            let tail_start = meta.len() - IDENTITY_SAMPLE as u64;
            file.seek(SeekFrom::Start(tail_start))
                .map_err(|e| LedgerError::unreadable(&path, &e))?;
            let mut tail = vec![0u8; IDENTITY_SAMPLE];
            file.read_exact(&mut tail)
                .map_err(|e| LedgerError::unreadable(&path, &e))?;
            hasher.update(&tail);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// ARC — the localization shelves (tag → display string)
// ---------------------------------------------------------------------------

const ARC_MAGIC: u32 = 0x0043_5241; // "ARC\0"
const ARC_RECORD_HEADER_SIZE: usize = 44;
const ARC_PART_HEADER_SIZE: usize = 12;

fn le_u32(data: &[u8], pos: usize) -> Result<u32, LedgerError> {
    data.get(pos..pos + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| LedgerError::CodexShelfMissing {
            detail: format!("ARC/ARZ truncated at offset {pos}"),
        })
}

fn slice(data: &[u8], pos: usize, len: usize) -> Result<&[u8], LedgerError> {
    data.get(pos..pos + len)
        .ok_or_else(|| LedgerError::CodexShelfMissing {
            detail: format!("ARC/ARZ truncated at offset {pos} (wanted {len} bytes)"),
        })
}

/// Parse one `Text_EN.arc` into tag → string pairs. Per gd-edit's `arc.clj`:
/// header, record table at `record_table_offset`, string table after it,
/// record headers after both; each record is either stored plain or as
/// LZ4-block-compressed parts.
fn parse_text_arc(data: &[u8], table: &mut HashMap<String, String>) -> Result<(), LedgerError> {
    let magic = le_u32(data, 0)?;
    let version = le_u32(data, 4)?;
    if magic != ARC_MAGIC || version != 3 {
        return Err(LedgerError::CodexShelfMissing {
            detail: format!("not an ARC v3 archive (magic {magic:#x}, version {version})"),
        });
    }
    let file_entries = le_u32(data, 8)? as usize;
    let record_table_size = le_u32(data, 16)? as usize;
    let string_table_size = le_u32(data, 20)? as usize;
    let record_table_offset = le_u32(data, 24)? as usize;

    let headers_at = record_table_offset + record_table_size + string_table_size;
    for i in 0..file_entries {
        let at = headers_at + i * ARC_RECORD_HEADER_SIZE;
        let entry_type = le_u32(data, at)?;
        let offset = le_u32(data, at + 4)? as usize;
        let compressed = le_u32(data, at + 8)? as usize;
        let decompressed = le_u32(data, at + 12)? as usize;
        // at+16 decompressed adler32, at+20 filetime (i64)
        let parts = le_u32(data, at + 28)? as usize;
        let first_part = le_u32(data, at + 32)? as usize;

        let contents: Vec<u8> = if entry_type == 1 && compressed == decompressed {
            slice(data, offset, compressed)?.to_vec()
        } else {
            let mut out = Vec::with_capacity(decompressed);
            for p in 0..parts {
                let part_at = record_table_offset + (first_part + p) * ARC_PART_HEADER_SIZE;
                let p_offset = le_u32(data, part_at)? as usize;
                let p_comp = le_u32(data, part_at + 4)? as usize;
                let p_decomp = le_u32(data, part_at + 8)? as usize;
                let chunk = slice(data, p_offset, p_comp)?;
                if p_comp == p_decomp {
                    out.extend_from_slice(chunk);
                } else {
                    let inflated = lz4_flex::block::decompress(chunk, p_decomp).map_err(|e| {
                        LedgerError::CodexShelfMissing {
                            detail: format!("ARC part failed to decompress: {e}"),
                        }
                    })?;
                    out.extend_from_slice(&inflated);
                }
            }
            out
        };
        for line in String::from_utf8_lossy(&contents).lines() {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].trim();
                let value = line[eq + 1..].trim();
                if !key.is_empty() {
                    table.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ARZ — the record shelves (record path → fields)
// ---------------------------------------------------------------------------

/// One loaded `.arz` database: string table + record directory, with record
/// bodies decompressed on demand. Per gd-edit's `arz.clj`.
struct ArzShelf {
    data: Vec<u8>,
    strings: Vec<String>,
    /// record path → (body offset, compressed size, decompressed size)
    directory: HashMap<String, (usize, usize, usize)>,
}

impl ArzShelf {
    fn open(data: Vec<u8>) -> Result<Self, LedgerError> {
        let version = le_u32(&data, 2)? & 0xFFFF;
        if version != 3 {
            return Err(LedgerError::CodexShelfMissing {
                detail: format!("unknown arz version {version}"),
            });
        }
        let record_table_start = le_u32(&data, 4)? as usize;
        let record_table_entries = le_u32(&data, 12)? as usize;
        let string_table_start = le_u32(&data, 16)? as usize;

        // String table: u32 count, then count length-prefixed ASCII strings.
        let mut pos = string_table_start;
        let count = le_u32(&data, pos)? as usize;
        pos += 4;
        let mut strings = Vec::with_capacity(count);
        for _ in 0..count {
            let len = le_u32(&data, pos)? as usize;
            pos += 4;
            let s = slice(&data, pos, len)?;
            strings.push(String::from_utf8_lossy(s).into_owned());
            pos += len;
        }

        // Record directory: filename index, length-prefixed type string,
        // offset, compressed, decompressed, then two u32s (timestamps).
        let mut directory = HashMap::with_capacity(record_table_entries);
        let mut pos = record_table_start;
        for _ in 0..record_table_entries {
            let filename_idx = le_u32(&data, pos)? as usize;
            pos += 4;
            let type_len = le_u32(&data, pos)? as usize;
            pos += 4 + type_len;
            let offset = le_u32(&data, pos)? as usize;
            let compressed = le_u32(&data, pos + 4)? as usize;
            let decompressed = le_u32(&data, pos + 8)? as usize;
            pos += 20;
            if let Some(name) = strings.get(filename_idx) {
                directory.insert(name.clone(), (offset, compressed, decompressed));
            }
        }
        Ok(Self {
            data,
            strings,
            directory,
        })
    }

    /// Decompress one record and pull only the fields the Ledger needs.
    fn record_essentials(
        &self,
        record_path: &str,
    ) -> Result<Option<RecordEssentials>, LedgerError> {
        let Some(&(offset, compressed, decompressed)) = self.directory.get(record_path) else {
            return Ok(None);
        };
        // Record bodies sit at offset + 24 (past the file header).
        let raw = slice(&self.data, offset + 24, compressed)?;
        let body = lz4_flex::block::decompress(raw, decompressed).map_err(|e| {
            LedgerError::CodexShelfMissing {
                detail: format!("record {record_path} failed to decompress: {e}"),
            }
        })?;
        let mut essentials = RecordEssentials::default();
        let mut pos = 0usize;
        while pos + 8 <= body.len() {
            let field_type = u16::from_le_bytes([body[pos], body[pos + 1]]);
            let count = u16::from_le_bytes([body[pos + 2], body[pos + 3]]) as usize;
            let name_idx = le_u32(&body, pos + 4)? as usize;
            pos += 8;
            let values_end = pos + count * 4;
            if values_end > body.len() {
                break;
            }
            if field_type == 2 && count >= 1 {
                if let Some(field_name) = self.strings.get(name_idx) {
                    if let Some(target) = essentials.slot_for(field_name) {
                        let value_idx = le_u32(&body, pos)? as usize;
                        if let Some(value) = self.strings.get(value_idx) {
                            *target = Some(value.clone());
                        }
                    }
                }
            }
            pos = values_end;
        }
        Ok(Some(essentials))
    }
}

/// The four string fields resolution needs — everything else in a record is
/// skipped unread.
#[derive(Debug, Default, Clone)]
struct RecordEssentials {
    item_name_tag: Option<String>,
    description: Option<String>,
    classification: Option<String>,
    class: Option<String>,
    loot_randomizer_name: Option<String>,
}

impl RecordEssentials {
    fn slot_for(&mut self, field_name: &str) -> Option<&mut Option<String>> {
        match field_name {
            "itemNameTag" => Some(&mut self.item_name_tag),
            "description" => Some(&mut self.description),
            "itemClassification" => Some(&mut self.classification),
            "Class" => Some(&mut self.class),
            "lootRandomizerName" => Some(&mut self.loot_randomizer_name),
            _ => None,
        }
    }
}

/// Classification string → ordinal ink tier, per the bench enumeration of
/// the investor's real save set (2026-08-16, recorded in the gadget's
/// CLAUDE.md — Design System #00012 Open Question #1). Unknown values render
/// tier-0: a correct, legible page with one ink weight.
pub fn classification_tier(classification: Option<&str>) -> u8 {
    match classification {
        Some("Common") => 0,
        Some("Magical") => 1,
        Some("Rare") => 2,
        Some("Epic") => 3,
        Some("Legendary") => 4,
        Some("Quest") => 5,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// The codex itself
// ---------------------------------------------------------------------------

pub struct Codex {
    install_root: PathBuf,
    cache_dir: PathBuf,
    cache: CodexCache,
    /// Lazily-opened shelves — only when the cache cannot answer.
    shelves: Option<Vec<ArzShelf>>,
    localization: Option<HashMap<String, String>>,
}

impl Codex {
    /// Open the codex for one install. Reads the cache if its identity
    /// matches the install's current database hash; a stale cache is
    /// discarded and re-resolved — the Ledger never serves stale names
    /// against an updated database.
    pub fn open(install_root: &Path, cache_dir: &Path) -> Result<Self, LedgerError> {
        let db_hash = database_identity(install_root)?;
        let cache_path = cache_dir.join(CACHE_FILE);
        let cache = match std::fs::read(&cache_path) {
            Ok(bytes) => match serde_json::from_slice::<CodexCache>(&bytes) {
                Ok(cached) if cached.db_hash == db_hash => cached,
                _ => CodexCache {
                    db_hash,
                    entries: HashMap::new(),
                },
            },
            Err(_) => CodexCache {
                db_hash,
                entries: HashMap::new(),
            },
        };
        Ok(Self {
            install_root: install_root.to_path_buf(),
            cache_dir: cache_dir.to_path_buf(),
            cache,
            shelves: None,
            localization: None,
        })
    }

    /// True when every requested path is already answerable from cache —
    /// the warm start's whole promise (no arz is opened at all).
    pub fn cache_answers<'a, I: IntoIterator<Item = &'a String>>(&self, paths: I) -> bool {
        paths
            .into_iter()
            .all(|p| self.cache.entries.contains_key(p))
    }

    /// Resolve every record path in `paths`, serving from cache where the
    /// hash matched and reading the install's own shelves for the rest.
    /// Unresolvable paths get an entry with `name: None` — surfaced to the
    /// UI as the raw path, never silently dropped.
    pub fn resolve(
        &mut self,
        paths: &HashSet<String>,
    ) -> Result<HashMap<String, ResolvedRecord>, LedgerError> {
        let missing: Vec<String> = paths
            .iter()
            .filter(|p| !self.cache.entries.contains_key(*p))
            .cloned()
            .collect();
        if !missing.is_empty() {
            self.open_shelves()?;
            let shelves = self.shelves.as_ref().expect("just opened");
            let localization = self.localization.as_ref().expect("just opened");
            let mut fresh = HashMap::with_capacity(missing.len());
            for path in &missing {
                let mut resolved = ResolvedRecord::default();
                for shelf in shelves {
                    if let Some(essentials) = shelf.record_essentials(path)? {
                        let tag = essentials
                            .item_name_tag
                            .as_deref()
                            .or(essentials.loot_randomizer_name.as_deref())
                            .or(essentials.description.as_deref());
                        resolved.name = tag.and_then(|t| localization.get(t).cloned());
                        resolved.tier = classification_tier(essentials.classification.as_deref());
                        resolved.classification = essentials.classification;
                        resolved.slot_class = essentials.class;
                        break;
                    }
                }
                fresh.insert(path.clone(), resolved);
            }
            self.cache.entries.extend(fresh);
            self.write_cache()?;
        }
        Ok(paths
            .iter()
            .map(|p| {
                (
                    p.clone(),
                    self.cache.entries.get(p).cloned().unwrap_or_default(),
                )
            })
            .collect())
    }

    fn open_shelves(&mut self) -> Result<(), LedgerError> {
        if self.shelves.is_some() {
            return Ok(());
        }
        let mut shelves = Vec::new();
        for rel in DATABASE_SHELVES {
            let path = self.install_root.join(rel);
            if !path.is_file() {
                continue; // expansions are optional — base game alone resolves
            }
            let data = guarded_read(&path, &self.install_root, &self.cache_dir)?;
            shelves.push(ArzShelf::open(data)?);
        }
        if shelves.is_empty() {
            return Err(LedgerError::CodexShelfMissing {
                detail: format!(
                    "no database.arz under {} — point me at the install",
                    self.install_root.display()
                ),
            });
        }
        let mut localization = HashMap::new();
        for rel in TEXT_SHELVES {
            let path = self.install_root.join(rel);
            if !path.is_file() {
                continue;
            }
            let data = guarded_read(&path, &self.install_root, &self.cache_dir)?;
            parse_text_arc(&data, &mut localization)?;
        }
        self.shelves = Some(shelves);
        self.localization = Some(localization);
        Ok(())
    }

    fn write_cache(&self) -> Result<(), LedgerError> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| LedgerError::unreadable(&self.cache_dir, &e))?;
        let path = self.cache_dir.join(CACHE_FILE);
        shelf_guard(&path, &self.install_root, &self.cache_dir)?;
        let bytes =
            serde_json::to_vec(&self.cache).map_err(|e| LedgerError::CodexShelfMissing {
                detail: format!("cache serialization failed: {e}"),
            })?;
        std::fs::write(&path, bytes).map_err(|e| LedgerError::unreadable(&path, &e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_refuse_to_open_any_path_outside_its_own_shelves() {
        // The countervailing RD-2 rule: the codex writes its cache, so it is
        // excluded from the read-only sweep — in exchange it can never open a
        // path under a discovered save root. The guard is the mechanism.
        let install = Path::new("/fake/steamapps/common/Grim Dawn");
        let cache = Path::new("/fake/appdata/smugglers-ledger");
        let save_root = Path::new("/fake/userdata/54202139/219990/remote/save/transfer.gst");
        assert!(shelf_guard(save_root, install, cache).is_err());
        assert!(shelf_guard(&install.join("database/database.arz"), install, cache).is_ok());
        assert!(shelf_guard(&cache.join("codex-cache.json"), install, cache).is_ok());
    }

    #[test]
    fn it_should_map_the_bench_enumerated_classifications_onto_the_ink_ramp() {
        assert_eq!(classification_tier(Some("Common")), 0);
        assert_eq!(classification_tier(Some("Magical")), 1);
        assert_eq!(classification_tier(Some("Rare")), 2);
        assert_eq!(classification_tier(Some("Epic")), 3);
        assert_eq!(classification_tier(Some("Legendary")), 4);
        assert_eq!(classification_tier(Some("Quest")), 5);
        assert_eq!(
            classification_tier(Some("SomethingNew")),
            0,
            "unknown → tier-0 fallback"
        );
        assert_eq!(classification_tier(None), 0);
    }
}
