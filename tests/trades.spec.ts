// THE TRADES contracts: the tree is drawn at the game's own coordinates with
// a line for every connection the install itself lists, a worn graft is
// withheld from a skill nobody learned (the game's own rule), the rollover
// names the tier and what it costs the bar, and the search reaches every
// skill in every trade — not only the ones somebody bought.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import { nextTick } from "vue";
import SkillTree from "@/components/SkillTree.vue";
import SkillRollover from "@/components/SkillRollover.vue";
import TradesPanel from "@/pages/TradesPanel.vue";
import { useTrades } from "@/composables/useTrades";
import type { HandBuild, MasteryTree, SkillNode } from "@/types/trades";
import { invokeMock } from "./setup";

let wrapper: VueWrapper | undefined;

const SOLDIER_BAR = "records/skills/playerclass01/_classtraining_class01.dbr";
const CADENCE = "records/skills/playerclass01/cadence1.dbr";
const DISCORD = "records/skills/playerclass01/cadence1b.dbr";
const FIGHTING_FORM = "records/skills/playerclass01/cadence2.dbr";

function node(overrides: Partial<SkillNode> & { record: string; x: number }): SkillNode {
  return {
    name: "A Skill",
    blurb: "",
    tier: 1,
    unlockLevel: 1,
    maxLevel: 16,
    ultimateLevel: 26,
    kind: "active",
    y: 319,
    circular: false,
    icon: null,
    parent: null,
    conversion: null,
    ...overrides,
  };
}

/** Soldier's Cadence row, at the game's own panel coordinates. */
const SOLDIER: MasteryTree = {
  classIndex: 1,
  name: "Soldier",
  blurb: "Soldiers of the imperial army.",
  barRecord: SOLDIER_BAR,
  barMaxLevel: 50,
  barIcon: null,
  tierUnlocks: [1, 5, 10, 15, 20, 25, 32, 40, 50],
  nodes: [
    node({ record: CADENCE, name: "Cadence", x: 246, tier: 1, unlockLevel: 1 }),
    node({
      record: DISCORD,
      name: "Discord",
      x: 326,
      y: 281,
      tier: 2,
      unlockLevel: 5,
      circular: true,
      kind: "transmuter",
      parent: CADENCE,
      conversion: "33–100% Physical → Elemental",
      maxLevel: 3,
      ultimateLevel: 3,
    }),
    node({
      record: FIGHTING_FORM,
      name: "Fighting Form",
      x: 486,
      tier: 4,
      unlockLevel: 15,
      circular: true,
      kind: "modifier",
      parent: CADENCE,
      maxLevel: 12,
      ultimateLevel: 22,
    }),
  ],
};

const CADENCE_UPHEAVEL: HandBuild = {
  hand: "CadenceUpheavel",
  level: 42,
  classTag: "tagSkillClassName0106",
  allocated: [
    { record: SOLDIER_BAR, level: 50 },
    { record: CADENCE, level: 1 },
  ],
  // A ring granting ranks to a skill this hand never bought.
  granted: [
    { record: CADENCE, level: 2 },
    { record: FIGHTING_FORM, level: 3 },
  ],
  masteryGranted: [{ record: SOLDIER_BAR, level: 1 }],
  allGranted: 0,
};

async function settle(): Promise<void> {
  await Promise.resolve();
  await nextTick();
  await nextTick();
}

beforeEach(() => {
  useTrades()._resetForTests();
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: unknown) => {
    if (command === "list_trades") {
      return Promise.resolve({ trees: [SOLDIER], builds: [CADENCE_UPHEAVEL] });
    }
    if (command === "list_characters") {
      return Promise.resolve([{ name: "CadenceUpheavel" }]);
    }
    return Promise.resolve(null);
  });
});

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
});

describe("the tree", () => {
  it("should draw one line for every connection the install itself lists", () => {
    wrapper = mount(SkillTree, { props: { tree: SOLDIER } });
    // Two children hang off Cadence; the root hangs off nothing.
    expect(wrapper.findAll("line")).toHaveLength(2);
    expect(wrapper.findAll("[data-testid^='skill-node-']")).toHaveLength(3);
  });

  it("should place each skill in the column the game places it in", () => {
    wrapper = mount(SkillTree, { props: { tree: SOLDIER } });
    const cadence = wrapper.get(`[data-testid='skill-node-${CADENCE}']`);
    const fightingForm = wrapper.get(`[data-testid='skill-node-${FIGHTING_FORM}']`);
    // Tier 1 is the origin; tier 4 stands three columns to its right.
    expect(cadence.attributes("style")).toContain("left: 0px");
    expect(fightingForm.attributes("style")).toContain("left: 238px");
  });

  it("should letter the rule with the mastery level each column opens at", () => {
    wrapper = mount(SkillTree, { props: { tree: SOLDIER } });
    const rule = wrapper.get("[data-testid='tier-rule']").text();
    expect(rule).toContain("15");
    expect(rule).toContain("50");
  });

  it("should show no ranks at all while nobody carries the trade", () => {
    wrapper = mount(SkillTree, { props: { tree: SOLDIER } });
    expect(wrapper.find("[data-testid^='skill-rank-']").exists()).toBe(false);
    expect(wrapper.get("[data-testid='bar-level']").text()).toBe("0 / 50");
  });

  it("should withhold a worn graft from a skill the hand never learned", async () => {
    const { carriedBy } = useTrades();
    mount(TradesPanel);
    await settle();
    carriedBy.value = "CadenceUpheavel";
    wrapper = mount(SkillTree, { props: { tree: SOLDIER } });
    // Cadence was bought (1) and the gear adds 2 for the skill + 1 for the
    // mastery: 4. Fighting Form carries a +3 ring and no point spent — the
    // game grants it nothing, and neither does the Ledger.
    expect(wrapper.get(`[data-testid='skill-rank-${CADENCE}']`).text()).toBe("4");
    expect(wrapper.find(`[data-testid='skill-rank-${FIGHTING_FORM}']`).exists()).toBe(false);
    expect(wrapper.get("[data-testid='bar-level']").text()).toBe("50 / 50");
  });
});

