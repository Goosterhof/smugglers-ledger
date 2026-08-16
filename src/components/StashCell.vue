<script setup lang="ts">
// One warehouse cell. Occupied: rarity-inked corner tick + stack count.
// Empty: BLANK PAPER — no border-dash, no fill, no placeholder (B6: an empty
// cell in a ledger is not a component; it is unwritten paper).
import type { NamedContraband } from "@/types/ledger";

const { item = null, compact = false } = defineProps<{
  item?: NamedContraband | null;
  /** 39px cells below 1180px, 52px at and above (2× / 1.5× pitch). */
  compact?: boolean;
}>();

const emit = defineEmits<{ readout: [item: NamedContraband | null] }>();
</script>

<template>
  <div
    class="border border-sl-rule relative"
    :class="[
      compact ? 'w-[39px] h-[39px]' : 'w-[52px] h-[52px]',
      item !== null ? 'hover:bg-sl-folio-shade focus:bg-sl-folio-shade' : '',
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
      <!-- The corner tick wears the game's own loot hue — the stash grid
           reads like the game's, at a glance. -->
      <span
        class="absolute top-0 left-0 w-[8px] h-[8px] shadow-[inset_0_0_0_1px_rgba(36,32,26,0.3)]"
        :style="{
          background: `var(--sl-loot-${item.tier >= 0 && item.tier <= 5 ? item.tier : 0})`,
        }"
        aria-hidden="true"
      />
      <span
        v-if="item.stack > 1"
        class="absolute bottom-[2px] right-[3px] font-figure text-[11px] tabular-nums text-sl-ink"
      >
        {{ item.stack }}
      </span>
    </template>
  </div>
</template>
