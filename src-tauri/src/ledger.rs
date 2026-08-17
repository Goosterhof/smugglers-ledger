//! The ledger's spine — the loop nothing else owns (3.5A).
//!
//! `manifest.rs` parses ONE `player.gdc`; `warehouse.rs` parses ONE
//! `transfer.gst`; `codex.rs` resolves names. This module walks every
//! `save/main/_<Name>/player.gdc` under the root discovery chose, parses the
//! lot, resolves it, holds the resolved aggregate in Tauri managed state, and
//! serves the commands the frontend invokes.
//!
//! One bad save never sinks the fleet (the 1C ruling): a file that fails to
//! parse surfaces as a flagged hand carrying the voiced 4D copy, while every
//! other character parses and renders normally.
//!
//! Read-only always (RD-2): this module reads save bytes and never writes a
//! byte anywhere.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::codex::{Codex, ResolvedRecord, StatLine};
use crate::contraband::{Contraband, PlacedContraband, StashTab};
use crate::discovery::{self, CandidateRoot};
use crate::error::LedgerError;
use crate::manifest::{self, Manifest};
use crate::warehouse::{self, Warehouse};

/// The game's own equipment slot order inside the inventory block, per the
/// reference implementations — the location string's vocabulary.
pub const EQUIPMENT_SLOT_NAMES: [&str; 12] = [
    "HEAD",
    "AMULET",
    "CHEST",
    "LEGS",
    "SHOULDERS",
    "GLOVES",
    "BELT",
    "FEET",
    "RING I",
    "RING II",
    "RELIC",
    "MEDAL",
];

// ---------------------------------------------------------------------------
// The aggregate
// ---------------------------------------------------------------------------

/// One hand in THE HANDS rail: a character that parsed, or one that is
/// flagged — never silently missing.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hand {
    /// Directory name (`_Spinny`) when the header never parsed; the true
    /// character name once it did.
    pub name: String,
    pub level: u32,
    pub class_tag: String,
    pub hardcore: bool,
    pub iron: u32,
    /// The voiced flag when the cipher would not turn (4D's struck state).
    pub flagged: Option<String>,
    #[serde(skip)]
    pub manifest: Option<Manifest>,
}

/// The whole hoard: every hand, the warehouse, and the codex's answers.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hoard {
    pub root: PathBuf,
    pub hands: Vec<Hand>,
    #[serde(skip)]
    pub warehouse: Option<Warehouse>,
    /// The shared stash's own flag, when transfer.gst would not turn.
    pub warehouse_flagged: Option<String>,
    #[serde(skip)]
    pub resolved: std::collections::HashMap<String, ResolvedRecord>,
    pub last_turned_epoch_ms: u64,
}

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Walk one root into an unresolved hoard: every character save plus the
/// transfer stash, flagged-not-fatal per file.
pub fn assemble_hoard(root: &Path) -> Hoard {
    let mut hands = Vec::new();
    for save in discovery::character_saves(root) {
        let dir_name = save
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().trim_start_matches('_').to_string())
            .unwrap_or_else(|| "unknown hand".into());
        match std::fs::read(&save)
            .map_err(|e| LedgerError::unreadable(&save, &e))
            .and_then(|bytes| manifest::parse_player(&bytes))
        {
            Ok(parsed) => hands.push(Hand {
                name: parsed.header.name.clone(),
                level: parsed.header.level,
                class_tag: parsed.header.class_tag.clone(),
                hardcore: parsed.header.hardcore,
                iron: parsed.info.iron,
                flagged: None,
                manifest: Some(parsed),
            }),
            Err(err) => hands.push(Hand {
                name: dir_name,
                level: 0,
                class_tag: String::new(),
                hardcore: false,
                iron: 0,
                flagged: Some(err.to_string()),
                manifest: None,
            }),
        }
    }

    let (warehouse, warehouse_flagged) = match discovery::transfer_stash(root) {
        Some(path) => match std::fs::read(&path)
            .map_err(|e| LedgerError::unreadable(&path, &e))
            .and_then(|bytes| warehouse::parse_stash(&bytes))
        {
            Ok(parsed) => (Some(parsed), None),
            Err(err) => (None, Some(err.to_string())),
        },
        None => (None, None),
    };

    Hoard {
        root: root.to_path_buf(),
        hands,
        warehouse,
        warehouse_flagged,
        resolved: std::collections::HashMap::new(),
        last_turned_epoch_ms: now_epoch_ms(),
    }
}

