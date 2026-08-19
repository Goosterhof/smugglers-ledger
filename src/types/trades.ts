// THE TRADES — the shapes `trades.rs` and `ledger.rs::list_trades` serve.
// Field names follow the serde camelCase renames.

/** What a node is, in the skill panel's own vocabulary. */
export type SkillKind = "mastery" | "active" | "passive" | "modifier" | "transmuter";

export interface SkillNode {
  /** The skill record — the key a save's allocated ranks are filed under. */
  record: string;
  name: string;
  blurb: string;
  /** The column, 1–9. The tier IS the column on the game's own panel. */
  tier: number;
  /** The mastery level that column unlocks at. */
  unlockLevel: number;
  /** Ranks buyable with skill points. */
  maxLevel: number;
  /** The hard ceiling once gear's "+N to" grafts pile on top. */
  ultimateLevel: number;
  kind: SkillKind;
  /** The game's own panel coordinates — the tree is laid out at them. */
  x: number;
  y: number;
  circular: boolean;
  /** The `.tex` inside the install's own UI.arc, decoded on demand. */
  icon: string | null;
  /** The record this node hangs off — null for a root or a standalone. */
  parent: string | null;
  /** A transmuter's conversion: "33–100% Physical → Elemental". */
  conversion: string | null;
}

export interface MasteryTree {
  classIndex: number;
  name: string;
  blurb: string;
  /** The mastery bar's own record — what a save files the bar's level under. */
  barRecord: string;
  barMaxLevel: number;
  barIcon: string | null;
  /** The nine numbers the game prints under the panel's columns — the mastery
   * level each tier opens at. Served with the tree so the panel letters its
   * rule from the same authority the nodes were read against. */
  tierUnlocks: number[];
  nodes: SkillNode[];
}

export interface SkillRank {
  record: string;
  level: number;
}

export interface HandBuild {
  hand: string;
  level: number;
  /** The save's own class tag, e.g. `tagSkillClassName0106`. */
  classTag: string;
  /** Bought ranks, mastery bars included. */
  allocated: SkillRank[];
  /** Ranks the worn gear adds to one named skill. */
  granted: SkillRank[];
  /** Ranks the worn gear adds to every skill in one mastery, by bar record. */
  masteryGranted: SkillRank[];
  /** Ranks the worn gear adds to every skill, full stop. */
  allGranted: number;
}

export interface TradesSheet {
  trees: MasteryTree[];
  builds: HandBuild[];
}

/** What one node reads as for one hand — the game's own arithmetic. */
export interface NodeStanding {
  /** Ranks the skill points bought. */
  bought: number;
  /** Ranks the worn gear grafts on top — zero until a point is spent, the
   * way the game withholds them from a skill nobody has learned. */
  granted: number;
  /** bought + granted, held under the ultimate ceiling. */
  total: number;
  /** True while the mastery bar sits below the column's unlock level. */
  locked: boolean;
}
