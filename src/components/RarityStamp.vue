<script setup lang="ts">
// The rarity stamp: the game's own loot hue on a square chip, the rarity
// WORD beside it in the contrast-safe tier ink. The word is the carrier
// (investor's ruling 2026-08-16 — "words instead of the marks, so it becomes
// much clearer"); the chip is the game speaking its own color language.
// Compact mode (equipment boxes, tight inline spots) shows the chip alone
// and keeps the word in the title/aria surface.
import { computed } from "vue";
import { RARITY_NAMES } from "@/types/ledger";

const { tier, compact = false } = defineProps<{ tier: number; compact?: boolean }>();

const safeTier = computed(() => (tier >= 0 && tier <= 5 ? tier : 0));
const word = computed(() => RARITY_NAMES[safeTier.value]);
</script>

<template>
  <span
    class="inline-flex items-center gap-[6px] leading-[26px]"
    :title="word"
    :aria-label="word"
    data-testid="rarity-stamp"
  >
    <span
      class="inline-block w-[8px] h-[8px] shrink-0 shadow-[inset_0_0_0_1px_rgba(36,32,26,0.3)]"
      :style="{ background: `var(--sl-loot-${safeTier})` }"
      aria-hidden="true"
    />
    <span
      v-if="!compact"
      class="sl-column-head normal-case tracking-[0.4px]"
      :style="{ color: `var(--sl-tier-${safeTier})` }"
      >{{ word }}</span
    >
  </span>
</template>
