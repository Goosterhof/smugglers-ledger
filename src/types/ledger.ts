// The shapes the Rust spine serves (ledger.rs projections), mirrored for the
// panels. Field names follow the serde camelCase renames.

export interface NamedContraband {
  /** Resolved display name — null means contraband still in the crate: the
   * UI renders the raw record path, and search still finds it (4C). */
  name: string | null;
  recordPath: string;
  prefix: string | null;
  suffix: string | null;
  /** Ordinal rarity tier 0–5 — the WORD carries it, the game-hue chip seconds it. */
  tier: number;
  slotClass: string | null;
  /** Resolved component (e.g. "Ectoplasm") — null when the socket is empty. */
  component: string | null;
  augment: string | null;
  seed: number;
  /** Aggregated stat lines (base + affixes + component + augment). */
  stats: StatLine[];
  stack: number;
  x: number;
  y: number;
}

/** One readable stat line, two-tone: ember magnitude, ash label. */
export interface StatLine {
  magnitude: string;
  label: string;
}

/** The six rarity words, in tier order — the game's own ladder. */
export const RARITY_NAMES = ["Common", "Magical", "Rare", "Epic", "Legendary", "Quest"] as const;

/** The docket's place filters — the structured `place` values the spine serves. */
export const PLACE_NAMES = [
  "EQUIPPED",
  "BAGS",
  "WEAPON SET",
  "PERSONAL STASH",
  "SHARED STASH",
] as const;

export interface NamedTab {
  /** The tab's OWN parsed grid geometry — never an assumed fixed size. */
  width: number;
  height: number;
  items: NamedContraband[];
}

export interface CharacterSheet {
  name: string;
  level: number;
  classTag: string;
  hardcore: boolean;
  iron: number;
  /** The voiced 4D flag when this save's cipher would not turn. */
  flagged: string | null;
  /** Twelve slots in the game's own order; null is an empty ruled box. */
  equipment: (NamedContraband | null)[];
  weaponSet1: (NamedContraband | null)[];
  weaponSet2: (NamedContraband | null)[];
  bags: NamedContraband[][];
  personalStash: NamedTab[];
}

export interface WarehouseSheet {
  flagged: string | null;
  tabs: NamedTab[];
}

export interface LedgerHit {
  name: string | null;
  recordPath: string;
  prefix: string | null;
  suffix: string | null;
  tier: number;
  slotClass: string | null;
  component: string | null;
  augment: string | null;
  seed: number;
  stats: StatLine[];
  stack: number;
  /** The owner — a character name in caps, or "SHARED STASH". */
  hand: string;
  /** Container class — one of PLACE_NAMES. */
  place: string;
  /** Non-null for 100% of results — the gated contract. */
  location: string;
}

export interface CandidateRoot {
  path: string;
  saveCount: number;
  freshestEpochMs: number;
}

export interface LedgerOverview {
  chosenRoot: string | null;
  roots: CandidateRoot[];
  lastTurnedEpochMs: number;
  codexNote: string | null;
  /** True while the first-run shelf-walk is still reading — 4D's cold state. */
  codexCold: boolean;
}

/** The game's own equipment slot order — the manifest's paper doll. */
export const EQUIPMENT_SLOT_NAMES = [
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
] as const;

export type PanelName = "hoard" | "manifest" | "warehouse";
