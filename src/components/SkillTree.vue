<script setup lang="ts">
// One mastery panel, drawn at the game's own coordinates.
//
// The geometry is the install's, not a layout of ours: every node carries the
// panel position the game draws it at, and every line is the connector run
// the game itself lists. What changes is the unit — the game's 80-px columns
// and 70-px rows become 3 and 2 of the sheet's own pitch, so the tree rules
// with the rest of the page instead of floating on it.
//
// The one deliberate break with #00013's radius-0 rule: a modifier node is
// drawn ROUND, because round-versus-square is how the skill panel itself says
// "this one hangs off that one", and the pictures inside are the game's own.
// A shape borrowed from the subject is not chrome.
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { MasteryTree, NodeStanding, SkillNode } from "@/types/trades";
import { useTrades } from "@/composables/useTrades";
import SkillRollover from "@/components/SkillRollover.vue";

const { tree, mark = null } = defineProps<{
  tree: MasteryTree;
  /** The record the search landed on — struck with the keeper's own mark. */
  mark?: string | null;
}>();

const { standingOf, barLevelOf, carriedBy } = useTrades();

/** The sheet's units for the game's grid: 3 pitch across, 2 pitch down. */
const COLUMN = 78;
const ROW = 52;
/** The game's own spacing, which those two are the sheet's answer to, and the
 * panel x of tier 1 — the origin every column is measured from. */
const GAME_COLUMN = 80;
const GAME_ROW = 70;
const FIRST_COLUMN_X = 246;
/** The plate a skill sits on, and the smaller round one a modifier gets. */
const NODE = 34;
const SMALL = 26;
const ROLLOVER_WIDTH = 292;
/** What the rollover is worth at its tallest — the flip decision needs a
 * number before the thing is rendered, and a generous one only ever nudges a
 * short card up a little. */
const ROLLOVER_HEIGHT = 300;

const topOfPanel = computed(() => Math.min(...tree.nodes.map((n) => n.y)));
const bottomOfPanel = computed(() => Math.max(...tree.nodes.map((n) => n.y)));

/** The centre of a node's plate, in sheet pixels. */
function centre(node: SkillNode): { x: number; y: number } {
  return {
    x: ((node.x - FIRST_COLUMN_X) * COLUMN) / GAME_COLUMN + NODE / 2,
    y: ((node.y - topOfPanel.value) * ROW) / GAME_ROW + NODE / 2,
  };
}

const panelWidth = computed(() => (tree.tierUnlocks.length - 1) * COLUMN + NODE);
const panelHeight = computed(
  () => ((bottomOfPanel.value - topOfPanel.value) * ROW) / GAME_ROW + NODE,
);

/** Every node's standing, computed once per render rather than per read. */
const standings = computed<ReadonlyMap<string, NodeStanding>>(
  () => new Map(tree.nodes.map((node) => [node.record, standingOf(tree, node)])),
);
function standing(node: SkillNode): NodeStanding {
  return standings.value.get(node.record) ?? { bought: 0, granted: 0, total: 0, locked: false };
}

const byRecord = computed(() => new Map(tree.nodes.map((n) => [n.record, n])));

/** Every drawn line: parent centre to child centre. A line whose child has
 * been learned inks up — the panel's own "this branch is live" reading. */
const lines = computed(() =>
  tree.nodes.flatMap((node) => {
    if (node.parent === null) return [];
    const parent = byRecord.value.get(node.parent);
    if (parent === undefined) return [];
    return [
      {
        key: node.record,
        from: centre(parent),
        to: centre(node),
        live: standing(node).bought > 0,
      },
    ];
  }),
);

// THE PICTURES — the skill panel's own icons, decoded from the install's
// UI.arc on demand. One ask per bitmap per session, shared across trees.
const drawn = new Map<string, string | null>();
const icons = ref<ReadonlyMap<string, string | null>>(new Map());

