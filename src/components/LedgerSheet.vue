<script setup lang="ts">
// The sheet — the iron plate, one value-step up from the room. Iron & Ichor
// is dark-everywhere (no polarity flip), so the #00012 `.sl-folio` alias remap
// is RETIRED: the :root aliases are globally correct and there is no second
// ground to re-declare. The var()-indirection trap dissolves by construction.
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
    class="relative bg-sl-folio text-sl-ink p-[32px_40px] shadow-folio overflow-y-auto"
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
