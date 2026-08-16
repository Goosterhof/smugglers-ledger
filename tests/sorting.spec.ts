// THE FOOTING contracts: the hoard orders on the clicked column, a second
// click on the same column flips direction, and the order is stable (ties
// fall back to the record path so the same hoard always lays out the same).

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { useLedger } from "@/composables/useLedger";

function hit(over: Record<string, unknown>) {
  return {
    name: "Item",
    recordPath: "records/a.dbr",
    prefix: null,
    suffix: null,
    tier: 0,
    slotClass: null,
    component: null,
    augment: null,
    seed: 0,
    stack: 1,
    hand: "SPINNY",
    place: "BAGS",
    location: "SPINNY — BAGS",
    ...over,
  };
}

describe("THE FOOTING (sorting)", () => {
  let ledger: ReturnType<typeof useLedger>;

  beforeEach(async () => {
    ledger = useLedger();
    // The store's one-time boot fetch runs async on first use and clobbers
    // hits; let it settle before seeding, so the seed is what the test sorts.
    await flushPromises();
    ledger.hits.value = [
      hit({ name: "Zed", recordPath: "records/z.dbr", tier: 4, stack: 1, location: "C — BAGS" }),
      hit({ name: "Alpha", recordPath: "records/a.dbr", tier: 1, stack: 9, location: "A — BAGS" }),
      hit({ name: "Mid", recordPath: "records/m.dbr", tier: 2, stack: 5, location: "B — BAGS" }),
    ];
  });

  afterEach(() => {
    ledger.hits.value = [];
    ledger.sortBy("where"); // reset to default asc via a known state
    if (ledger.sortDir.value === "desc") ledger.sortBy("where");
  });

  it("should order by item name ascending", () => {
    ledger.sortBy("item");
    expect(ledger.sortedHits.value.map((h) => h.name)).toStrictEqual(["Alpha", "Mid", "Zed"]);
  });

  it("should flip direction on a second click of the same column", () => {
    ledger.sortBy("count"); // figures default to desc
    expect(ledger.sortedHits.value.map((h) => h.stack)).toStrictEqual([9, 5, 1]);
    ledger.sortBy("count");
    expect(ledger.sortedHits.value.map((h) => h.stack)).toStrictEqual([1, 5, 9]);
  });

  it("should order by rarity tier, rarest first by default", () => {
    ledger.sortBy("rarity");
    expect(ledger.sortedHits.value.map((h) => h.tier)).toStrictEqual([4, 2, 1]);
  });

  it("should break ties on the record path for a stable order", () => {
    ledger.hits.value = [
      hit({ name: "Same", recordPath: "records/b.dbr", tier: 2 }),
      hit({ name: "Same", recordPath: "records/a.dbr", tier: 2 }),
    ];
    ledger.sortBy("rarity");
    expect(ledger.sortedHits.value.map((h) => h.recordPath)).toStrictEqual([
      "records/a.dbr",
      "records/b.dbr",
    ]);
  });
});
