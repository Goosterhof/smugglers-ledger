<script setup lang="ts">
// THE MANIFEST — one build, fully dressed. Deliberate layout disruption
// (#00038): the twelve equipment slots hang as a paper doll of ruled boxes
// off the margin rule, six left, six right; weapon sets footed across the
// bottom; the bags/personal-stash table resumes the ledger rhythm below.
// An empty slot is an empty ruled box with its slot name and nothing in it.
import { computed } from "vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import RarityMark from "@/components/RarityMark.vue";
import { useLedger } from "@/composables/useLedger";
import type { CharacterSheet, NamedContraband } from "@/types/ledger";
import { EQUIPMENT_SLOT_NAMES } from "@/types/ledger";

const { characters, chosenHand } = useLedger();

const hand = computed<CharacterSheet | null>(() => {
  const byName = characters.value.find((c) => c.name === chosenHand.value);
  return byName ?? characters.value[0] ?? null;
});

const leftSlots = computed(() => (hand.value?.equipment ?? []).slice(0, 6));
const rightSlots = computed(() => (hand.value?.equipment ?? []).slice(6, 12));

const bagEntries = computed(() => {
  const sheet = hand.value;
  if (sheet === null) return [];
  const rows: { item: NamedContraband; where: string }[] = [];
  for (const [b, bag] of sheet.bags.entries()) {
    for (const item of bag) {
      rows.push({ item, where: `BAGS, BAG ${b + 1}, CELL ${item.x},${item.y}` });
    }
  }
  for (const [t, tab] of sheet.personalStash.entries()) {
    for (const item of tab.items) {
      rows.push({ item, where: `PERSONAL STASH, TAB ${t + 1}, CELL ${item.x},${item.y}` });
    }
  }
  return rows;
});

const classLabel = computed(() => {
  const tag = hand.value?.classTag ?? "";
  return tag.replace("tagSkillClassName", "CLASS ");
});

function itemLabel(item: NamedContraband): string {
  return item.name ?? item.recordPath;
}
</script>

<template>
  <div v-if="hand !== null">
    <p class="sl-column-head text-sl-ink-soft mb-[26px]">
      lvl {{ hand.level }} · {{ classLabel }} · <span aria-hidden="true">⌗</span>
      {{ hand.iron.toLocaleString("en-US") }} iron
      <span v-if="hand.hardcore"> · HARDCORE</span>
    </p>

    <!-- A save the cipher couldn't turn: struck and wax-marked, still on the page -->
    <div v-if="hand.flagged !== null" class="border-l-4 border-sl-wax pl-[12px]">
      <EmptyShelf state="wontTurn" />
      <p class="sl-entry-sub text-sl-ink-soft mt-[13px]">{{ hand.flagged }}</p>
    </div>

    <template v-else>
      <!-- The paper doll: six left at margin, six right — a body, not a list -->
      <div class="grid grid-cols-2 gap-x-[48px] gap-y-[4px] mb-[26px]">
        <div
          v-for="(slot, i) in [...leftSlots, ...rightSlots]"
          :key="EQUIPMENT_SLOT_NAMES[i < 6 ? i : i]"
          class="flex items-baseline gap-[8px]"
          :class="i >= 6 ? 'flex-row-reverse text-right' : ''"
        >
          <span class="sl-column-head text-sl-ink-soft w-[86px] shrink-0">
            {{ EQUIPMENT_SLOT_NAMES[i] }}
          </span>
          <span
            class="flex-1 border border-sl-rule px-[8px] h-[26px] font-folio text-[14px] leading-[24px] text-sl-ink truncate"
            :title="slot !== null ? itemLabel(slot) : undefined"
          >
            <template v-if="slot !== null">
              <RarityMark :tier="slot.tier" /> {{ itemLabel(slot) }}
            </template>
          </span>
        </div>
      </div>

      <!-- Weapon sets, footed across the bottom in two bracketed groups -->
      <div class="grid grid-cols-2 gap-x-[48px] border-t border-sl-rule-strong pt-[13px] mb-[26px]">
        <div v-for="(set, s) in [hand.weaponSet1, hand.weaponSet2]" :key="s">
          <p class="sl-column-head text-sl-ink-soft">Weapon Set {{ s === 0 ? "I" : "II" }}</p>
          <div v-for="(slot, w) in set" :key="w" class="flex items-baseline gap-[8px]">
            <span class="sl-column-head text-sl-ink-soft w-[48px] shrink-0">{{
              w === 0 ? "MAIN" : "OFF"
            }}</span>
            <span
              class="flex-1 border border-sl-rule px-[8px] h-[26px] font-folio text-[14px] leading-[24px] text-sl-ink truncate"
            >
              <template v-if="slot !== null">
                <RarityMark :tier="slot.tier" /> {{ itemLabel(slot) }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <!-- Bags + personal stash: the ledger rhythm resumes -->
      <div
        class="grid grid-cols-[minmax(160px,1fr)_44px_72px_minmax(200px,1.2fr)] px-[8px] bg-sl-folio-shade border-b border-sl-rule-strong"
      >
        <span class="sl-column-head text-sl-ink-soft">Item</span>
        <span class="sl-column-head text-sl-ink-soft">Mark</span>
        <span class="sl-column-head text-sl-ink-soft text-right pr-[12px]">Count</span>
        <span class="sl-column-head text-sl-ink">Whereabouts</span>
      </div>
      <div
        v-for="(row, i) in bagEntries"
        :key="`${row.item.recordPath}-${row.where}`"
        class="grid grid-cols-[minmax(160px,1fr)_44px_72px_minmax(200px,1.2fr)] items-baseline border-b border-sl-rule px-[8px] h-[26px]"
        :class="(i + 1) % 5 === 0 ? 'bg-sl-folio-shade' : ''"
      >
        <span
          v-if="row.item.name !== null"
          class="font-folio text-[15px] leading-[26px] text-sl-ink truncate"
          >{{ row.item.name }}</span
        >
        <span v-else class="sl-entry-sub text-sl-ink-soft self-center truncate">{{
          row.item.recordPath
        }}</span>
        <RarityMark :tier="row.item.tier" />
        <span class="sl-entry-figure text-sl-ink text-right pr-[12px]">{{ row.item.stack }}</span>
        <span class="sl-entry-where text-sl-ink truncate">{{ row.where }}</span>
      </div>

      <p class="sl-foot text-sl-ink-soft border-t border-sl-rule-strong mt-[-1px]">
        {{ bagEntries.length }} ENTRIES ON THIS HAND
      </p>
    </template>
  </div>
</template>
