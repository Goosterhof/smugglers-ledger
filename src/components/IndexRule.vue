<script setup lang="ts">
// The index rule — search as the sheet's own ruled line. No box, no fill,
// no magnifier icon: the query is written in the same hand as the answers,
// and STRIKE (the bookkeeper's correction, never "clear") appears only when
// there is something to strike.
import { onMounted, onUnmounted, ref } from "vue";
import { useLedger } from "@/composables/useLedger";

const { query, search, strike } = useLedger();
const input = ref<HTMLInputElement | null>(null);

function onKeydown(event: KeyboardEvent): void {
  const typing = event.target instanceof HTMLInputElement;
  if ((event.key === "/" || (event.ctrlKey && event.key === "f")) && !typing) {
    event.preventDefault();
    input.value?.focus();
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div
    class="flex items-center gap-[8px] h-[26px] mb-[26px] border-b border-sl-rule-strong focus-within:border-b-2 focus-within:border-sl-lamp"
  >
    <span class="text-sl-ink-soft" aria-hidden="true">▸</span>
    <input
      ref="input"
      :value="query"
      type="search"
      class="flex-1 bg-transparent border-0 outline-none sl-entry-where text-sl-ink placeholder:font-folio placeholder:italic placeholder:normal-case placeholder:tracking-normal placeholder:text-sl-ink-soft"
      placeholder="name a thing…"
      aria-label="Search the hoard"
      data-testid="index-rule"
      @input="search(($event.target as HTMLInputElement).value)"
    />
    <button
      v-if="query !== ''"
      class="sl-chrome-label text-sl-wax focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
      data-testid="strike"
      @click="strike"
    >
      Strike
    </button>
  </div>
</template>
