<script setup lang="ts">
// THE DOCKET — the detailed-search line under the index rule. Not a filter
// toolbar: a clerk's docket of cuts, written in the sheet's own small caps.
// Each stamp is a toggle; an active cut is inked full and underlined the way
// a bookkeeper rules a subtotal. LIFT clears every cut (the docket's own
// "Strike"). The hand cut is a plain select dressed as ledger text — one
// owner at a time, because a docket names one party per page.
import { computed } from "vue";
import { PLACE_NAMES, RARITY_NAMES } from "@/types/ledger";
import { useLedger } from "@/composables/useLedger";

const {
  characters,
  tierCut,
  placeCut,
  handCut,
  docketCut,
  toggleTier,
  togglePlace,
  cutHand,
  liftCuts,
} = useLedger();

const hands = computed(() => [
  ...characters.value.map((c) => c.name.toUpperCase()),
  "SHARED STASH",
]);
</script>

<template>
  <div
    class="flex flex-wrap items-baseline gap-x-[16px] gap-y-[4px] mb-[22px] mt-[-13px]"
    data-testid="docket-cuts"
  >
    <span class="sl-column-head text-sl-ink-soft" aria-hidden="true">Cut by</span>

    <span class="flex items-baseline gap-[10px]" role="group" aria-label="Cut by rarity">
      <button
        v-for="(word, tier) in RARITY_NAMES"
        :key="word"
        class="sl-column-head normal-case tracking-[0.4px] focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        :class="tierCut.has(tier) ? 'border-b-2' : 'opacity-60 hover:opacity-100'"
        :style="{
          color: `var(--sl-loot-${tier})`,
          borderColor: tierCut.has(tier) ? `var(--sl-loot-${tier})` : undefined,
        }"
        :aria-pressed="tierCut.has(tier)"
        :data-testid="`cut-tier-${tier}`"
        @click="toggleTier(tier)"
      >
        {{ word }}
      </button>
    </span>

    <span class="text-sl-rule-strong" aria-hidden="true">·</span>

    <span class="flex items-baseline gap-[10px]" role="group" aria-label="Cut by place">
      <button
        v-for="place in PLACE_NAMES"
        :key="place"
        class="sl-column-head focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        :class="
          placeCut.has(place)
            ? 'text-sl-ink border-b-2 border-sl-ink'
            : 'text-sl-ink-soft hover:text-sl-ink'
        "
        :aria-pressed="placeCut.has(place)"
        :data-testid="`cut-place-${place.replace(/ /g, '-')}`"
        @click="togglePlace(place)"
      >
        {{ place }}
      </button>
    </span>

    <span class="text-sl-rule-strong" aria-hidden="true">·</span>

    <select
      :value="handCut ?? ''"
      class="sl-column-head bg-transparent border-0 border-b border-sl-rule text-sl-ink outline-none focus-visible:border-sl-ink cursor-pointer"
      aria-label="Cut by hand"
      data-testid="cut-hand"
      @change="cutHand(($event.target as HTMLSelectElement).value || null)"
    >
      <option value="">EVERY HAND</option>
      <option v-for="hand in hands" :key="hand" :value="hand">{{ hand }}</option>
    </select>

    <button
      v-if="docketCut"
      class="sl-chrome-label text-sl-wax focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
      data-testid="lift-cuts"
      @click="liftCuts()"
    >
      Lift
    </button>
  </div>
</template>
