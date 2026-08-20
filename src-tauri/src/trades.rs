//! THE TRADES — the game's own mastery trees, read off the shelves.
//!
//! A save says a character carries `tagSkillClassName0106`; it never says
//! what a Soldier can learn. That lives in the install, split across two
//! record families the Ledger reads together:
//!
//! - `records/ui/skills/classNN/classtable.dbr` — the mastery's name tag, its
//!   blurb, and the list of skill BUTTONS on its panel. Each button record
//!   carries the panel coordinate the game draws it at (`bitmapPositionX/Y`),
//!   whether it is drawn round or square (`isCircular`), and the skill record
//!   it points at.
//! - `records/skills/playerclassNN/*.dbr` — the skills themselves: display
//!   name, description, max rank, ultimate rank, tier, icon.
//!
//! **The connections are the game's own, not a guess.** A chain's root skill
//! carries `skillConnectionOn` — one connector texture per column step of the
//! run the game draws to its right (`branchup`, a transmuter stub, then a run
//! of `center` segments). The array's LENGTH is therefore how far right the
//! chain reaches, and every node inside that reach on the root's own line is
//! a link in it. A node with no connector array and no root reaching it is a
//! standalone — three of Soldier's passives sit side by side in one row with
//! no line drawn between them, and this reader draws none either.
//!
//! Same legal floor as the codex (RD-3): every byte read here comes from the
//! user's own licensed install at runtime, cached for that user, never
//! bundled and never shipped.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::codex::{ArzShelf, Field};

/// How many `classNN` panels to probe. Nine masteries shipped through the
/// third expansion and the tenth (Berserker) came with Fangs of Asterkarn —
/// probing past the last one costs four failed directory lookups and means a
/// future mastery appears in the Ledger without a code change.
const CLASS_PROBE_CEILING: u32 = 16;

/// The mastery level each tier column unlocks at — the nine small numbers the
/// game prints under the columns of the skill panel.
///
/// **This is the one value in the Ledger not read from the install.**
/// `records/ui/skills/classcommon/skills_classpanelconfiguration.dbr` carries
/// the nine milestone *widgets* (`masteryMilestoneNumber1..9`, text boxes at
/// x = 246…886) and `masteryMilestoneValueMax`, but the numbers themselves are
/// engine-side: a full sweep of the arz string tables for "milestone" turns up
/// the field names and nothing else. Investor-confirmed against his own skill
/// panel, 2026-08-19.
const TIER_UNLOCK: [u32; 9] = [1, 5, 10, 15, 20, 25, 32, 40, 50];

/// The panel x of tier 1, and the step between columns — measured off the
/// button records and cross-checked against the milestone widgets, which sit
/// at exactly these nine x values.
const TIER_ONE_X: i32 = 246;
const TIER_STEP_X: i32 = 80;

/// How far off its parent's line a transmuter's branch sits. The observed
/// offsets are 32 and 38 panel pixels (branch down and branch up); the next
/// row is 108 away, so the band separates them with room to spare.
const ROW_BAND: i32 = 56;

/// What a node is, in the vocabulary the skill panel itself uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillKind {
    /// The mastery bar — the only "skill" whose level is bought by the bar.
    Mastery,
    Active,
    Passive,
    /// A round node hanging off a skill: the upgrades that read "+X to …".
    Modifier,
    /// A transmuter — the conversion node on the branch stub.
    Transmuter,
}

/// One mastery panel, whole.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasteryTree {
    /// 1–10 today: the `classNN` the records live under.
    pub class_index: u32,
    pub name: String,
    pub blurb: String,
    /// The mastery bar's own record — the one a save carries a LEVEL for, and
    /// the one a "+1 to all skills in Soldier" graft names.
    pub bar_record: String,
    pub bar_max_level: u32,
    pub bar_icon: Option<String>,
    /// The nine numbers the game prints under the panel's columns: the
    /// mastery level each tier opens at. Served with the tree so the panel
    /// letters its rule from the same authority the nodes were read against,
    /// never from a copy of the table.
    pub tier_unlocks: Vec<u32>,
    pub nodes: Vec<SkillNode>,
}

