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

  it("should strike a flagged entry in wax without deleting it", () => {
    wrapper = mount(EntryRow, { props: { hit: NAMED, struck: true } });
    const row = wrapper.get("[data-testid='entry-row']");
    expect(row.classes().join(" ")).toContain("line-through");
    expect(wrapper.text()).toContain("Ectoplasm");
  });
});
