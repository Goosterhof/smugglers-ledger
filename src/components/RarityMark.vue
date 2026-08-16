<script setup lang="ts">
// The stamped rarity mark: six ordinals, ascending density. The GLYPH is the
// carrier; the ink colour is reinforcement (WCAG 1.4.1 — rarity is never
// carried by colour alone). Mapping settled by the bench enumeration
// 2026-08-16: Common · Magical · Rare · Epic · Legendary · Quest.
import { computed } from "vue";

const GLYPHS = ["·", "◦", "◇", "◆", "✦", "✷"] as const;
const NAMES = ["Common", "Magical", "Rare", "Epic", "Legendary", "Quest"] as const;

const { tier } = defineProps<{ tier: number }>();

const safeTier = computed(() => (tier >= 0 && tier <= 5 ? tier : 0));
</script>

<template>
  <span
    class="font-figure text-[11px] leading-[26px] text-center"
    :style="{ color: `var(--sl-tier-${safeTier})` }"
    :title="NAMES[safeTier]"
    :aria-label="NAMES[safeTier]"
    data-testid="rarity-mark"
  >
    {{ GLYPHS[safeTier] }}
  </span>
</template>
