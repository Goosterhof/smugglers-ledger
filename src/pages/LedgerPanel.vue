<script setup lang="ts">
// THE HOARD — cross-everything search. The index rule at the head, THE
// DOCKET's cuts beneath it, the ruled entries, the footed totals (B5: the
// count is footed at the bottom of the column, the way a bookkeeper totals —
// never a toolbar badge). When cuts are active the foot names both figures:
// what the cut shows OF what the query found.
import { computed } from "vue";
import IndexRule from "@/components/IndexRule.vue";
import TheDocket from "@/components/TheDocket.vue";
import EntryRow from "@/components/EntryRow.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import { useLedger } from "@/composables/useLedger";

const { hits, filteredHits, docketCut, state, wetKeys, rowKey } = useLedger();

const foot = computed(() => {
  const shown = filteredHits.value;
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
        <span class="sl-column-head text-sl-ink-soft">Item</span>
        <span class="sl-column-head text-sl-ink-soft">Rarity</span>
        <span class="sl-column-head text-sl-ink-soft text-right pr-[12px]">Count</span>
        <span class="sl-column-head text-sl-ink">Whereabouts</span>
      </div>

      <EntryRow
        v-for="(hit, i) in filteredHits"
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
