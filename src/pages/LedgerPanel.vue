<script setup lang="ts">
// THE HOARD — cross-everything search. The index rule at the head, THE
// DOCKET's cuts beneath it, the ruled entries ordered by THE FOOTING (the
// clickable column heads), the footed totals (B5: the count is footed at the
// bottom of the column, the way a bookkeeper totals — never a toolbar badge).
// When cuts are active the foot names both figures: what the cut shows OF what
// the query found.
import { computed } from "vue";
import IndexRule from "@/components/IndexRule.vue";
import TheDocket from "@/components/TheDocket.vue";
import EntryRow from "@/components/EntryRow.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import { useLedger } from "@/composables/useLedger";

const { hits, sortedHits, sortKey, sortDir, sortBy, docketCut, state, wetKeys, rowKey } =
  useLedger();

type SortKey = "item" | "rarity" | "count" | "where";
const COLUMNS: { key: SortKey; label: string; align: string }[] = [
  { key: "item", label: "Item", align: "" },
  { key: "rarity", label: "Rarity", align: "" },
  { key: "count", label: "Count", align: "text-right pr-[12px]" },
  { key: "where", label: "Whereabouts", align: "" },
];

// The footing caret: ▴ ascending, ▾ descending, in ember, on the active column.
function caret(key: SortKey): string {
  return sortKey.value === key ? (sortDir.value === "asc" ? "▴" : "▾") : "";
}

const foot = computed(() => {
  const shown = sortedHits.value;
  const entries = shown.length;
  const inHand = shown.reduce((sum, h) => sum + h.stack, 0);
  const hands = new Set(shown.map((h) => h.hand)).size;
  const base = `${entries} ENTRIES · ${inHand} IN HAND · ${hands} HANDS`;
  return docketCut.value ? `${base} — CUT FROM ${hits.value.length}` : base;
});
</script>

<template>
  <div>
    <IndexRule />
    <TheDocket />

    <EmptyShelf v-if="state === 'noResults'" state="noResults" />
    <template v-else>
      <div
        class="grid grid-cols-[minmax(160px,1fr)_112px_72px_minmax(200px,1.2fr)] px-[8px] bg-sl-folio-shade border-b border-sl-rule-strong"
      >
        <button
          v-for="col in COLUMNS"
          :key="col.key"
          class="sl-column-head text-left focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
          :class="[
            col.align,
            // Active head: brightened ash word + a drawn 2px ichor subtotal
            // rule beneath (ichor as a RULE, never inked text) + an ember caret.
            sortKey === col.key
              ? 'text-sl-ink border-b-2 border-sl-lamp'
              : 'text-sl-ink-soft hover:text-sl-ink',
          ]"
          :aria-sort="
            sortKey === col.key ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'
          "
          :data-testid="`sort-${col.key}`"
          @click="sortBy(col.key)"
        >
          {{ col.label
          }}<span v-if="caret(col.key)" class="text-sl-ember ml-[3px]" aria-hidden="true">{{
            caret(col.key)
          }}</span>
        </button>
      </div>

      <EntryRow
        v-for="(hit, i) in sortedHits"
        :key="rowKey(hit)"
        :hit="hit"
        :wet="wetKeys.has(rowKey(hit))"
        :banded="(i + 1) % 5 === 0"
      />

      <p
        class="sl-foot text-sl-ink-soft border-t border-sl-rule-strong mt-[-1px]"
        data-testid="foot"
      >
        {{ foot }}
      </p>
    </template>
  </div>
</template>