/// Every distinct base-record path in the hoard — the codex's worklist.
pub fn distinct_records(hoard: &Hoard) -> HashSet<String> {
    let mut records = HashSet::new();
    let mut add = |item: &Contraband| {
        if !item.base_name.is_empty() {
            records.insert(item.base_name.clone());
        }
        for affix in [
            &item.prefix_name,
            &item.suffix_name,
            &item.component_name,
            &item.augment_name,
        ] {
            if !affix.is_empty() {
                records.insert(affix.clone());
            }
        }
    };
    for hand in &hoard.hands {
        if let Some(manifest) = &hand.manifest {
            for sack in &manifest.inventory.sacks {
                for placed in sack {
                    add(&placed.item);
                }
            }
            for slot in manifest
                .inventory
                .equipment
                .iter()
                .chain(&manifest.inventory.weapon_set_1)
                .chain(&manifest.inventory.weapon_set_2)
            {
                add(&slot.item);
            }
            for tab in &manifest.personal_stash {
                for placed in &tab.items {
                    add(&placed.item);
                }
            }
        }
    }
    if let Some(warehouse) = &hoard.warehouse {
        for tab in &warehouse.tabs {
            for placed in &tab.items {
                add(&placed.item);
            }
        }
    }
    records
}

// ---------------------------------------------------------------------------
// Projections — what the commands serve
// ---------------------------------------------------------------------------

