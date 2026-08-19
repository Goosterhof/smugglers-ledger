<script setup lang="ts">
// THE TRADES — the ten masteries as a ruled index, each one unfolding its own
// tree in place the way an entry unfolds its docket. A search over every skill
// in the game sits at the head of the page; the "carried by" cut dresses the
// trees in one hand's own ranks, so the panel answers both questions a player
// has: what CAN be learned here, and what did I actually learn.
import { computed, ref } from "vue";
import SkillTree from "@/components/SkillTree.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import { useTrades, type TradeHit } from "@/composables/useTrades";
import { useLedger } from "@/composables/useLedger";

const {
  trees,
  openTrade,
  openTradeAt,
  carriedBy,
  query,
  hits,
  strike,
  showInTree,
  shelfNote,
  loaded,
  standingOf,
  barLevelOf,
  handsCarrying,
} = useTrades();
const { characters } = useLedger();

/** The record the search sent us to — struck with a mark in its tree. */
const marked = ref<string | null>(null);

function follow(hit: TradeHit): void {
  marked.value = hit.node.record;
  showInTree(hit);
}

const hands = computed(() => characters.value.map((c) => c.name));

/** One index row per mastery: how much there is to learn, and who learned it. */
const rows = computed(() =>
  trees.value.map((tree) => {
    const carriers = handsCarrying(tree);
    return {
      tree,
      skills: tree.nodes.length,
      carriers,
      /** The chosen hand's own standing in this trade, when they carry it. */
      bar: barLevelOf(tree),
      learned: tree.nodes.filter((node) => standingOf(tree, node).bought > 0).length,
    };
  }),
);

/** The search's own footing. A ledger foots its columns; it does not print
 * "1 TRADES". */
const hitsFoot = computed(() => {
  const trades = new Set(hits.value.map((hit) => hit.tree.name)).size;
  const skills = hits.value.length === 1 ? "1 SKILL" : `${hits.value.length} SKILLS`;
  return `${skills} ACROSS ${trades === 1 ? "1 TRADE" : `${trades} TRADES`}`;
});

const foot = computed(() => {
  const skills = rows.value.reduce((sum, row) => sum + row.skills, 0);
  const trades = rows.value.length === 1 ? "1 TRADE" : `${rows.value.length} TRADES`;
  const base = `${trades} · ${skills} SKILLS`;
  if (carriedBy.value === null) return base;
  const learned = rows.value.reduce((sum, row) => sum + row.learned, 0);
  return `${base} — ${carriedBy.value.toUpperCase()} HAS LEARNED ${learned}`;
});
</script>

