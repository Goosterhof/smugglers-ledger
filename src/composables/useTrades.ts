// THE TRADES' own store: the ten trees read once from the install, every
// hand's build laid over them, and the one piece of arithmetic that matters —
// what a given skill reads as for a given hand.
//
// The trees do not move (they change when the game is patched, and the Rust
// side re-reads them then), so they load once and stay. The builds re-arrive
// with every turn of the ledger, because a save write can change them.

import { computed, ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  HandBuild,
  MasteryTree,
  NodeStanding,
  SkillNode,
  SkillRank,
  TradesSheet,
} from "@/types/trades";

const trees = shallowRef<MasteryTree[]>([]);
const builds = shallowRef<HandBuild[]>([]);
/** The trade whose tree is unfolded, by class index — null folds them all. */
const openTrade = ref<number | null>(null);
/** Whose ranks the tree wears. Null = the trade as it stands, unlearned. */
const carriedBy = ref<string | null>(null);
const query = ref("");
/** Set when the install could not be read — voiced, never a blank page. */
const shelfNote = ref<string | null>(null);
const loaded = ref(false);
let inFlight: Promise<void> | null = null;

/** A rank list as a lookup — the spine serves arrays, the panel asks by key. */
function byRecord(ranks: SkillRank[]): ReadonlyMap<string, number> {
  return new Map(ranks.map((r) => [r.record, r.level]));
}

async function fetchTrades(): Promise<void> {
  try {
    const sheet = await invoke<TradesSheet>("list_trades");
    trees.value = sheet.trees;
    builds.value = sheet.builds;
    shelfNote.value = null;
  } catch (error) {
    // The install is the only place the trees exist (RD-3): no install, no
    // trees, and the panel says so in its own voice.
    trees.value = [];
    builds.value = [];
    shelfNote.value = String(error);
  } finally {
    loaded.value = true;
  }
}

/** The chosen hand's build, or null when the tree is worn by nobody. */
const chosenBuild = computed<HandBuild | null>(
  () => builds.value.find((b) => b.hand === carriedBy.value) ?? null,
);

const allocatedRanks = computed(() =>
  chosenBuild.value === null ? new Map<string, number>() : byRecord(chosenBuild.value.allocated),
);
const grantedRanks = computed(() =>
  chosenBuild.value === null ? new Map<string, number>() : byRecord(chosenBuild.value.granted),
);
const masteryGrants = computed(() =>
  chosenBuild.value === null
    ? new Map<string, number>()
    : byRecord(chosenBuild.value.masteryGranted),
);

/**
 * What one skill reads as for the chosen hand — the game's own arithmetic:
 * gear grafts land only on a skill that has been LEARNED (a "+2 to Cadence"
 * ring does nothing for a character who never bought Cadence), mastery-wide
 * and all-skills grafts pile on the same way, and the sum is held under the
 * skill's ultimate ceiling.
 */
function standingOf(tree: MasteryTree, node: SkillNode): NodeStanding {
  const bought = allocatedRanks.value.get(node.record) ?? 0;
  const barLevel = allocatedRanks.value.get(tree.barRecord) ?? 0;
  const locked = chosenBuild.value !== null && barLevel < node.unlockLevel;
  if (bought === 0) {
    return { bought: 0, granted: 0, total: 0, locked };
  }
  const granted =
    (grantedRanks.value.get(node.record) ?? 0) +
    (masteryGrants.value.get(tree.barRecord) ?? 0) +
    (chosenBuild.value?.allGranted ?? 0);
  return {
    bought,
    granted,
    total: Math.min(bought + granted, node.ultimateLevel),
    locked,
  };
}

/** The bar's own level for the chosen hand — 0 when nobody wears the tree. */
function barLevelOf(tree: MasteryTree): number {
  return allocatedRanks.value.get(tree.barRecord) ?? 0;
}

/** Which hands carry a mastery: the ones whose save filed a level for its
 * bar. Read off the ranks, not off the class tag — the bar is the fact. */
function handsCarrying(tree: MasteryTree): string[] {
  return builds.value
    .filter((b) => b.allocated.some((r) => r.record === tree.barRecord && r.level > 0))
    .map((b) => b.hand);
}

/** One row of the search: a skill, and the trade it belongs to. */
export interface TradeHit {
  tree: MasteryTree;
  node: SkillNode;
}

/** The search runs over the trees themselves — every skill in the game, not
 * only the ones somebody learned. Matches the skill's name, its description,
 * its trade's name, and its raw record path (a skill the codex could not name
 * is still findable by the string the panel shows for it — the 4C ruling). */
const hits = computed<TradeHit[]>(() => {
  const needle = query.value.trim().toLowerCase();
  if (needle === "") return [];
  const found: TradeHit[] = [];
  for (const tree of trees.value) {
    const tradeMatches = tree.name.toLowerCase().includes(needle);
    for (const node of tree.nodes) {
      if (
        tradeMatches ||
        node.name.toLowerCase().includes(needle) ||
        node.blurb.toLowerCase().includes(needle) ||
        node.record.toLowerCase().includes(needle)
      ) {
        found.push({ tree, node });
      }
    }
  }
  found.sort(
    (a, b) =>
      a.tree.classIndex - b.tree.classIndex ||
      a.node.tier - b.node.tier ||
      a.node.name.localeCompare(b.node.name),
  );
  return found;
});

/** THE TRADES' store. The first call reads the shelves; every call shares. */
export function useTrades() {
  if (!loaded.value && inFlight === null) {
    inFlight = fetchTrades();
    // A save write can change a build (a point spent, a ring swapped) but
    // never a tree — the Rust side holds the trees for the process and hands
    // back fresh builds, so re-asking is cheap and always current. RD-5: the
    // page is fresh on next glance, and there is no refresh button here
    // either.
    void listen("ledger-turned", () => void fetchTrades());
    void listen("save-changed", () => void fetchTrades());
  }

  function openTradeAt(classIndex: number | null): void {
    openTrade.value = openTrade.value === classIndex ? null : classIndex;
  }

  /** Jump from a search row to the tree it lives in, with the search struck
   * — the answer to "where does this skill sit" is the tree, not the row. */
  function showInTree(hit: TradeHit): void {
    openTrade.value = hit.tree.classIndex;
    query.value = "";
  }

  function strike(): void {
    query.value = "";
  }

  return {
    /** The store is a module-level singleton (one page, one set of trees);
     * the component tests need it back at zero between scenarios. */
    _resetForTests(): void {
      trees.value = [];
      builds.value = [];
      openTrade.value = null;
      carriedBy.value = null;
      query.value = "";
      shelfNote.value = null;
      loaded.value = false;
      inFlight = null;
    },
    trees,
    builds,
    openTrade,
    openTradeAt,
    carriedBy,
    chosenBuild,
    query,
    hits,
    strike,
    showInTree,
    shelfNote,
    loaded,
    standingOf,
    barLevelOf,
    handsCarrying,
  };
}