/// One item as a panel renders it: resolved name when the codex knows it,
/// the raw record path otherwise — visible AND searchable either way (4C).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedContraband {
    pub name: Option<String>,
    pub record_path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub tier: u8,
    pub slot_class: Option<String>,
    pub component: Option<String>,
    pub augment: Option<String>,
    pub seed: u32,
    /// The item's full stat lines — base record + every affix + component +
    /// augment, aggregated in read order.
    pub stats: Vec<StatLine>,
    /// The item's skill grafts — plus-skills, mastery grants, and the Monster
    /// Infrequent modifier lines — aggregated the same way. The skill search
    /// matches on these labels.
    pub skills: Vec<StatLine>,
    /// The base record's icon bitmap path (`.tex` in Items.arc), if any.
    pub bitmap: Option<String>,
    pub stack: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedTab {
    pub width: u32,
    pub height: u32,
    pub items: Vec<NamedContraband>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSheet {
    pub name: String,
    pub level: u32,
    pub class_tag: String,
    pub hardcore: bool,
    pub iron: u32,
    pub flagged: Option<String>,
    /// Twelve slots in the game's own order; an empty slot is `None`.
    pub equipment: Vec<Option<NamedContraband>>,
    pub weapon_set_1: Vec<Option<NamedContraband>>,
    pub weapon_set_2: Vec<Option<NamedContraband>>,
    pub bags: Vec<Vec<NamedContraband>>,
    pub personal_stash: Vec<NamedTab>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseSheet {
    pub flagged: Option<String>,
    pub tabs: Vec<NamedTab>,
}

/// One search hit. `location` is non-null and non-empty for 100% of results
/// — the gated contract: a result that says "you have it" without saying
/// where is not an overview, it's a rumor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerHit {
    pub name: Option<String>,
    pub record_path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub tier: u8,
    pub slot_class: Option<String>,
    pub component: Option<String>,
    pub augment: Option<String>,
    pub seed: u32,
    pub stats: Vec<StatLine>,
    pub skills: Vec<StatLine>,
    pub bitmap: Option<String>,
    pub stack: u32,
    /// The owner in the location's own casing — a character name, or
    /// "SHARED STASH" for the warehouse.
    pub hand: String,
    /// The container class, structured for the docket's filters:
    /// "EQUIPPED" | "BAGS" | "WEAPON SET" | "PERSONAL STASH" | "SHARED STASH".
    pub place: String,
    pub location: String,
}

fn name_item(hoard: &Hoard, item: &Contraband, x: u32, y: u32) -> NamedContraband {
    let resolved = hoard.resolved.get(&item.base_name);
    let affix_name = |affix: &String| -> Option<String> {
        if affix.is_empty() {
            return None;
        }
        hoard
            .resolved
            .get(affix)
            .and_then(|r| r.name.clone())
            .or_else(|| Some(affix.clone()))
    };
    // The item's stats and skill grafts are its base record's plus every
    // affix, component, and augment it carries — collected in the order a
    // player reads them (a "+1 to Cadence" suffix counts as much as the base).
    let mut stats: Vec<StatLine> = Vec::new();
    let mut skills: Vec<StatLine> = Vec::new();
    for record_name in [
        &item.base_name,
        &item.prefix_name,
        &item.suffix_name,
        &item.component_name,
        &item.augment_name,
    ] {
        if let Some(r) = hoard.resolved.get(record_name) {
            stats.extend(r.stats.iter().cloned());
            skills.extend(r.skills.iter().cloned());
        }
    }
    NamedContraband {
        name: resolved.and_then(|r| r.name.clone()),
        record_path: item.base_name.clone(),
        prefix: affix_name(&item.prefix_name),
        suffix: affix_name(&item.suffix_name),
        tier: resolved.map(|r| r.tier).unwrap_or(0),
        slot_class: resolved.and_then(|r| r.slot_class.clone()),
        component: affix_name(&item.component_name),
        augment: affix_name(&item.augment_name),
        seed: item.seed,
        stats,
        skills,
        bitmap: resolved.and_then(|r| r.bitmap.clone()),
        stack: item.stack_count.max(1),
        x,
        y,
    }
}

fn name_placed(hoard: &Hoard, placed: &PlacedContraband) -> NamedContraband {
    name_item(hoard, &placed.item, placed.x, placed.y)
}

fn name_tab(hoard: &Hoard, tab: &StashTab) -> NamedTab {
    NamedTab {
        width: tab.width,
        height: tab.height,
        items: tab.items.iter().map(|p| name_placed(hoard, p)).collect(),
    }
}

pub fn character_sheets(hoard: &Hoard) -> Vec<CharacterSheet> {
    hoard
        .hands
        .iter()
        .map(|hand| {
            let dressed = |slots: &[manifest::DressedSlot]| -> Vec<Option<NamedContraband>> {
                slots
                    .iter()
                    .map(|slot| {
                        (!slot.item.base_name.is_empty())
                            .then(|| name_item(hoard, &slot.item, 0, 0))
                    })
                    .collect()
            };
            match &hand.manifest {
                Some(m) => CharacterSheet {
                    name: hand.name.clone(),
                    level: hand.level,
                    class_tag: hand.class_tag.clone(),
                    hardcore: hand.hardcore,
                    iron: hand.iron,
                    flagged: None,
                    equipment: dressed(&m.inventory.equipment),
                    weapon_set_1: dressed(&m.inventory.weapon_set_1),
                    weapon_set_2: dressed(&m.inventory.weapon_set_2),
                    bags: m
                        .inventory
                        .sacks
                        .iter()
                        .map(|sack| sack.iter().map(|p| name_placed(hoard, p)).collect())
                        .collect(),
                    personal_stash: m
                        .personal_stash
                        .iter()
                        .map(|t| name_tab(hoard, t))
                        .collect(),
                },
                None => CharacterSheet {
                    name: hand.name.clone(),
                    level: hand.level,
                    class_tag: hand.class_tag.clone(),
                    hardcore: hand.hardcore,
                    iron: hand.iron,
                    flagged: hand.flagged.clone(),
                    equipment: Vec::new(),
                    weapon_set_1: Vec::new(),
                    weapon_set_2: Vec::new(),
                    bags: Vec::new(),
                    personal_stash: Vec::new(),
                },
            }
        })
        .collect()
}

pub fn warehouse_sheet(hoard: &Hoard) -> WarehouseSheet {
    WarehouseSheet {
        flagged: hoard.warehouse_flagged.clone(),
        tabs: hoard
            .warehouse
            .as_ref()
            .map(|w| w.tabs.iter().map(|t| name_tab(hoard, t)).collect())
            .unwrap_or_default(),
    }
}

/// Search every location the hoard has: bags, equipment, both weapon sets,
/// personal stashes, and the shared warehouse. Matches the resolved display
/// name OR the raw record path (case-insensitive), so an item the codex
/// missed is findable by the same string the UI shows for it (the 4C ruling)
/// — AND the skill-graft labels, so searching a skill surfaces every item
/// that grants "+N to it" and every Monster Infrequent that modifies it.
pub fn search_hoard(hoard: &Hoard, query: &str) -> Vec<LedgerHit> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let matches = |named: &NamedContraband| -> bool {
        named
            .name
            .as_deref()
            .is_some_and(|n| n.to_lowercase().contains(&needle))
            || named.record_path.to_lowercase().contains(&needle)
            || named
                .prefix
                .as_deref()
                .is_some_and(|p| p.to_lowercase().contains(&needle))
            || named
                .suffix
                .as_deref()
                .is_some_and(|s| s.to_lowercase().contains(&needle))
            || named
                .component
                .as_deref()
                .is_some_and(|c| c.to_lowercase().contains(&needle))
            || named
                .augment
                .as_deref()
                .is_some_and(|a| a.to_lowercase().contains(&needle))
            || named
                .skills
                .iter()
                .any(|s| s.label.to_lowercase().contains(&needle))
    };
    let mut push = |named: NamedContraband, hand: &str, place: &str, location: String| {
        debug_assert!(!location.is_empty(), "the location contract is absolute");
        if matches(&named) {
            hits.push(LedgerHit {
                name: named.name,
                record_path: named.record_path,
                prefix: named.prefix,
                suffix: named.suffix,
                tier: named.tier,
                slot_class: named.slot_class,
                component: named.component,
                augment: named.augment,
                seed: named.seed,
                stats: named.stats,
                skills: named.skills,
                bitmap: named.bitmap,
                stack: named.stack,
                hand: hand.to_string(),
                place: place.to_string(),
                location,
            });
        }
    };

    for hand in &hoard.hands {
        let Some(m) = &hand.manifest else { continue };
        let owner = hand.name.to_uppercase();
        for (b, sack) in m.inventory.sacks.iter().enumerate() {
            for placed in sack {
                push(
                    name_placed(hoard, placed),
                    &owner,
                    "BAGS",
                    format!(
                        "{owner} — BAGS, BAG {}, CELL {},{}",
                        b + 1,
                        placed.x,
                        placed.y
                    ),
                );
            }
        }
        for (i, slot) in m.inventory.equipment.iter().enumerate() {
            if !slot.item.base_name.is_empty() {
                push(
                    name_item(hoard, &slot.item, 0, 0),
                    &owner,
                    "EQUIPPED",
                    format!("{owner} — EQUIPPED, {}", EQUIPMENT_SLOT_NAMES[i]),
                );
            }
        }
        for (set_name, set) in [
            ("I", &m.inventory.weapon_set_1),
            ("II", &m.inventory.weapon_set_2),
        ] {
            for (i, slot) in set.iter().enumerate() {
                if !slot.item.base_name.is_empty() {
                    push(
                        name_item(hoard, &slot.item, 0, 0),
                        &owner,
                        "WEAPON SET",
                        format!(
                            "{owner} — WEAPON SET {set_name}, {}",
                            if i == 0 { "MAIN HAND" } else { "OFF HAND" }
                        ),
                    );
                }
            }
        }
        for (t, tab) in m.personal_stash.iter().enumerate() {
            for placed in &tab.items {
                push(
                    name_placed(hoard, placed),
                    &owner,
                    "PERSONAL STASH",
                    format!(
                        "{owner} — PERSONAL STASH, TAB {}, CELL {},{}",
                        t + 1,
                        placed.x,
                        placed.y
                    ),
                );
            }
        }
    }
    if let Some(warehouse) = &hoard.warehouse {
        for (t, tab) in warehouse.tabs.iter().enumerate() {
            for placed in &tab.items {
                push(
                    name_placed(hoard, placed),
                    "SHARED STASH",
                    "SHARED STASH",
                    format!(
                        "SHARED STASH — TAB {}, CELL {},{}",
                        t + 1,
                        placed.x,
                        placed.y
                    ),
                );
            }
        }
    }
    hits
}

