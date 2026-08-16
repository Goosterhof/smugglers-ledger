<script setup lang="ts">
// The title wings: app name and the standing READ-ONLY seal — RD-2 worn on
// the face. The gadget's most important promise should be visible without
// reading a README.
</script>

<template>
  <header
    class="relative flex items-center justify-between h-[48px] px-[24px] bg-sl-cellar border-b-2 border-sl-iron"
  >
    <h1 class="font-chrome text-[13px] font-800 tracking-[3px] uppercase text-sl-chalk">
      The Smuggler's Ledger
    </h1>
    <!-- The label is ash (ichor as small text just misses the floor, gate-fixed);
         only the ⌗ mark stays ichor — a glyph, not text. -->
    <p
      class="sl-chrome-label text-sl-chalk"
      title="The Ledger opens save files for reading and never for writing."
    >
      <span class="text-sl-lamp" aria-hidden="true">⌗</span> Read-only
    </p>
  </header>
</template>
