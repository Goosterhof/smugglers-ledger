<script setup lang="ts">
// One warehouse cell — a pegboard square at the tab's own geometry. The grid
// must READ as a grid (the investor saw items floating in blank space when the
// hairlines were too faint to see), so every cell carries a visible rule.
// Occupied: a rarity-loot wash fills the whole cell (an item SITS here, not a
// dot in a corner) with its stack count; a solid corner tick keeps the rarity
// unambiguous. Empty: the ruled square, nothing in it. Item ICONS (from the
// game's Items.arc) are the next arc — until then the wash is the item.
import { computed } from "vue";
import type { NamedContraband } from "@/types/ledger";

const { item = null, compact = false } = defineProps<{
  item?: NamedContraband | null;
  /** 39px cells below 1180px, 52px at and above (2× / 1.5× pitch). */
  compact?: boolean;
}>();

const emit = defineEmits<{ readout: [item: NamedContraband | null] }>();

const tier = computed(() => (item && item.tier >= 0 && item.tier <= 5 ? item.tier : 0));
</script>

<template>
  <div
    class="border border-sl-rule-strong relative"
    :class="[
      compact ? 'w-[39px] h-[39px]' : 'w-[52px] h-[52px]',
      item !== null ? 'hover:brightness-110 focus:brightness-110' : '',
    ]"
    :tabindex="item !== null ? 0 : -1"
    :aria-label="
      item !== null ? `${item.name ?? item.recordPath}, ${item.stack} in hand` : undefined
    "
    data-testid="stash-cell"
    :data-occupied="item !== null ? 'true' : undefined"
    @mouseenter="emit('readout', item)"
    @focus="emit('readout', item)"
  >
    <template v-if="item !== null">
      <!-- Full-cell rarity wash — the item's footprint, in the game's own hue -->
      <span
        class="absolute inset-0 opacity-[0.28]"
        :style="{ background: `var(--sl-loot-${tier})` }"
        aria-hidden="true"
      />
      <!-- Solid corner tick — rarity, unambiguous even where the wash is faint -->
      <span
        class="absolute top-0 left-0 w-[8px] h-[8px] shadow-[inset_0_0_0_1px_rgba(0,0,0,0.35)]"
        :style="{ background: `var(--sl-loot-${tier})` }"
        aria-hidden="true"
      />
      <span
        v-if="item.stack > 1"
        class="absolute bottom-[1px] right-[3px] font-figure text-[11px] tabular-nums text-sl-ink"
      >
        {{ item.stack }}
      </span>
    </template>
  </div>
</template>
