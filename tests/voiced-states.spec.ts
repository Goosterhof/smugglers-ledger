// The six 4D voiced states, each asserted against its TRIGGER CONDITION —
// the error voice is gated, not decorative (Phase 4 criterion, 6 sites).
//
// The composable holds module-level state, so every scenario re-imports a
// fresh module graph via vi.resetModules() and steers the mocked bridge.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { invokeMock } from "./setup";

type Steer = {
  characters?: unknown[];
  stash?: { flagged: string | null; tabs: unknown[] };
  overview?: Record<string, unknown>;
  hits?: unknown[];
  reject?: boolean;
  holdFirstFetch?: boolean;
};

const READY_OVERVIEW = {
  chosenRoot: "/roots/save",
  roots: [],
  lastTurnedEpochMs: 1_755_300_000_000,
  codexNote: null,
  codexCold: false,
};

function answer(steer: Steer, command: unknown): Promise<unknown> {
  switch (command) {
    case "list_characters":
      return Promise.resolve(steer.characters ?? []);
    case "list_stash":
      return Promise.resolve(steer.stash ?? { flagged: null, tabs: [] });
    case "ledger_overview":
      return Promise.resolve(steer.overview ?? READY_OVERVIEW);
    case "search_ledger":
      return Promise.resolve(steer.hits ?? []);
    default:
      return Promise.reject(new Error(`unmocked command ${String(command)}`));
  }
}

function steerBridge(steer: Steer): void {
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: unknown) => {
    if (steer.reject === true) {
      return Promise.reject(new Error("no hoard found on this machine"));
    }
    if (steer.holdFirstFetch === true) {
      // A promise that never resolves inside the spec — the state BEFORE the
      // first turn lands.
      return new Promise(() => undefined);
    }
    return answer(steer, command);
  });
}

async function mountApp(): Promise<VueWrapper> {
  const { default: App } = await import("@/App.vue");
  return mount(App);
}

function shelfStates(wrapper: VueWrapper): string[] {
  return wrapper
    .findAll("[data-testid='empty-shelf']")
    .map((node) => node.attributes("data-state") ?? "");
}

let wrapper: VueWrapper | undefined;

beforeEach(() => {
  vi.resetModules();
});

afterEach(() => {
  // happy-dom shares the window across specs — unmount so the composable's
  // keyboard listeners do not leak between scenarios.
  wrapper?.unmount();
  wrapper = undefined;
});

describe("the six voiced states (4D), gated against their triggers", () => {
  it("should voice the first run while the cipher is still turning", async () => {
    // Trigger: no fetch has landed yet.
    steerBridge({ holdFirstFetch: true });
    wrapper = await mountApp();
    await flushPromises();
    expect(shelfStates(wrapper)).toContain("firstRun");
    expect(wrapper.text()).toContain("Turning the cipher on the hoard…");
  });

  it("should voice the empty machine when discovery finds no hoard", async () => {
    // Trigger: the spine answers NoHoardFound.
    steerBridge({ reject: true });
    wrapper = await mountApp();
    await flushPromises();
    expect(shelfStates(wrapper)).toContain("noSaves");
    expect(wrapper.text()).toContain(
      "No hoard found on this machine — point me at a save folder and the ledger opens.",
    );
  });

  it("should voice the cold codex while the first shelf-walk runs", async () => {
    // Trigger: overview.codexCold — the spine's stage-1 store.
    steerBridge({ overview: { ...READY_OVERVIEW, codexCold: true } });
    wrapper = await mountApp();
    await flushPromises();
    expect(shelfStates(wrapper)).toContain("coldCodex");
    expect(wrapper.text()).toContain(
      "The codex is reading the shelves — first visit only, the ledger remembers.",
    );
  });

  it("should voice the missing install when the codex has no shelf", async () => {
    // Trigger: overview.codexNote — raw-path mode until pointed.
    steerBridge({
      overview: { ...READY_OVERVIEW, codexNote: "the codex has no shelf: not found" },
    });
    wrapper = await mountApp();
    await flushPromises();
    expect(shelfStates(wrapper)).toContain("noInstall");
    expect(wrapper.text()).toContain(
      "The codex has no shelf for these records yet — point me at the install.",
    );
  });

  it("should voice the empty net when a search returns nothing", async () => {
    // Trigger: non-empty query, zero hits.
    steerBridge({ hits: [] });
    wrapper = await mountApp();
    await flushPromises();
    const { useLedger } = await import("@/composables/useLedger");
    const ledger = useLedger();
    ledger.query.value = "philosopher's stone";
    await flushPromises();
    await wrapper.vm.$nextTick();
    expect(shelfStates(wrapper)).toContain("noResults");
    expect(wrapper.text()).toContain(
      "Nothing by that name in the hoard — not in any bag, tab, or slot.",
    );
  });

  it("should voice the save that will not turn as a struck, flagged hand", async () => {
    // Trigger: a flagged character (the 1C flagged-not-fatal path).
    const flagged = {
      name: "Trickster",
      level: 0,
      classTag: "",
      hardcore: false,
      iron: 0,
      flagged: "the ledger won't turn: block end mismatch",
      equipment: [],
      weaponSet1: [],
      weaponSet2: [],
      bags: [],
      personalStash: [],
    };
    steerBridge({ characters: [flagged] });
    wrapper = await mountApp();
    await flushPromises();
    const { useLedger } = await import("@/composables/useLedger");
    useLedger().openManifest("Trickster");
    await wrapper.vm.$nextTick();
    expect(shelfStates(wrapper)).toContain("wontTurn");
    expect(wrapper.text()).toContain(
      "The ledger won't turn — this save may be from a different game version.",
    );
  });
});
