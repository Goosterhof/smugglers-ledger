<script setup lang="ts">
// THE WAREHOUSE — the shared hoard, tab by tab. A hairline pegboard at each
// tab's OWN parsed width × height (log 1C/4C: never an assumed fixed size),
// items at their true x/y, empty cells blank paper. The readout is a FIXED
// region on the sheet — a margin note, never a floating tooltip card.
import { computed, ref, watch } from "vue";
import StashCell from "@/components/StashCell.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import { useLedger } from "@/composables/useLedger";
import type { NamedContraband } from "@/types/ledger";

const { stash } = useLedger();

const tabIndex = ref(0);
watch(
  () => stash.value.tabs.length,
  (count) => {
    if (tabIndex.value >= count) tabIndex.value = 0;
  },
);

const tab = computed(() => stash.value.tabs[tabIndex.value] ?? null);

/** cells[y][x] — the grid at the tab's own geometry. */
const cells = computed<(NamedContraband | null)[][]>(() => {
  const current = tab.value;
  if (current === null) return [];
  const grid: (NamedContraband | null)[][] = Array.from({ length: current.height }, () =>
    Array.from({ length: current.width }, () => null),
  );
  for (const item of current.items) {
    if (item.y < current.height && item.x < current.width) {
      const row = grid[item.y];
      if (row) row[item.x] = item;
    }
  }
  return grid;
});

const readout = ref<NamedContraband | null>(null);

const totalItems = computed(() => stash.value.tabs.reduce((sum, t) => sum + t.items.length, 0));

const foot = computed(() => {
  const current = tab.value;
  if (current === null) return "";
  return `TAB ${tabIndex.value + 1} — ${current.items.length} OF ${current.width * current.height} CELLS WRITTEN`;
});
</script>

<template>
  <div>
    <p class="sl-column-head text-sl-ink-soft mb-[26px]">
      shared transfer stash · {{ stash.tabs.length }} TABS · {{ totalItems }} ITEMS
    </p>

    <!-- The shared stash's own flag, when transfer.gst would not turn -->
    <div v-if="stash.flagged !== null" class="border-l-4 border-sl-wax pl-[12px]">
      <EmptyShelf state="wontTurn" />
      <p class="sl-entry-sub text-sl-ink-soft mt-[13px]">{{ stash.flagged }}</p>
    </div>

    <template v-else>
      <!-- Tab teeth -->
      <nav class="flex gap-[4px] mb-[13px]" aria-label="Stash tabs">
        <button
          v-for="(_, i) in stash.tabs"
          :key="i"
          class="sl-entry-figure px-[10px] border border-sl-rule-strong focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
          :class="
            i === tabIndex ? 'bg-sl-ink text-sl-folio' : 'text-sl-ink hover:bg-sl-folio-shade'
          "
          :aria-current="i === tabIndex ? 'true' : undefined"
          @click="tabIndex = i"
        >
          {{ i + 1 }}
        </button>
      </nav>

      <!-- The pegboard, at the tab's own parsed geometry -->
      <div v-if="tab !== null" class="overflow-x-auto pb-[13px]">
        <div v-for="(row, y) in cells" :key="y" class="flex" data-testid="stash-row">
          <StashCell v-for="(cell, x) in row" :key="x" :item="cell" @readout="readout = $event" />
        </div>
      </div>

      <!-- The margin readout: bookkeeping, not chrome -->
      <div
        class="border border-sl-rule-strong px-[12px] h-[52px] mb-[13px]"
        data-testid="margin-readout"
      >
        <template v-if="readout !== null">
          <p class="font-folio text-[15px] leading-[26px] text-sl-ink truncate">
            {{ readout.name ?? readout.recordPath }} · {{ readout.stack }} in hand · TAB
            {{ tabIndex + 1 }}, CELL {{ readout.x }},{{ readout.y }}
          </p>
          <p class="sl-entry-sub text-sl-ink-soft truncate">{{ readout.recordPath }}</p>
        </template>
      </div>

      <p class="sl-foot text-sl-ink-soft border-t border-sl-rule-strong">{{ foot }}</p>
    </template>
  </div>
</template>
