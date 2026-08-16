<script setup lang="ts">
// THE HOARD — cross-everything search. The index rule at the head, the ruled
// entries, the footed totals (B5: the count is footed at the bottom of the
// column, the way a bookkeeper totals — never a toolbar badge).
import { computed } from "vue";
import IndexRule from "@/components/IndexRule.vue";
import EntryRow from "@/components/EntryRow.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import { useLedger } from "@/composables/useLedger";

const { hits, state, wetKeys, rowKey } = useLedger();

const foot = computed(() => {
  const entries = hits.value.length;
  const inHand = hits.value.reduce((sum, h) => sum + h.stack, 0);
  const hands = new Set(hits.value.map((h) => h.location.split(" — ")[0])).size;
  return `${entries} ENTRIES · ${inHand} IN HAND · ${hands} HANDS`;
});
</script>

<template>
  <div>
    <IndexRule />

    <EmptyShelf v-if="state === 'noResults'" state="noResults" />
    <template v-else>
      <div
        class="grid grid-cols-[minmax(160px,1fr)_44px_72px_minmax(200px,1.2fr)] px-[8px] bg-sl-folio-shade border-b border-sl-rule-strong"
      >
        <span class="sl-column-head text-sl-ink-soft">Item</span>
        <span class="sl-column-head text-sl-ink-soft">Mark</span>
        <span class="sl-column-head text-sl-ink-soft text-right pr-[12px]">Count</span>
        <span class="sl-column-head text-sl-ink">Whereabouts</span>
      </div>

      <EntryRow
        v-for="(hit, i) in hits"
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
