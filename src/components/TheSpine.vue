<script setup lang="ts">
// The tab spine: four printed tabs down the sheet's stage-right edge — the
// Prompt Book's binder-thumb-tab shape, reused deliberately (#00012). The
// active tab dog-ears INTO the page: it takes the folio's own cream and ink
// and loses its left border, continuous with the sheet.
import { useLedger } from "@/composables/useLedger";
import type { PanelName } from "@/types/ledger";

const { panel } = useLedger();

const TABS: { name: PanelName; label: string }[] = [
  { name: "hoard", label: "The Hoard" },
  { name: "manifest", label: "The Manifest" },
  { name: "warehouse", label: "The Warehouse" },
  { name: "trades", label: "The Trades" },
];
</script>

<template>
  <nav class="w-[96px] shrink-0 flex flex-col gap-[4px] pt-[78px]" aria-label="The spine">
    <button
      v-for="tab in TABS"
      :key="tab.name"
      class="sl-chrome-label text-left px-[12px] py-[13px] border-2 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
      :class="
        panel === tab.name
          ? 'bg-sl-folio text-sl-ink border-sl-folio border-r-[3px] border-r-sl-lamp -ml-[26px] pl-[38px]'
          : 'bg-sl-cellar sl-chit border-sl-iron'
      "
      :style="{ transitionDuration: 'var(--sl-dur-swap)' }"
      :aria-current="panel === tab.name ? 'page' : undefined"
      @click="panel = tab.name"
    >
      {{ tab.label }}
    </button>
  </nav>
</template>
