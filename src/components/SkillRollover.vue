<script setup lang="ts">
// THE ROLLOVER — a skill's own annotation, written the way the game writes
// it: the name, what it costs the bar, what rank it stands at, and its own
// words. Same hand as THE DOCKET (in-place annotation, no modal, no drawer),
// but the tree cannot unfold a row beneath itself, so this one is pinned
// beside the node the eye is already on.
import { computed } from "vue";
import type { MasteryTree, NodeStanding, SkillNode } from "@/types/trades";

const { node, tree, standing, worn } = defineProps<{
  node: SkillNode;
  tree: MasteryTree;
  standing: NodeStanding;
  /** The hand wearing the tree, if any — the rank lines only mean something
   * when somebody is carrying the trade. */
  worn: string | null;
  /** The bar's current level, for the locked line's evidence. */
  barLevel: number;
}>();

const KIND_WORDS: Record<string, string> = {
  mastery: "Mastery",
  active: "Skill",
  passive: "Passive",
  modifier: "Modifier",
  transmuter: "Transmuter",
};

const kindWord = computed(() => KIND_WORDS[node.kind] ?? "Skill");

/** "12 / 16" bought, and what the gear makes of it. */
const rankLine = computed(() => {
  if (worn === null) return null;
  if (standing.bought === 0) {
    return standing.locked ? "Not learned — the column is shut" : "Not learned";
  }
  const base = `${standing.bought} / ${node.maxLevel}`;
  return standing.granted > 0
    ? `${base} · +${standing.granted} worn → ${standing.total} / ${node.ultimateLevel}`
    : base;
});
</script>

<template>
  <div
    class="w-[292px] bg-sl-cellar border-2 border-sl-iron px-[12px] py-[8px]"
    :style="{ boxShadow: 'var(--sl-shadow-folio)' }"
    role="tooltip"
    data-testid="skill-rollover"
  >
    <p class="font-chrome text-[13px] font-800 tracking-[1px] uppercase text-sl-chalk">
      {{ node.name }}
    </p>
    <p class="sl-chrome-label text-sl-chalk-soft mt-[2px]">
      {{ kindWord }} · {{ tree.name }} · Tier {{ node.tier }}
    </p>

    <p class="sl-entry-sub text-sl-chalk-soft mt-[8px]" data-testid="rollover-unlock">
      Opens at mastery
      <span class="text-sl-ember">{{ node.unlockLevel }}</span>
      · {{ node.maxLevel }} ranks · ultimate {{ node.ultimateLevel }}
    </p>

    <p
      v-if="rankLine !== null"
      class="sl-entry-figure text-sl-chalk mt-[4px]"
      data-testid="rollover-rank"
    >
      {{ rankLine }}
    </p>
    <p
      v-if="worn !== null && standing.locked"
      class="sl-entry-sub text-sl-wax-lit"
      data-testid="rollover-locked"
    >
      {{ worn }}'s bar stands at {{ barLevel }}.
    </p>

    <p v-if="node.conversion !== null" class="sl-entry-sub text-sl-ember mt-[4px]">
      {{ node.conversion }}
    </p>

    <p
      v-if="node.blurb !== ''"
      class="font-folio italic text-[13px] leading-[17px] text-sl-chalk mt-[8px] border-t border-sl-rule pt-[6px]"
    >
      {{ node.blurb }}
    </p>
  </div>
</template>
