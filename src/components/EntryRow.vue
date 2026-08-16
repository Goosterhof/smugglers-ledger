<script setup lang="ts">
// One ruled entry. States per #00012: default, hover (field lifts, ink
// deepens — no transform, no shadow), keyboard focus (2px lamp inset rule on
// the left edge — the only place the lamp touches paper, and it is a rule,
// not a glow), wet (just arrived — THE WET INK), struck (flagged), and
// unresolved (the raw record path IS the name, and search matches it).
//
// Click (or Enter) unfolds THE DOCKET — the entry's own sub-ledger
// annotation, written beneath the ruled line in the clerk's smaller hand:
// full composed name, rarity word, slot, fitted component and augment, seed,
// whereabouts, and the record path. No drawer, no modal — a ledger annotates
// in place.
import { computed, ref } from "vue";
import type { LedgerHit } from "@/types/ledger";
import { RARITY_NAMES } from "@/types/ledger";
import RarityStamp from "@/components/RarityStamp.vue";

const {
  hit,
  wet = false,
  banded = false,
  struck = false,
} = defineProps<{
  hit: LedgerHit;
  /** Just arrived or changed: mounts at 2× pitch, full ink, +0.4px tracking,
   * settles to 1× pitch over --sl-dur-ink. Reaction surface ONLY — no quill,
   * no hand, no page turn, no toast, no spinner. */
  wet?: boolean;
  /** Every 5th entry sits on the counting band — grouping, not zebra. */
  banded?: boolean;
  struck?: boolean;
}>();

const open = ref(false);

const fullName = computed(() => {
  const base = hit.name ?? hit.recordPath;
  return [hit.prefix, base, hit.suffix].filter(Boolean).join(" ");
});

/** "WeaponMelee_Sword2h" → "Sword 2h"; "ArmorProtective_Head" → "Head". */
const slotWord = computed(() => {
  if (hit.slotClass === null) return "—";
  const tail = hit.slotClass.split("_").pop() ?? hit.slotClass;
  return tail.replace(/([a-z])([A-Z0-9])/g, "$1 $2");
});

const rarityWord = computed(() => RARITY_NAMES[hit.tier >= 0 && hit.tier <= 5 ? hit.tier : 0]);
</script>

<template>
  <div>
    <div
      class="grid grid-cols-[minmax(160px,1fr)_112px_72px_minmax(200px,1.2fr)] items-baseline border-b border-sl-rule px-[8px] transition-all focus-visible:outline-none focus-visible:shadow-[inset_2px_0_0_0_var(--sl-lamp)] hover:bg-sl-folio-shade group cursor-pointer"
      :class="[
        wet ? 'h-[52px] tracking-[0.4px]' : 'h-[26px] tracking-normal',
        (banded && !wet) || open ? 'bg-sl-folio-shade' : '',
        struck ? 'line-through decoration-sl-wax' : '',
      ]"
      :style="{
        transitionDuration: 'var(--sl-dur-ink)',
        transitionTimingFunction: 'var(--sl-ease-settle)',
      }"
      tabindex="0"
      role="button"
      :aria-expanded="open"
      data-testid="entry-row"
      :data-wet="wet ? 'true' : undefined"
      @click="open = !open"
      @keydown.enter.prevent="open = !open"
    >
      <span
        v-if="hit.name !== null"
        class="font-folio text-[15px] leading-[26px] text-sl-ink truncate"
        >{{ hit.name }}</span
      >
      <!-- unresolved: the raw path IS the name — searchable by the same string (4C) -->
      <span
        v-else
        class="sl-entry-sub text-sl-ink-soft self-center truncate"
        :title="hit.recordPath"
        >{{ hit.recordPath }}</span
      >

      <RarityStamp :tier="hit.tier" />

      <span class="sl-entry-figure text-sl-ink text-right pr-[12px]">{{ hit.stack }}</span>

      <!-- WHEREABOUTS outranks the name (B3): full ink, always the darkest
           thing on the row — the product's question is WHERE. -->
      <span class="sl-entry-where text-sl-ink truncate" :title="hit.location">{{
        hit.location
      }}</span>
    </div>

    <!-- THE DOCKET — the unfolded annotation, indented off the margin rule -->
    <div
      v-if="open"
      class="border-b border-sl-rule bg-sl-folio-shade pl-[32px] pr-[8px] py-[13px] shadow-[inset_2px_0_0_0_var(--sl-lamp)]"
      data-testid="docket"
    >
      <p class="font-folio text-[15px] leading-[26px] text-sl-ink">{{ fullName }}</p>

      <!-- THE STAT BLOCK — the item's rolled properties, two-tone: ember
           magnitude, ash label. Aggregated base + affixes + component. -->
      <div
        v-if="hit.stats.length > 0"
        class="border-l border-sl-rule-strong pl-[12px] my-[6px]"
        data-testid="stat-block"
      >
        <p v-for="(stat, s) in hit.stats" :key="s" class="sl-entry-sub leading-[20px]">
          <span class="text-sl-ember tabular-nums">{{ stat.magnitude }}</span>
          <span class="text-sl-ink"> {{ stat.label }}</span>
        </p>
      </div>

      <div class="grid grid-cols-[96px_1fr] gap-x-[12px]">
        <span class="sl-column-head text-sl-ink-soft leading-[26px]">Rarity</span>
        <span class="sl-entry-sub text-sl-ink leading-[26px]"
          >{{ rarityWord }} · {{ slotWord }}</span
        >
        <span class="sl-column-head text-sl-ink-soft leading-[26px]">Fitted</span>
        <span class="sl-entry-sub text-sl-ink leading-[26px]"
          >{{ hit.component ?? "—"
          }}<template v-if="hit.augment !== null"> · {{ hit.augment }}</template></span
        >
        <span class="sl-column-head text-sl-ink-soft leading-[26px]">Held</span>
        <span class="sl-entry-sub text-sl-ink leading-[26px]"
          >{{ hit.stack }} in hand · {{ hit.location }}</span
        >
        <span class="sl-column-head text-sl-ink-soft leading-[26px]">Record</span>
        <span class="font-figure text-[11px] text-sl-ink-soft leading-[26px] break-all"
          >{{ hit.recordPath }} · seed {{ hit.seed }}</span
        >
      </div>
    </div>
  </div>
</template>