/// One skill on the panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillNode {
    pub record: String,
    pub name: String,
    pub blurb: String,
    /// The column, 1–9. The tier IS the column: x = 246 + 80·(tier − 1).
    pub tier: u32,
    /// The mastery level the column unlocks at — see [`TIER_UNLOCK`].
    pub unlock_level: u32,
    /// Ranks buyable with skill points.
    pub max_level: u32,
    /// The hard ceiling once gear's "+N to" grafts pile on top.
    pub ultimate_level: u32,
    pub kind: SkillKind,
    /// The game's own panel coordinates. The tree is laid out at them, so it
    /// reads the way the skill panel reads.
    pub x: i32,
    pub y: i32,
    pub circular: bool,
    /// The `.tex` inside the install's own UI.arc — decoded on demand.
    pub icon: Option<String>,
    /// The record this node hangs off, derived from the game's connector run.
    /// `None` = a chain root, or a standalone with no line drawn to it.
    pub parent: Option<String>,
    /// A transmuter's conversion, spelled the way the game spells it:
    /// "33% Physical → Elemental".
    pub conversion: Option<String>,
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

fn wants_classtable(field: &str) -> bool {
    matches!(
        field,
        "skillTabTitle" | "skillPaneDescriptionTag" | "tabSkillButtons"
    )
}

fn wants_button(field: &str) -> bool {
    matches!(
        field,
        "bitmapPositionX" | "bitmapPositionY" | "isCircular" | "skillName"
    )
}

fn wants_skill(field: &str) -> bool {
    matches!(
        field,
        "skillDisplayName"
            | "skillBaseDescription"
            | "skillMaxLevel"
            | "skillUltimateLevel"
            | "skillTier"
            | "skillUpBitmapName"
            | "Class"
            | "skillConnectionOn"
            | "buffSkillName"
            | "petSkillName"
            | "conversionInType"
            | "conversionOutType"
            | "conversionPercentage"
    )
}

/// The first shelf that holds the record wins — the same precedence the item
/// resolve uses. A shelf that errors on one record is skipped, never fatal:
/// a missing panel costs one mastery, not the whole page.
fn fields(
    shelves: &[ArzShelf],
    path: &str,
    keep: &dyn Fn(&str) -> bool,
) -> Option<HashMap<String, Field>> {
    shelves
        .iter()
        .find_map(|shelf| shelf.record_fields(path, keep).ok().flatten())
}

/// A localization tag through the game's own text tables, with the game's
/// inline formatting codes (`^o` and kin) taken back out.
fn say(localization: &HashMap<String, String>, tag: Option<&str>) -> String {
    tag.and_then(|t| localization.get(t))
        .map(|text| plain(text))
        .unwrap_or_default()
}

/// Grim Dawn writes colour and style changes inline as `^` plus one letter.
/// They are instructions to the game's own renderer, not words — strip them
/// and collapse what the removal leaves behind.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '^' {
            chars.next();
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The column a panel coordinate sits in. Tier 1 is x = 246 and every column
/// is 80 further right; anything left of tier 1 is the mastery bar, which
/// hangs below the panel at x = 0 and belongs to no column.
fn tier_at(x: i32) -> u32 {
    if x < TIER_ONE_X {
        return 0;
    }
    let column = (x - TIER_ONE_X + TIER_STEP_X / 2) / TIER_STEP_X + 1;
    column.clamp(1, TIER_UNLOCK.len() as i32) as u32
}

/// The mastery level a tier unlocks at. Tier 0 (the bar) unlocks at 1 — you
/// buy the bar to open the panel at all.
fn unlock_level(tier: u32) -> u32 {
    match tier {
        0 => 1,
        t => TIER_UNLOCK[(t as usize - 1).min(TIER_UNLOCK.len() - 1)],
    }
}

