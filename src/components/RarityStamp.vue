<script setup lang="ts">
// The rarity word — the game's own loot colour, and nothing else. The chip
// swatch and the muted tier ink are both retired (investor ruling 2026-08-16:
// "rarity is shown by colour AND wording, from the game itself"). On the Iron
// & Ichor dark ground the vivid loot hues are legible AS text (every word
// clears its floor against --sl-folio — the whole reason the dark ground was
// forced), so the word carries rarity alone. Compact mode is kept for tight
// inline spots (equipment boxes) where the word sits beside the item name.
import { computed } from "vue";
import { RARITY_NAMES } from "@/types/ledger";

const { tier } = defineProps<{ tier: number; compact?: boolean }>();

const safeTier = computed(() => (tier >= 0 && tier <= 5 ? tier : 0));
const word = computed(() => RARITY_NAMES[safeTier.value]);
</script>

<template>
  <span
    class="sl-column-head normal-case tracking-[0.4px] leading-[26px]"
    :style="{ color: `var(--sl-loot-${safeTier})` }"
    :title="word"
    :aria-label="word"
    data-testid="rarity-stamp"
    >{{ word }}</span
  >
</template>