describe("the rollover", () => {
  it("should name the tier, the level it opens at, and what the gear adds", () => {
    wrapper = mount(SkillRollover, {
      props: {
        node: SOLDIER.nodes[0],
        tree: SOLDIER,
        standing: { bought: 1, granted: 3, total: 4, locked: false },
        worn: "CadenceUpheavel",
        barLevel: 50,
      },
    });
    const text = wrapper.text();
    expect(text).toContain("Cadence");
    expect(text).toContain("Tier 1");
    expect(text).toContain("Opens at mastery");
    expect(wrapper.get("[data-testid='rollover-rank']").text()).toBe("1 / 16 · +3 worn → 4 / 26");
  });

  it("should say the column is shut, and what the bar stands at, when it is", () => {
    wrapper = mount(SkillRollover, {
      props: {
        node: SOLDIER.nodes[2],
        tree: SOLDIER,
        standing: { bought: 0, granted: 0, total: 0, locked: true },
        worn: "MirrorOfErectus",
        barLevel: 1,
      },
    });
    expect(wrapper.get("[data-testid='rollover-rank']").text()).toContain("the column is shut");
    expect(wrapper.get("[data-testid='rollover-locked']").text()).toContain("stands at 1");
  });

  it("should spell a transmuter's conversion the way the game spells it", () => {
    wrapper = mount(SkillRollover, {
      props: {
        node: SOLDIER.nodes[1],
        tree: SOLDIER,
        standing: { bought: 0, granted: 0, total: 0, locked: false },
        worn: null,
        barLevel: 0,
      },
    });
    expect(wrapper.text()).toContain("33–100% Physical → Elemental");
  });
});

describe("the trades index", () => {
  it("should foot the trades and name every hand that carries one", async () => {
    wrapper = mount(TradesPanel);
    await settle();
    expect(wrapper.get("[data-testid='trades-foot']").text()).toBe("1 TRADE · 3 SKILLS");
    expect(wrapper.get("[data-testid='trade-row-Soldier']").text()).toContain("CadenceUpheavel");
  });

  it("should unfold a trade's own tree in place", async () => {
    wrapper = mount(TradesPanel);
    await settle();
    expect(wrapper.find("[data-testid='skill-tree']").exists()).toBe(false);
    await wrapper.get("[data-testid='trade-row-Soldier']").trigger("click");
    expect(wrapper.find("[data-testid='skill-tree']").exists()).toBe(true);
  });

  it("should reach every skill in the game, learned or not", async () => {
    wrapper = mount(TradesPanel);
    await settle();
    await wrapper.get("[data-testid='trades-rule']").setValue("form");
    await nextTick();
    expect(wrapper.find(`[data-testid='trade-hit-${FIGHTING_FORM}']`).exists()).toBe(true);
    // One trade is a TRADE — a ledger foots its columns in English.
    expect(wrapper.get("[data-testid='hits-foot']").text()).toBe("1 SKILL ACROSS 1 TRADE");
  });

  it("should answer a search row with the hand's own standing when one is worn", async () => {
    wrapper = mount(TradesPanel);
    await settle();
    useTrades().carriedBy.value = "CadenceUpheavel";
    await wrapper.get("[data-testid='trades-rule']").setValue("cadence");
    await nextTick();
    expect(wrapper.get(`[data-testid='hit-ranks-${CADENCE}']`).text()).toBe("4 / 26");
    expect(wrapper.get(`[data-testid='hit-ranks-${FIGHTING_FORM}']`).text()).toBe("not learned");
  });

  it("should send a search row to its own tree, with the search struck", async () => {
    wrapper = mount(TradesPanel);
    await settle();
    await wrapper.get("[data-testid='trades-rule']").setValue("discord");
    await nextTick();
    await wrapper.get(`[data-testid='trade-hit-${DISCORD}']`).trigger("click");
    await nextTick();
    expect(wrapper.find("[data-testid='skill-tree']").exists()).toBe(true);
    expect(useTrades().query.value).toBe("");
  });

  it("should say so in its own voice when the install cannot be reached", async () => {
    invokeMock.mockImplementation((command: unknown) => {
      if (command === "list_trades") return Promise.reject(new Error("no shelf"));
      return Promise.resolve([]);
    });
    wrapper = mount(TradesPanel);
    await settle();
    expect(wrapper.get("[data-testid='trades-note']").text()).toContain("Point me at Grim Dawn");
  });
});
