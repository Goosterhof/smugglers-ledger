// EntryRow contracts: the unresolved fallback renders the raw record path
// (visible AND searchable by the same string — log 4C), the location column
// is always present, and the wet state carries the arrival treatment.

import { afterEach, describe, expect, it } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import EntryRow from "@/components/EntryRow.vue";

let wrapper: VueWrapper | undefined;

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
});

const NAMED = {
  name: "Ectoplasm",
  recordPath: "records/items/materia/compa_ectoplasm.dbr",
  prefix: null,
  suffix: null,
  tier: 0,
  slotClass: null,
  component: null,
  augment: null,
  seed: 277104776,
  stats: [],
  skills: [],
  bitmap: null,
  stack: 12,
  hand: "SPINNY",
  place: "BAGS",
  location: "SPINNY — BAGS, BAG 1, CELL 2,3",
};

describe("EntryRow", () => {
  it("should render the resolved name with its location at full ink", () => {
    wrapper = mount(EntryRow, { props: { hit: NAMED } });
    expect(wrapper.text()).toContain("Ectoplasm");
    expect(wrapper.text()).toContain("SPINNY — BAGS, BAG 1, CELL 2,3");
  });

  it("should fall back to the raw record path when the codex has no name", () => {
    wrapper = mount(EntryRow, {
      props: { hit: { ...NAMED, name: null } },
    });
    expect(wrapper.text()).toContain("records/items/materia/compa_ectoplasm.dbr");
  });

  it("should mount a wet arrival at double pitch and settle markers", () => {
    wrapper = mount(EntryRow, { props: { hit: NAMED, wet: true } });
    const row = wrapper.get("[data-testid='entry-row']");
    expect(row.attributes("data-wet")).toBe("true");
    expect(row.classes().join(" ")).toContain("h-[52px]");
  });

  it("should stamp the rarity WORD in the game's hue family", () => {
    wrapper = mount(EntryRow, { props: { hit: { ...NAMED, tier: 4 } } });
    const stamp = wrapper.get("[data-testid='rarity-stamp']");
    expect(stamp.text()).toContain("Legendary");
    expect(stamp.attributes("aria-label")).toBe("Legendary");
  });

  it("should unfold THE DOCKET on click with the entry's full account", async () => {
    wrapper = mount(EntryRow, {
      props: {
        hit: {
          ...NAMED,
          prefix: "Thunderstruck",
          suffix: "of Attack",
          component: "Ectoplasm",
          slotClass: "WeaponMelee_Sword2h",
          tier: 2,
        },
      },
    });
    expect(wrapper.find("[data-testid='docket']").exists()).toBe(false);
    await wrapper.get("[data-testid='entry-row']").trigger("click");
    const docket = wrapper.get("[data-testid='docket']");
    expect(docket.text()).toContain("Thunderstruck Ectoplasm of Attack");
    expect(docket.text()).toContain("Rare");
    expect(docket.text()).toContain("Sword 2h");
    expect(docket.text()).toContain("seed 277104776");
    expect(docket.text()).toContain("records/items/materia/compa_ectoplasm.dbr");
    await wrapper.get("[data-testid='entry-row']").trigger("click");
    expect(wrapper.find("[data-testid='docket']").exists()).toBe(false);
  });

  it("should render the two-tone stat block in the docket when stats are present", async () => {
    wrapper = mount(EntryRow, {
      props: {
        hit: {
          ...NAMED,
          stats: [
            { magnitude: "+20%", label: "Lightning Damage" },
            { magnitude: "+35", label: "Armor" },
          ],
        },
      },
    });
    await wrapper.get("[data-testid='entry-row']").trigger("click");
    const block = wrapper.get("[data-testid='stat-block']");
    expect(block.text()).toContain("+20%");
    expect(block.text()).toContain("Lightning Damage");
    expect(block.text()).toContain("+35");
    expect(block.text()).toContain("Armor");
  });

  it("should rule the skill grafts apart from the stats in the docket", async () => {
    wrapper = mount(EntryRow, {
      props: {
        hit: {
          ...NAMED,
          skills: [
            { magnitude: "+2", label: "to Blade Arc" },
            { magnitude: "+70%", label: "Fire Damage to Savagery" },
          ],
        },
      },
    });
    await wrapper.get("[data-testid='entry-row']").trigger("click");
    const block = wrapper.get("[data-testid='skill-block']");
    expect(block.text()).toContain("+2");
    expect(block.text()).toContain("to Blade Arc");
    expect(block.text()).toContain("+70%");
    expect(block.text()).toContain("Fire Damage to Savagery");
    // No stats on this hit — the graft block stands alone, never merged in.
    expect(wrapper.find("[data-testid='stat-block']").exists()).toBe(false);
  });

  it("should strike a flagged entry in wax without deleting it", () => {
    wrapper = mount(EntryRow, { props: { hit: NAMED, struck: true } });
    const row = wrapper.get("[data-testid='entry-row']");
    expect(row.classes().join(" ")).toContain("line-through");
    expect(wrapper.text()).toContain("Ectoplasm");
  });
});
