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
  tier: 0,
  stack: 12,
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

  it("should strike a flagged entry in wax without deleting it", () => {
    wrapper = mount(EntryRow, { props: { hit: NAMED, struck: true } });
    const row = wrapper.get("[data-testid='entry-row']");
    expect(row.classes().join(" ")).toContain("line-through");
    expect(wrapper.text()).toContain("Ectoplasm");
  });
});
