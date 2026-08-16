<script setup lang="ts">
// THE SHIPMENT STRIP — a single chrome line along the room's bottom edge.
// Soot ground, chalk hand, no glow, no toast, no modal: the border official
// states the fact and waits. Three faces:
//   cleared  — "A new edition has cleared the border" + TAKE DELIVERY / STAND PAT
//   crossing — the percentage, stated plainly; at 100 the hands change
//   refused  — the seal did not verify; the Ledger stands as it is
import { useShipment } from "@/shipment/useShipment";

const { status, editionVersion, crossingPct, visible, takeDelivery, standPat } = useShipment();
</script>

<template>
  <div
    v-if="visible"
    class="relative flex items-baseline gap-[16px] h-[26px] px-[24px] border-t border-sl-iron bg-sl-cellar"
    data-testid="shipment-strip"
  >
    <template v-if="status === 'cleared'">
      <span class="sl-chrome-label text-sl-lamp">Shipment</span>
      <span class="font-folio text-[14px] leading-[26px] text-sl-chalk">
        A new edition has cleared the border — v{{ editionVersion }}.
      </span>
      <button
        class="sl-chrome-label text-sl-lamp focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        data-testid="take-delivery"
        @click="takeDelivery()"
      >
        Take delivery
      </button>
      <button
        class="sl-chrome-label text-sl-chalk-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        data-testid="stand-pat"
        @click="standPat()"
      >
        Stand pat
      </button>
    </template>

    <template v-else-if="status === 'crossing'">
      <span class="sl-chrome-label text-sl-lamp">Shipment</span>
      <span class="font-folio text-[14px] leading-[26px] text-sl-chalk" data-testid="crossing">
        {{
          crossingPct >= 100
            ? "Changing hands — the Ledger reopens itself."
            : `Under way — ${crossingPct}%. The seal is checked at the door.`
        }}
      </span>
    </template>

    <template v-else-if="status === 'refused'">
      <span class="sl-chrome-label text-sl-wax-lit">Refused</span>
      <span class="font-folio text-[14px] leading-[26px] text-sl-chalk" data-testid="refused">
        The seal on that shipment does not verify. The Ledger stands as it is — nothing was
        installed.
      </span>
      <button
        class="sl-chrome-label text-sl-chalk-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
        data-testid="stand-pat"
        @click="standPat()"
      >
        Noted
      </button>
    </template>
  </div>
</template>