async function drawIcons(): Promise<void> {
  const wanted = tree.nodes.map((n) => n.icon).filter((b): b is string => b !== null);
  await Promise.all(
    wanted.map(async (bitmap) => {
      if (drawn.has(bitmap)) return;
      try {
        drawn.set(bitmap, await invoke<string | null>("skill_icon", { bitmap }));
      } catch {
        // A picture that will not decode costs a picture, never the tree.
        drawn.set(bitmap, null);
      }
    }),
  );
  icons.value = new Map(drawn);
}

watch(
  () => tree.classIndex,
  () => void drawIcons(),
  { immediate: true },
);

const held = ref<SkillNode | null>(null);
const heldAt = ref<{ left: number; top: number } | null>(null);

/**
 * The rollover is pinned to the VIEWPORT, off the node's own box — not laid
 * inside the tree. The tree scrolls sideways, and a scroll container clips
 * both axes: an annotation placed inside it gets its bottom sheared off the
 * moment the node it belongs to sits low on the panel. Fixed to the window,
 * it flips left when it would run off the edge and rides up when it would run
 * off the bottom, so every one of the 311 skills can be read.
 */
function hold(node: SkillNode, event: FocusEvent | MouseEvent): void {
  const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const flipX = box.right + 8 + ROLLOVER_WIDTH > window.innerWidth;
  heldAt.value = {
    left: flipX ? box.left - 8 - ROLLOVER_WIDTH : box.right + 8,
    top: Math.max(8, Math.min(box.top, window.innerHeight - 8 - ROLLOVER_HEIGHT)),
  };
  held.value = node;
}

function release(): void {
  held.value = null;
  heldAt.value = null;
}

const barLevel = computed(() => barLevelOf(tree));

/** How far the bar has run, measured on the MILESTONES' own scale: the nine
 * numbers stand at the nine columns, so a bar at 15 reaches the fourth column
 * exactly, and a bar at 28 sits three fifths of the way from the sixth to the
 * seventh. The game's own rail reads this way; a plain level/max percentage
 * would put the fill somewhere the numbers above it do not agree with. */
const barFill = computed(() => {
  const milestones = tree.tierUnlocks;
  const level = barLevel.value;
  const at = (column: number): number => column * COLUMN + NODE / 2;
  if (level <= milestones[0]) {
    return (level / milestones[0]) * at(0);
  }
  for (let column = 0; column < milestones.length - 1; column += 1) {
    if (level <= milestones[column + 1]) {
      const run = (level - milestones[column]) / (milestones[column + 1] - milestones[column]);
      return at(column) + run * COLUMN;
    }
  }
  return at(milestones.length - 1);
});
const plate = (node: SkillNode): number => (node.circular ? SMALL : NODE);
const px = (value: number): string => `${value}px`;
</script>