// ---------------------------------------------------------------------------
// Managed state + commands
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LedgerState {
    pub inner: Mutex<LedgerInner>,
}

#[derive(Default)]
pub struct LedgerInner {
    pub roots: Vec<CandidateRoot>,
    pub hoard: Option<Hoard>,
    /// The 4D install-not-found state, carried to the frontend.
    pub codex_note: Option<String>,
    /// True while a first-run (cold) codex resolve is walking the shelves —
    /// 4D's "the codex is reading the shelves" state.
    pub codex_cold: bool,
    /// The resolved game-install root — where Items.arc lives, for icons.
    pub install_root: Option<PathBuf>,
}

/// Discovery + parse + resolve, into managed state. Called at startup, on
/// `save-changed` from the watch, and on a root switch.
///
/// Two stages, and `on_stage` fires after each store: the parsed hoard lands
/// first (raw record paths, instantly readable), then — only when the codex
/// cache cannot already answer — the cold shelf-walk fills the names in. The
/// warm path stores exactly once, resolved: that is the state The Vision's
/// "parsed before the window finishes drawing" sentence describes.
pub fn turn_the_ledger<F: Fn()>(
    state: &LedgerState,
    root_override: Option<PathBuf>,
    on_stage: F,
) -> Result<(), LedgerError> {
    let steam = discovery::steam_install_root();
    let documents = dirs::document_dir();
    let mut roots = discovery::discover_roots(steam.as_deref(), documents.as_deref());

    let chosen = match root_override {
        Some(path) => {
            // A manual pick or a switch: keep it first in the rail.
            roots.retain(|r| r.path != path);
            path
        }
        None => match roots.first() {
            Some(freshest) => freshest.path.clone(),
            None => {
                let mut inner = state.inner.lock().expect("ledger state poisoned");
                inner.roots = Vec::new();
                inner.hoard = None;
                on_stage();
                return Err(LedgerError::NoHoardFound);
            }
        },
    };

    let hoard = assemble_hoard(&chosen);
    let records = distinct_records(&hoard);

    // Open the codex (cache check only — no shelf is read here).
    let mut codex_note = None;
    let install_root = discovery::game_install_root(steam.as_deref());
    let mut codex = match install_root.clone() {
        Some(install) => match Codex::open(&install, &codex_cache_dir()) {
            Ok(codex) => Some(codex),
            Err(err) => {
                codex_note = Some(err.to_string());
                None
            }
        },
        None => {
            codex_note = Some(
                "The codex has no shelf for these records yet — point me at the install.".into(),
            );
            None
        }
    };
    let cold = codex
        .as_ref()
        .map(|c| !c.cache_answers(records.iter()))
        .unwrap_or(false);

    if cold {
        // Stage 1: the parsed hoard, readable now, names pending.
        let mut inner = state.inner.lock().expect("ledger state poisoned");
        inner.roots = roots.clone();
        inner.codex_note = codex_note.clone();
        inner.codex_cold = true;
        inner.install_root = install_root.clone();
        inner.hoard = Some(assemble_hoard(&chosen));
        drop(inner);
        on_stage();
    }

    let mut hoard = hoard;
    if let Some(codex) = codex.as_mut() {
        match codex.resolve(&records) {
            Ok(resolved) => hoard.resolved = resolved,
            Err(err) => codex_note = Some(err.to_string()),
        }
    }

    let mut inner = state.inner.lock().expect("ledger state poisoned");
    // The chosen root leads the rail; every other discovered root stays
    // visible as a switch (RD-4).
    inner.roots = roots;
    inner.codex_note = codex_note;
    inner.codex_cold = false;
    inner.install_root = install_root;
    inner.hoard = Some(hoard);
    drop(inner);
    on_stage();
    Ok(())
}