/// A skill record's display tag, chasing the indirection an aura or pet skill
/// hides behind (`buffSkillName` / `petSkillName` — the same hop the codex's
/// graft resolver walks).
fn skill_face(shelves: &[ArzShelf], record: &str) -> Option<(String, HashMap<String, Field>)> {
    let mut current = record.to_string();
    let first = fields(shelves, &current, &wants_skill)?;
    if first.contains_key("skillDisplayName") {
        return Some((current, first));
    }
    let mut carried = first;
    for _hop in 0..3 {
        let next = carried
            .get("buffSkillName")
            .or_else(|| carried.get("petSkillName"))
            .and_then(Field::text)
            .map(str::to_string)?;
        current = next;
        carried = fields(shelves, &current, &wants_skill)?;
        if carried.contains_key("skillDisplayName") {
            return Some((current, carried));
        }
    }
    None
}

/// "33% Physical → Elemental", from a transmuter's own three fields.
fn conversion_of(skill: &HashMap<String, Field>) -> Option<String> {
    let from = skill.get("conversionInType")?.text()?;
    let to = skill.get("conversionOutType")?.text()?;
    let percentages = match skill.get("conversionPercentage") {
        Some(Field::Floats(values)) => values.iter().map(|v| *v as i32).collect::<Vec<_>>(),
        Some(Field::Ints(values)) => values.clone(),
        _ => Vec::new(),
    };
    let ramp = match (percentages.first(), percentages.last()) {
        (Some(first), Some(last)) if first != last => format!("{first}–{last}% "),
        (Some(first), _) => format!("{first}% "),
        _ => String::new(),
    };
    Some(format!("{ramp}{from} → {to}"))
}

/// Read every mastery panel the install carries.
pub(crate) fn read_trades(
    shelves: &[ArzShelf],
    localization: &HashMap<String, String>,
) -> Vec<MasteryTree> {
    (1..=CLASS_PROBE_CEILING)
        .filter_map(|class_index| read_one_trade(shelves, localization, class_index))
        .collect()
}

fn read_one_trade(
    shelves: &[ArzShelf],
    localization: &HashMap<String, String>,
    class_index: u32,
) -> Option<MasteryTree> {
    let table = fields(
        shelves,
        &format!("records/ui/skills/class{class_index:02}/classtable.dbr"),
        &wants_classtable,
    )?;
    let name = say(
        localization,
        table.get("skillTabTitle").and_then(Field::text),
    );
    let blurb = say(
        localization,
        table.get("skillPaneDescriptionTag").and_then(Field::text),
    );

    let mut nodes: Vec<SkillNode> = Vec::new();
    let mut spans: Vec<usize> = Vec::new();
    let mut bar: Option<(String, u32, Option<String>)> = None;

    for button_path in table
        .get("tabSkillButtons")
        .map(Field::texts)
        .unwrap_or_default()
    {
        let Some(button) = fields(shelves, button_path, &wants_button) else {
            continue;
        };
        let Some(record) = button.get("skillName").and_then(Field::text) else {
            continue;
        };
        let Some((face_record, skill)) = skill_face(shelves, record) else {
            continue;
        };
        let x = button
            .get("bitmapPositionX")
            .and_then(Field::int)
            .unwrap_or(0);
        let y = button
            .get("bitmapPositionY")
            .and_then(Field::int)
            .unwrap_or(0);
        let icon = skill
            .get("skillUpBitmapName")
            .and_then(Field::text)
            .map(str::to_string);
        let max_level = skill
            .get("skillMaxLevel")
            .and_then(Field::int)
            .unwrap_or(1)
            .max(1) as u32;
        let class = skill.get("Class").and_then(Field::text).unwrap_or_default();

        // The mastery bar is not a node on the panel — it is the bar under it,
        // and the only record a save carries a bar level for.
        if class.contains("Mastery") {
            bar = Some((record.to_string(), max_level, icon));
            continue;
        }

        let tier = tier_at(x);
        let kind = if class.contains("Transmuter") {
            SkillKind::Transmuter
        } else if class.contains("Passive") {
            SkillKind::Passive
        } else {
            SkillKind::Active
        };
        spans.push(
            skill
                .get("skillConnectionOn")
                .map(|f| f.texts().len())
                .unwrap_or(0),
        );
        nodes.push(SkillNode {
            // The record the SAVE names is the button's own target; the face
            // record (one hop deeper for an aura) only lends its words.
            record: record.to_string(),
            name: say(
                localization,
                skill.get("skillDisplayName").and_then(Field::text),
            )
            .then_or(&face_record),
            blurb: say(
                localization,
                skill.get("skillBaseDescription").and_then(Field::text),
            ),
            tier,
            unlock_level: unlock_level(tier),
            max_level,
            ultimate_level: skill
                .get("skillUltimateLevel")
                .and_then(Field::int)
                .unwrap_or(max_level as i32)
                .max(max_level as i32) as u32,
            kind,
            x,
            y,
            circular: button.get("isCircular").is_some_and(Field::truthy),
            icon,
            parent: None,
            conversion: conversion_of(&skill),
        });
    }

    hang_chains(&mut nodes, &spans);
    let (bar_record, bar_max_level, bar_icon) = bar?;
    Some(MasteryTree {
        class_index,
        name,
        blurb,
        bar_record,
        bar_max_level,
        bar_icon,
        tier_unlocks: TIER_UNLOCK.to_vec(),
        nodes,
    })
}

