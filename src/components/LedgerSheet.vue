<script setup lang="ts">
// The folio — the lit page. The `.sl-folio` scope class lives here and
// nowhere else: it carries the full alias remap AND every re-declared
// indirected map (the var()-indirection trap, #00012 The Two Grounds).
import { computed } from "vue";
import { useLedger } from "@/composables/useLedger";

const { title } = defineProps<{ title: string }>();
const { overview } = useLedger();

const lastTurned = computed(() => {
  const ms = overview.value.lastTurnedEpochMs;
  if (ms === 0) return "—";
  return new Date(ms).toLocaleTimeString("en-GB", { hour12: false });
});
</script>

<template>
  <section
    class="sl-folio relative bg-sl-folio text-sl-ink p-[32px_40px] shadow-folio overflow-y-auto"
    aria-label="The ledger"
  >
    <header class="flex items-baseline justify-between h-[26px] mb-[26px]">
      <h2 class="font-folio text-[26px] leading-[26px] text-sl-ink font-400">{{ title }}</h2>
      <p class="sl-column-head text-sl-ink-soft z-30" aria-live="polite" data-testid="last-turned">
        <span aria-hidden="true">⌗</span> Last turned {{ lastTurned }}
      </p>
    </header>
    <div class="border-l border-sl-rule-strong pl-[16px] min-h-[80%]" :style="{ marginLeft: '0' }">
      <slot />
    </div>
  </section>
</template>

<style>
/* The alias remap. Re-declare INDIRECTED maps here too — a :root map of
   var()-to-var() bakes at :root and inherits the SOOT value into the page
   (ink-on-paper-compositing.md; #00012 calls this the trap that bites). */
.sl-folio {
  --sl-surface: var(--sl-folio);
  --sl-text: var(--sl-ink);
  --sl-text-muted: var(--sl-ink-soft);
  --sl-line: var(--sl-rule);
  --sl-accent: var(--sl-wax);
  /* re-declared indirected map — NOT inherited: */
  --sl-input-bg: var(--sl-folio);
  --sl-input-line: var(--sl-rule-strong);
}
</style>
