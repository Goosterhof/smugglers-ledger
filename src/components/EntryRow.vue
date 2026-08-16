<script setup lang="ts">
// One ruled entry. States per #00012: default, hover (field lifts, ink
// deepens — no transform, no shadow), keyboard focus (2px lamp inset rule on
// the left edge — the only place the lamp touches paper, and it is a rule,
// not a glow), wet (just arrived — THE WET INK), struck (flagged), and
// unresolved (the raw record path IS the name, and search matches it).
import type { LedgerHit } from "@/types/ledger";
import RarityMark from "@/components/RarityMark.vue";

const {
  hit,
  wet = false,
  banded = false,
  struck = false,
} = defineProps<{
  hit: LedgerHit;
  /** Just arrived or changed: mounts at 2× pitch, full ink, +0.4px tracking,
   * settles to 1× pitch over --sl-dur-ink. Reaction surface ONLY — no quill,
   * no hand, no page turn, no toast, no spinner. */
  wet?: boolean;
  /** Every 5th entry sits on the counting band — grouping, not zebra. */
  banded?: boolean;
  struck?: boolean;
}>();
</script>

<template>
  <div
    class="grid grid-cols-[minmax(160px,1fr)_44px_72px_minmax(200px,1.2fr)] items-baseline border-b border-sl-rule px-[8px] transition-all focus-visible:outline-none focus-visible:shadow-[inset_2px_0_0_0_var(--sl-lamp)] hover:bg-sl-folio-shade group"
    :class="[
      wet ? 'h-[52px] tracking-[0.4px]' : 'h-[26px] tracking-normal',
      banded && !wet ? 'bg-sl-folio-shade' : '',
      struck ? 'line-through decoration-sl-wax' : '',
    ]"
    :style="{
      transitionDuration: 'var(--sl-dur-ink)',
      transitionTimingFunction: 'var(--sl-ease-settle)',
    }"
    tabindex="0"
    data-testid="entry-row"
    :data-wet="wet ? 'true' : undefined"
  >
    <span
      v-if="hit.name !== null"
      class="font-folio text-[15px] leading-[26px] text-sl-ink truncate"
      >{{ hit.name }}</span
    >
    <!-- unresolved: the raw path IS the name — searchable by the same string (4C) -->
    <span
      v-else
      class="sl-entry-sub text-sl-ink-soft self-center truncate"
      :title="hit.recordPath"
      >{{ hit.recordPath }}</span
    >

    <RarityMark :tier="hit.tier" />

    <span class="sl-entry-figure text-sl-ink text-right pr-[12px]">{{ hit.stack }}</span>

    <!-- WHEREABOUTS outranks the name (B3): full ink, always the darkest
         thing on the row — the product's question is WHERE. -->
    <span class="sl-entry-where text-sl-ink truncate" :title="hit.location">{{
      hit.location
    }}</span>
  </div>
</template>
