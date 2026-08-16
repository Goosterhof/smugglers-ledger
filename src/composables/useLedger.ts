// The reactive store every panel reads (4A) — one place that invokes the
// spine's commands, holds the resolved result set, tracks which entries just
// arrived (THE WET INK's trigger), and owns the six 4D states.

import { computed, ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CharacterSheet,
  LedgerHit,
  LedgerOverview,
  PanelName,
  WarehouseSheet,
} from "@/types/ledger";

/** The six voiced states (4D) — owned here, gated in the component tests. */
export type ShelfState =
  | "noSaves"
  | "noInstall"
  | "wontTurn"
  | "noResults"
  | "firstRun"
  | "coldCodex";

const SEARCH_DEBOUNCE_MS = 180;

const characters = shallowRef<CharacterSheet[]>([]);
const stash = shallowRef<WarehouseSheet>({ flagged: null, tabs: [] });
const overview = shallowRef<LedgerOverview>({
  chosenRoot: null,
  roots: [],
  lastTurnedEpochMs: 0,
  codexNote: null,
  codexCold: false,
});
const query = ref("");
const hits = shallowRef<LedgerHit[]>([]);
/** THE DOCKET's standing filters — empty sets mean "no cut on this axis". */
const tierCut = ref<ReadonlySet<number>>(new Set());
const placeCut = ref<ReadonlySet<string>>(new Set());
const handCut = ref<string | null>(null);
const panel = ref<PanelName>("hoard");
const chosenHand = ref<string | null>(null);
/** Row keys that arrived or changed on the last turn — the wet ink set. */
const wetKeys = shallowRef<ReadonlySet<string>>(new Set());
const firstTurnLanded = ref(false);
const noHoard = ref(false);
let searchTimer: ReturnType<typeof setTimeout> | undefined;
let listening = false;

/** THE DOCKET's view: the query's hits with the standing cuts applied. */
const filteredHits = computed<LedgerHit[]>(() => {
  const tiers = tierCut.value;
  const places = placeCut.value;
  const hand = handCut.value;
  if (tiers.size === 0 && places.size === 0 && hand === null) return hits.value;
  return hits.value.filter(
    (h) =>
      (tiers.size === 0 || tiers.has(h.tier)) &&
      (places.size === 0 || places.has(h.place)) &&
      (hand === null || h.hand === hand),
  );
});

const docketCut = computed(
  () => tierCut.value.size > 0 || placeCut.value.size > 0 || handCut.value !== null,
);

function toggleTier(tier: number): void {
  const next = new Set(tierCut.value);
  if (next.has(tier)) next.delete(tier);
  else next.add(tier);
  tierCut.value = next;
}

function togglePlace(place: string): void {
  const next = new Set(placeCut.value);
  if (next.has(place)) next.delete(place);
  else next.add(place);
  placeCut.value = next;
}

function cutHand(hand: string | null): void {
  handCut.value = hand;
}

function liftCuts(): void {
  tierCut.value = new Set();
  placeCut.value = new Set();
  handCut.value = null;
}

function rowKey(hit: LedgerHit): string {
  return `${hit.recordPath}|${hit.location}|${hit.stack}`;
}

async function fetchAll(markWet: boolean): Promise<void> {
  try {
    const previous = new Set(hits.value.map(rowKey));
    const [nextCharacters, nextStash, nextOverview] = await Promise.all([
      invoke<CharacterSheet[]>("list_characters"),
      invoke<WarehouseSheet>("list_stash"),
      invoke<LedgerOverview>("ledger_overview"),
    ]);
    characters.value = nextCharacters;
    stash.value = nextStash;
    overview.value = nextOverview;
    noHoard.value = false;
    firstTurnLanded.value = true;
    await runSearch();
    if (markWet) {
      const fresh = new Set<string>();
      for (const hit of hits.value) {
        const key = rowKey(hit);
        if (!previous.has(key)) {
          fresh.add(key);
        }
      }
      wetKeys.value = fresh;
    } else {
      wetKeys.value = new Set();
    }
  } catch {
    // The spine answered NoHoardFound (or has no state yet): the manual
    // picker's front door, voiced — never a blank page.
    noHoard.value = true;
    firstTurnLanded.value = true;
  }
}

/** With no query, THE HOARD shows the full hoard ("STRIKE → the full hoard
 * returns"): every base record path starts with `records/`, so that prefix
 * is the match-everything net — the backend stays the single authority for
 * location strings. */
const FULL_HOARD_NET = "records/";

async function runSearch(): Promise<void> {
  const q = query.value.trim();
  try {
    hits.value = await invoke<LedgerHit[]>("search_ledger", {
      query: q === "" ? FULL_HOARD_NET : q,
    });
  } catch {
    hits.value = [];
  }
}

/** The store. First call arms the event listeners; every call shares state. */
export function useLedger() {
  if (!listening) {
    listening = true;
    void listen("ledger-turned", () => void fetchAll(true));
    void listen("save-changed", () => void fetchAll(true));
    void fetchAll(false);
  }

  const state = computed<ShelfState | null>(() => {
    if (!firstTurnLanded.value) return "firstRun";
    if (noHoard.value) return "noSaves";
    if (overview.value.codexCold) return "coldCodex";
    if (overview.value.codexNote !== null) return "noInstall";
    if ((query.value.trim() !== "" || docketCut.value) && filteredHits.value.length === 0)
      return "noResults";
    return null;
  });

  function search(next: string): void {
    query.value = next;
    if (searchTimer !== undefined) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => void runSearch(), SEARCH_DEBOUNCE_MS);
  }

  function strike(): void {
    query.value = "";
    hits.value = [];
  }

  async function switchRoot(path: string): Promise<void> {
    firstTurnLanded.value = false;
    try {
      await invoke("switch_root", { path });
    } catch {
      noHoard.value = true;
      firstTurnLanded.value = true;
    }
  }

  function openManifest(hand: string): void {
    chosenHand.value = hand;
    panel.value = "manifest";
  }

  return {
    characters,
    stash,
    overview,
    query,
    hits,
    filteredHits,
    tierCut,
    placeCut,
    handCut,
    docketCut,
    toggleTier,
    togglePlace,
    cutHand,
    liftCuts,
    panel,
    chosenHand,
    wetKeys,
    state,
    rowKey,
    search,
    strike,
    switchRoot,
    openManifest,
  };
}