fn codex_cache_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("smugglers-ledger")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerOverview {
    pub chosen_root: Option<PathBuf>,
    pub roots: Vec<CandidateRoot>,
    pub last_turned_epoch_ms: u64,
    pub codex_note: Option<String>,
    pub codex_cold: bool,
}

#[tauri::command]
pub fn list_characters(
    state: tauri::State<'_, LedgerState>,
) -> Result<Vec<CharacterSheet>, LedgerError> {
    let inner = state.inner.lock().expect("ledger state poisoned");
    let hoard = inner.hoard.as_ref().ok_or(LedgerError::NoHoardFound)?;
    Ok(character_sheets(hoard))
}

#[tauri::command]
pub fn list_stash(state: tauri::State<'_, LedgerState>) -> Result<WarehouseSheet, LedgerError> {
    let inner = state.inner.lock().expect("ledger state poisoned");
    let hoard = inner.hoard.as_ref().ok_or(LedgerError::NoHoardFound)?;
    Ok(warehouse_sheet(hoard))
}

#[tauri::command]
pub fn search_ledger(
    state: tauri::State<'_, LedgerState>,
    query: String,
) -> Result<Vec<LedgerHit>, LedgerError> {
    let inner = state.inner.lock().expect("ledger state poisoned");
    let hoard = inner.hoard.as_ref().ok_or(LedgerError::NoHoardFound)?;
    Ok(search_hoard(hoard, &query))
}