<template>
  <section class="pt-[13px]" :aria-label="`The ${tree.name} tree`" data-testid="skill-tree">
    <p
      class="font-folio italic text-[13px] leading-[17px] text-sl-ink-soft mb-[13px] max-w-[640px]"
    >
      {{ tree.blurb }}
    </p>

    <div class="overflow-x-auto pb-[8px]">
      <div
        class="relative"
        :style="{ width: px(panelWidth), height: px(panelHeight) }"
        data-testid="tree-panel"
      >
        <svg
          class="absolute inset-0 pointer-events-none"
          :width="panelWidth"
          :height="panelHeight"
          aria-hidden="true"
        >
          <line
            v-for="line in lines"
            :key="line.key"
            :x1="line.from.x"
            :y1="line.from.y"
            :x2="line.to.x"
            :y2="line.to.y"
            :stroke="line.live ? 'var(--sl-ember)' : 'var(--sl-rule-strong)'"
            :stroke-width="line.live ? 2 : 1"
          />
        </svg>

        <button
          v-for="node in tree.nodes"
          :key="node.record"
          type="button"
          class="absolute grid place-items-center border-2 bg-sl-folio-shade focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
          :class="[
            mark === node.record
              ? 'border-sl-lamp'
              : standing(node).bought > 0
                ? 'border-sl-ember'
                : standing(node).locked
                  ? 'border-sl-rule'
                  : 'border-sl-iron hover:border-sl-lamp',
          ]"
          :style="{
            left: px(centre(node).x - plate(node) / 2),
            top: px(centre(node).y - plate(node) / 2),
            width: px(plate(node)),
            height: px(plate(node)),
            borderRadius: node.circular ? '50%' : '0',
            opacity: carriedBy !== null && standing(node).bought === 0 ? '0.55' : '1',
          }"
          :aria-label="`${node.name} — tier ${node.tier}, opens at mastery ${node.unlockLevel}`"
          :data-testid="`skill-node-${node.record}`"
          @mouseenter="hold(node, $event)"
          @mouseleave="release()"
          @focus="hold(node, $event)"
          @blur="release()"
        >
          <img
            v-if="node.icon !== null && icons.get(node.icon)"
            :src="icons.get(node.icon) ?? ''"
            class="max-w-none"
            :style="{ width: px(plate(node) - 6), height: px(plate(node) - 6) }"
            alt=""
          />
          <span v-else class="sl-entry-sub text-sl-ink-soft" aria-hidden="true">
            {{ node.name.slice(0, 1) }}
          </span>

          <span
            v-if="standing(node).total > 0"
            class="absolute -bottom-[9px] -right-[6px] px-[3px] bg-sl-folio sl-entry-sub"
            :class="standing(node).granted > 0 ? 'text-sl-ember' : 'text-sl-ink'"
            :data-testid="`skill-rank-${node.record}`"
          >
            {{ standing(node).total }}
          </span>
        </button>

        <div
          v-if="held !== null && heldAt !== null"
          class="fixed z-10 pointer-events-none"
          :style="{ left: px(heldAt.left), top: px(heldAt.top) }"
        >
          <SkillRollover
            :node="held"
            :tree="tree"
            :standing="standing(held)"
            :worn="carriedBy"
            :bar-level="barLevel"
          />
        </div>
      </div>

      <!-- The bar rail, where the game keeps it: the nine column numbers are
           the mastery levels each tier opens at, and the bar beneath them is
           the account those numbers are drawn against. -->
      <div
        class="mt-[13px] border-t border-sl-rule-strong pt-[6px]"
        :style="{ width: px(panelWidth) }"
      >
        <div class="relative h-[26px]" data-testid="tier-rule">
          <span
            v-for="(unlock, column) in tree.tierUnlocks"
            :key="unlock"
            class="absolute sl-column-head text-center"
            :class="carriedBy !== null && barLevel < unlock ? 'text-sl-ink-soft' : 'text-sl-ember'"
            :style="{ left: px(column * COLUMN), width: px(NODE) }"
          >
            {{ unlock }}
          </span>
        </div>
        <!-- The bar is drawn on the numbers' OWN scale: each column's number
             is a milestone standing at that column, so the fill reads against
             them instead of running on a second, private scale of its own. -->
        <div class="relative h-[6px] bg-sl-folio-shade border border-sl-rule">
          <span
            v-for="(unlock, column) in tree.tierUnlocks"
            :key="unlock"
            class="absolute top-0 bottom-0 w-[1px] bg-sl-rule-strong"
            :style="{ left: px(column * COLUMN + NODE / 2) }"
            aria-hidden="true"
          />
          <span
            class="absolute left-0 top-0 bottom-0 bg-sl-ember"
            :style="{ width: px(barFill) }"
            data-testid="bar-fill"
          />
        </div>
        <div class="flex items-baseline justify-between mt-[4px]">
          <span class="sl-column-head text-sl-ink-soft">{{ tree.name }} bar</span>
          <span class="sl-entry-figure text-sl-ink" data-testid="bar-level">
            {{ barLevel }} / {{ tree.barMaxLevel }}
          </span>
        </div>
      </div>
    </div>
  </section>
</template>
