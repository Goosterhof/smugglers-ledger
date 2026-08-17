<script setup lang="ts">
// One warehouse cell — a pegboard square at the tab's own geometry. The grid
// must READ as a grid (the investor saw items floating in blank space when the
// hairlines were too faint to see), so every cell carries a visible rule.
// Occupied: the item's OWN icon from the game's Items.arc, anchored at its
// true cell and spanning its true footprint (game cells are 32px in the
// source art — the decoded PNG's natural size IS the span), over a rarity-loot
// wash with its stack count; a solid corner tick keeps the rarity unambiguous
// even while an icon is still resolving or the codex has none. Empty: the
// ruled square, nothing in it.
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { NamedContraband } from "@/types/ledger";

const { item = null, compact = false } = defineProps<{
  item?: NamedContraband | null;
  /** 39px cells below 1180px, 52px at and above (2× / 1.5× pitch). */
  compact?: boolean;
}>();

const emit = defineEmits<{ readout: [item: NamedContraband | null] }>();

const tier = computed(() => (item && item.tier >= 0 && item.tier <= 5 ? item.tier : 0));

/** One game cell in the source art — GD icon bitmaps are 32px per cell. */
const GAME_CELL_PX = 32;

const iconUrl = ref<string | null>(null);
/** Footprint in cells, derived from the decoded PNG's natural size on load. */
const iconSpan = ref({ w: 1, h: 1 });
const cellPx = computed(() => (compact ? 39 : 52));

watch(
  () => item?.bitmap ?? null,
  async (bitmap) => {
    iconUrl.value = null;
    iconSpan.value = { w: 1, h: 1 };
    if (bitmap === null) return;
    try {
      const url = await invoke<string | null>("item_icon", { bitmap });
      iconUrl.value = typeof url === "string" ? url : null;
    } catch {
      iconUrl.value = null;
    }
  },
  { immediate: true },
);

function placeIcon(event: Event): void {
  const img = event.target as HTMLImageElement;
  iconSpan.value = {
    w: Math.max(1, Math.round(img.naturalWidth / GAME_CELL_PX)),
    h: Math.max(1, Math.round(img.naturalHeight / GAME_CELL_PX)),
  };
}
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
      <!-- Full-cell rarity wash — the item's footprint, in the game's own hue.
           Lower alpha on the dark ground so the wash reads as a tint, not a
           block; the loot hues are bright, a little goes far on iron. -->
      <span
        class="absolute inset-0 opacity-[0.22]"
        :style="{ background: `var(--sl-loot-${tier})` }"
        aria-hidden="true"
      />
      <!-- The item itself — anchored at its true cell, spanning its true
           footprint over the empty neighbour cells (only the anchor cell holds
           the item in the grid, so nothing double-renders). Decorative: the
           cell's aria-label already names the item. -->
      <img
        v-if="iconUrl !== null"
        :src="iconUrl"
        alt=""
        class="absolute top-0 left-0 z-1 max-w-none object-contain p-[2px] pointer-events-none"
        :style="{ width: `${iconSpan.w * cellPx}px`, height: `${iconSpan.h * cellPx}px` }"
        data-testid="cell-icon"
        @load="placeIcon"
      />
      <!-- Solid corner tick — rarity, unambiguous even where the wash is faint -->
      <span
        class="absolute top-0 left-0 z-2 w-[8px] h-[8px]"
        :style="{ background: `var(--sl-loot-${tier})` }"
        aria-hidden="true"
      />
      <span
        v-if="item.stack > 1"
        class="absolute bottom-[1px] right-[3px] z-2 font-figure text-[11px] tabular-nums text-sl-ink"
      >
        {{ item.stack }}
      </span>
    </template>
  </div>
</template>