#[tauri::command]
pub fn ledger_overview(state: tauri::State<'_, LedgerState>) -> LedgerOverview {
    let inner = state.inner.lock().expect("ledger state poisoned");
    LedgerOverview {
        chosen_root: inner.hoard.as_ref().map(|h| h.root.clone()),
        roots: inner.roots.clone(),
        last_turned_epoch_ms: inner
            .hoard
            .as_ref()
            .map(|h| h.last_turned_epoch_ms)
            .unwrap_or(0),
        codex_note: inner.codex_note.clone(),
        codex_cold: inner.codex_cold,
    }
}

/// The decoded icon for one bitmap, as a `data:` PNG URL the webview can put
/// in an `<img>`. Resolved lazily on demand (the docket asks when it opens),
/// decoded from the user's own Items.arc, cached in the icon cabinet.
#[tauri::command]
pub fn item_icon(
    state: tauri::State<'_, LedgerState>,
    icons: tauri::State<'_, crate::icons::IconState>,
    bitmap: String,
) -> Option<String> {
    let install = state.inner.lock().ok()?.install_root.clone()?;
    let png = icons.icon_png(&install, &bitmap)?;
    use base64::Engine;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

// The root switch lives in lib.rs (`switch_root`): it must re-arm the watch
// on the new root, which needs the app handle — not this module's business.