/// A display name that fell through to nothing is the record path — visible
/// AND searchable by the same string, the 4C ruling applied to skills.
trait OrRecord {
    fn then_or(self, record: &str) -> String;
}

impl OrRecord for String {
    fn then_or(self, record: &str) -> String {
        if self.is_empty() {
            record.to_string()
        } else {
            self
        }
    }
}

/// Draw the game's own lines: give every node the parent the connector runs
/// imply. `spans[i]` is how many column steps node `i`'s connector array
/// covers — 0 for every node that owns no run.
///
/// A root's run claims each node to its right, inside its reach, on its own
/// line: those are the links of one chain, each hanging off the one before
/// it. A node inside the reach but OFF the line is the transmuter on the
/// branch stub, and hangs off the root itself. Nodes no run reaches keep no
/// parent — the panel draws them alone, and so does the Ledger.
pub(crate) fn hang_chains(nodes: &mut [SkillNode], spans: &[usize]) {
    let roots: Vec<usize> = (0..nodes.len()).filter(|i| spans[*i] > 0).collect();
    let mut parents: Vec<Option<usize>> = vec![None; nodes.len()];

    for &root in &roots {
        let reach = nodes[root].x + TIER_STEP_X * spans[root] as i32;
        let (line, branch): (Vec<usize>, Vec<usize>) = (0..nodes.len())
            .filter(|i| {
                *i != root
                    && spans[*i] == 0
                    && nodes[*i].x > nodes[root].x
                    && nodes[*i].x <= reach
                    && (nodes[*i].y - nodes[root].y).abs() <= ROW_BAND
                    // Two chains can share a row; the nearest root to the left
                    // owns the node, never a further one reaching over it.
                    && !roots.iter().any(|&other| {
                        other != root
                            && nodes[other].x > nodes[root].x
                            && nodes[other].x < nodes[*i].x
                            && (nodes[other].y - nodes[*i].y).abs() <= ROW_BAND
                    })
            })
            .partition(|i| nodes[*i].y == nodes[root].y);

        let mut previous = root;
        let mut in_line = line;
        in_line.sort_by_key(|i| nodes[*i].x);
        for member in in_line {
            parents[member] = Some(previous);
            previous = member;
        }
        for member in branch {
            parents[member] = Some(root);
        }
    }

    for (index, parent) in parents.into_iter().enumerate() {
        // A round node hanging off a skill is that skill's modifier — the
        // panel says so by drawing it round and connected.
        if let Some(parent) = parent {
            nodes[index].parent = Some(nodes[parent].record.clone());
            if nodes[index].circular && nodes[index].kind == SkillKind::Active {
                nodes[index].kind = SkillKind::Modifier;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(record: &str, x: i32, y: i32, circular: bool) -> SkillNode {
        SkillNode {
            record: record.into(),
            name: record.into(),
            blurb: String::new(),
            tier: tier_at(x),
            unlock_level: unlock_level(tier_at(x)),
            max_level: 16,
            ultimate_level: 26,
            kind: SkillKind::Active,
            x,
            y,
            circular,
            icon: None,
            parent: None,
            conversion: None,
        }
    }

    #[test]
    fn it_should_read_the_tier_column_off_the_panel_coordinate() {
        // The nine columns, measured off the milestone widgets themselves.
        for (index, x) in [246, 326, 406, 486, 566, 646, 726, 806, 886]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                tier_at(x),
                index as u32 + 1,
                "x {x} is column {}",
                index + 1
            );
        }
        // The mastery bar hangs below the panel at x = 0 — no column.
        assert_eq!(tier_at(0), 0);
    }

    #[test]
    fn it_should_unlock_each_tier_at_the_mastery_level_the_panel_prints() {
        assert_eq!(unlock_level(1), 1);
        assert_eq!(unlock_level(4), 15);
        assert_eq!(unlock_level(9), 50);
        // The bar itself is the first purchase — never gated behind a column.
        assert_eq!(unlock_level(0), 1);
    }

    #[test]
    fn it_should_hang_a_chain_off_the_run_the_connectors_draw() {
        // Soldier's Cadence row, at the game's own coordinates: the root at
        // tier 1 carries eight connectors (branch up, transmuter stub, six
        // centres), reaching tier 9 — Discord branches off it, Fighting Form
        // and Deadly Momentum link along the line.
        let mut nodes = vec![
            node("cadence1", 246, 319, false),
            node("cadence1b", 326, 281, true),
            node("cadence2", 486, 319, true),
            node("cadence3", 806, 319, true),
        ];
        hang_chains(&mut nodes, &[8, 0, 0, 0]);
        assert_eq!(nodes[0].parent, None, "the root hangs off nothing");
        assert_eq!(
            nodes[1].parent.as_deref(),
            Some("cadence1"),
            "the transmuter hangs off the root, not off the line"
        );
        assert_eq!(nodes[2].parent.as_deref(), Some("cadence1"));
        assert_eq!(
            nodes[3].parent.as_deref(),
            Some("cadence2"),
            "the line links node to node, never all of them to the root"
        );
        assert_eq!(nodes[2].kind, SkillKind::Modifier);
    }

    #[test]
    fn it_should_leave_a_row_of_standalone_passives_unconnected() {
        // Soldier's passive row: three round nodes side by side in one row,
        // none of them carrying a connector run. The panel draws no line
        // between them and neither do we — geometry alone would have.
        let mut nodes = vec![
            node("passive1", 326, 459, true),
            node("passiveshield", 406, 459, true),
            node("passive2", 486, 459, true),
        ];
        hang_chains(&mut nodes, &[0, 0, 0]);
        assert!(nodes.iter().all(|n| n.parent.is_none()));
        assert!(nodes.iter().all(|n| n.kind == SkillKind::Active));
    }

    #[test]
    fn it_should_not_let_one_run_reach_over_another_root() {
        // Two chains sharing a row: the second root's own member must not be
        // claimed by the first root's longer reach.
        let mut nodes = vec![
            node("first", 246, 179, false),
            node("firstUpgrade", 326, 179, true),
            node("second", 486, 179, false),
            node("secondUpgrade", 566, 179, true),
        ];
        hang_chains(&mut nodes, &[6, 0, 2, 0]);
        assert_eq!(nodes[1].parent.as_deref(), Some("first"));
        assert_eq!(nodes[3].parent.as_deref(), Some("second"));
    }

    #[test]
    fn it_should_strip_the_games_inline_formatting_from_a_description() {
        let raw = "Cadence strikes.  ^oEvery third hit lands harder.";
        assert_eq!(
            plain(raw),
            "Cadence strikes. Every third hit lands harder.",
            "the ^o is a renderer instruction, not a word"
        );
    }
}
