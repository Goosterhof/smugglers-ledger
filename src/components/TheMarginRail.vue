<script setup lang="ts">
// The margin rail — the room's furniture, on the soot, never on the page.
// THE ROOT: the chosen save root plus every other discovered root as a
// one-click switch (RD-4 — a second profile's hoard must never be silently
// invisible). THE HANDS: the ten characters, flagged ones struck. (THE MARKS
// glyph legend was retired 2026-08-16 with the RarityMark component — rarity
// is the coloured word now, no glyphs to legend.)
import { computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useLedger } from "@/composables/useLedger";

const { overview, characters, switchRoot, openManifest, state } = useLedger();

const rootLeaf = (path: string): string => {
  const parts = path.split(/[\\/]/).filter((p) => p !== "");
  return parts.slice(-4).join("/");
};

const otherRoots = computed(() =>
  overview.value.roots.filter((r) => r.path !== overview.value.chosenRoot),
);

async function pickRootByHand(): Promise<void> {
  const picked = await open({ directory: true, title: "Point me at a save folder" });
  if (typeof picked === "string") {
    await switchRoot(picked);
  }
}
</script>

<template>
  <aside class="w-[200px] shrink-0 flex flex-col gap-[26px] pt-[26px] overflow-y-auto">
    <section aria-label="The root">
      <h2 class="sl-chrome-label text-sl-chalk-soft mb-[8px]">The Root</h2>
      <p
        v-if="overview.chosenRoot !== null"
        class="font-figure text-[11px] leading-[16px] text-sl-chalk border-l-2 border-sl-lamp pl-[8px] break-all"
      >
        {{ rootLeaf(overview.chosenRoot) }}
      </p>
      <button
        v-for="root in otherRoots"
        :key="root.path"
        class="block w-full text-left font-figure text-[11px] leading-[16px] text-sl-chalk-soft pl-[10px] mt-[8px] break-all hover:text-sl-chalk focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        :title="`Switch to ${root.path} (${root.saveCount} save files)`"
        @click="switchRoot(root.path)"
      >
        {{ rootLeaf(root.path) }} <span aria-hidden="true">→</span>
      </button>
      <button
        v-if="state === 'noSaves'"
        class="sl-chrome-label mt-[8px] px-[10px] py-[6px] border-2 border-sl-iron text-sl-lamp hover:border-sl-lamp focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        @click="pickRootByHand"
      >
        Point me at it
      </button>
    </section>

    <section aria-label="The hands">
      <h2 class="sl-chrome-label text-sl-chalk-soft mb-[8px]">The Hands</h2>
      <button
        v-for="hand in characters"
        :key="hand.name"
        class="flex w-full items-baseline justify-between gap-[8px] text-left font-figure text-[12px] leading-[22px] hover:text-sl-chalk focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        :class="hand.flagged !== null ? 'text-sl-wax-lit' : 'text-sl-chalk-soft'"
        @click="openManifest(hand.name)"
      >
        <span class="truncate">
          <span v-if="hand.flagged !== null" aria-hidden="true">⚠ </span>{{ hand.name }}
        </span>
        <span v-if="hand.flagged === null" class="tabular-nums">{{ hand.level }}</span>
      </button>
    </section>
  </aside>
</template>