<template>
  <div>
    <!-- The index rule, in the trades' own words: this one searches the game,
         not the hoard, so it says so rather than borrowing THE HOARD's line. -->
    <div
      class="flex items-center gap-[8px] h-[26px] mb-[26px] border-b border-sl-rule-strong focus-within:border-b-2 focus-within:border-sl-lamp"
    >
      <span class="text-sl-ink-soft" aria-hidden="true">▸</span>
      <input
        :value="query"
        type="search"
        class="flex-1 bg-transparent border-0 outline-none sl-entry-where text-sl-ink placeholder:font-folio placeholder:italic placeholder:normal-case placeholder:tracking-normal placeholder:text-sl-ink-soft"
        placeholder="name a skill…"
        aria-label="Search every skill in every trade"
        data-testid="trades-rule"
        @input="query = ($event.target as HTMLInputElement).value"
      />
      <button
        v-if="query !== ''"
        class="sl-chrome-label text-sl-wax focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        data-testid="trades-strike"
        @click="strike()"
      >
        Strike
      </button>
    </div>

    <!-- The one cut this page has: whose ranks the trees wear. -->
    <div class="flex flex-wrap items-baseline gap-x-[16px] gap-y-[4px] mb-[22px] mt-[-13px]">
      <span class="sl-column-head text-sl-ink-soft" aria-hidden="true">Carried by</span>
      <select
        :value="carriedBy ?? ''"
        class="sl-column-head bg-transparent border-0 text-sl-ink outline-none cursor-pointer"
        :class="
          carriedBy !== null
            ? 'border-b-2 border-sl-lamp'
            : 'border-b border-sl-rule focus-visible:border-sl-lamp'
        "
        aria-label="Wear one hand's own ranks"
        data-testid="carried-by"
        @change="carriedBy = ($event.target as HTMLSelectElement).value || null"
      >
        <option value="">NOBODY — THE TRADE AS IT STANDS</option>
        <option v-for="hand in hands" :key="hand" :value="hand">{{ hand.toUpperCase() }}</option>
      </select>
    </div>

    <p v-if="shelfNote !== null" class="sl-state-voice text-sl-ink-soft" data-testid="trades-note">
      The trades live in the game's own install, and the Ledger cannot reach it. Point me at Grim
      Dawn and the ten panels will read themselves.
    </p>

    <p v-else-if="!loaded" class="sl-state-voice text-sl-ink-soft" data-testid="trades-reading">
      Reading the ten panels off the shelves…
    </p>

    <!-- The search's answer: ruled rows, each naming where the skill sits. -->
    <template v-else-if="query.trim() !== ''">
      <EmptyShelf v-if="hits.length === 0" state="noResults" />
      <template v-else>
        <div
          class="grid grid-cols-[minmax(160px,1fr)_140px_92px_minmax(120px,0.8fr)] px-[8px] bg-sl-folio-shade border-b border-sl-rule-strong"
        >
          <span class="sl-column-head text-sl-ink-soft">Skill</span>
          <span class="sl-column-head text-sl-ink-soft">Trade</span>
          <span class="sl-column-head text-sl-ink-soft">Opens at</span>
          <span class="sl-column-head text-sl-ink-soft">Ranks</span>
        </div>
        <button
          v-for="hit in hits"
          :key="`${hit.tree.classIndex}-${hit.node.record}`"
          class="w-full grid grid-cols-[minmax(160px,1fr)_140px_92px_minmax(120px,0.8fr)] px-[8px] text-left border-b border-sl-rule hover:bg-sl-clot-deep focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
          :data-testid="`trade-hit-${hit.node.record}`"
          @click="follow(hit)"
        >
          <span class="sl-entry-where text-sl-ink">{{ hit.node.name }}</span>
          <span class="sl-entry-where text-sl-ink-soft">{{ hit.tree.name }}</span>
          <span class="sl-entry-figure text-sl-ember">Mastery {{ hit.node.unlockLevel }}</span>
          <!-- With a hand chosen the column answers "do I have this"; with
               nobody carrying, it answers "what is it worth at most". -->
          <span
            v-if="carriedBy === null"
            class="sl-entry-figure text-sl-ink-soft"
            :data-testid="`hit-ranks-${hit.node.record}`"
          >
            {{ hit.node.maxLevel }} · ult {{ hit.node.ultimateLevel }}
          </span>
          <span
            v-else-if="standingOf(hit.tree, hit.node).bought > 0"
            class="sl-entry-figure text-sl-ember"
            :data-testid="`hit-ranks-${hit.node.record}`"
          >
            {{ standingOf(hit.tree, hit.node).total }} / {{ hit.node.ultimateLevel }}
          </span>
          <span
            v-else
            class="sl-entry-figure text-sl-ink-soft"
            :data-testid="`hit-ranks-${hit.node.record}`"
          >
            not learned
          </span>
        </button>
        <p
          class="sl-foot text-sl-ink-soft border-t border-sl-rule-strong mt-[-1px]"
          data-testid="hits-foot"
        >
          {{ hitsFoot }}
        </p>
      </template>
    </template>

    <!-- The index itself: one ruled row per trade, unfolding its own tree. -->
    <template v-else>
      <div
        class="grid grid-cols-[minmax(160px,1fr)_92px_92px_minmax(160px,1fr)] px-[8px] bg-sl-folio-shade border-b border-sl-rule-strong"
      >
        <span class="sl-column-head text-sl-ink-soft">Trade</span>
        <span class="sl-column-head text-sl-ink-soft">Skills</span>
        <span class="sl-column-head text-sl-ink-soft">Bar</span>
        <span class="sl-column-head text-sl-ink-soft">Carried by</span>
      </div>

      <div v-for="row in rows" :key="row.tree.classIndex" class="border-b border-sl-rule">
        <button
          class="w-full grid grid-cols-[minmax(160px,1fr)_92px_92px_minmax(160px,1fr)] px-[8px] text-left focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
          :class="
            openTrade === row.tree.classIndex
              ? 'sl-chit-taken border-b-2 border-sl-lamp'
              : 'text-sl-ink hover:bg-sl-clot-deep'
          "
          :aria-expanded="openTrade === row.tree.classIndex"
          :data-testid="`trade-row-${row.tree.name}`"
          @click="openTradeAt(row.tree.classIndex)"
        >
          <span class="sl-entry-where text-sl-ink">
            <span class="text-sl-ink-soft mr-[6px]" aria-hidden="true">
              {{ openTrade === row.tree.classIndex ? "▾" : "▸" }}
            </span>
            {{ row.tree.name }}
          </span>
          <span class="sl-entry-figure text-sl-ink-soft">{{ row.skills }}</span>
          <span class="sl-entry-figure" :class="row.bar > 0 ? 'text-sl-ember' : 'text-sl-ink-soft'">
            {{ carriedBy === null ? "—" : `${row.bar} / ${row.tree.barMaxLevel}` }}
          </span>
          <span class="sl-entry-sub text-sl-ink-soft self-center">
            {{ row.carriers.length === 0 ? "no hand" : row.carriers.join(", ") }}
          </span>
        </button>

        <SkillTree
          v-if="openTrade === row.tree.classIndex"
          :tree="row.tree"
          :mark="marked"
          class="px-[8px] pb-[13px]"
        />
      </div>

      <p
        class="sl-foot text-sl-ink-soft border-t border-sl-rule-strong mt-[-1px]"
        data-testid="trades-foot"
      >
        {{ foot }}
      </p>
    </template>
  </div>
</template>
