<script setup lang="ts">
// The six voiced states (log 4D) — owned string-by-string, gated by the
// component tests, never decorative. Set as a written note on an otherwise
// ruled, empty page: hung off the margin rule, never centred on the sheet.
import type { ShelfState } from "@/composables/useLedger";

const COPY: Record<ShelfState, string> = {
  noSaves: "No hoard found on this machine — point me at a save folder and the ledger opens.",
  noInstall: "The codex has no shelf for these records yet — point me at the install.",
  wontTurn: "The ledger won't turn — this save may be from a different game version.",
  noResults: "Nothing by that name in the hoard — not in any bag, tab, or slot.",
  firstRun: "Turning the cipher on the hoard…",
  coldCodex: "The codex is reading the shelves — first visit only, the ledger remembers.",
};

/** The two waiting states are the only ones that may carry motion — ink
 * filling the ruling left-to-right, never a spinner. Under reduced motion
 * the preflight clamps the fill to its terminal frame: a full, still line. */
const WAITING: ReadonlySet<ShelfState> = new Set(["firstRun", "coldCodex"]);

const { state, muted = false } = defineProps<{
  state: ShelfState;
  /** Secondary placement (a note under a rendered panel, not the whole sheet). */
  muted?: boolean;
}>();
</script>

<template>
  <div class="pl-[16px] pt-[26px]" data-testid="empty-shelf" :data-state="state">
    <p class="sl-state-voice" :class="muted ? 'text-sl-ink-soft' : 'text-sl-ink'" role="status">
      {{ COPY[state] }}
    </p>
    <div
      v-if="WAITING.has(state)"
      class="h-[1px] mt-[25px] bg-sl-rule-strong sl-ink-fill"
      data-testid="ink-fill"
      aria-hidden="true"
    />
  </div>
</template>

<style>
/* The ink fills the ruling left-to-right — a line being drawn, not a bar
   being loaded. Terminal frame = the full ruled line (scaleX(1)). */
.sl-ink-fill {
  transform-origin: left center;
  animation: sl-ink-draw var(--sl-dur-ink) var(--sl-ease-settle) both;
}
@keyframes sl-ink-draw {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}
</style>
