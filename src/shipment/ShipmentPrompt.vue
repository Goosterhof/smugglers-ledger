<script setup lang="ts">
// THE SHIPMENT NOTICE — a centred popup, not a bottom strip (investor ruling
// 2026-08-16: on a long hoard the strip sat below the fold and was never
// seen). A notice nailed to the door: a dimmed backdrop, one hard-edged iron
// card, the ichor rule across its head. Fixed-position, so it stands over the
// hoard however far it has scrolled. Iron & Ichor throughout — ash text, one
// ichor accent as a rule, no glow. Three faces:
//   cleared  — "A new edition has cleared the border" + TAKE DELIVERY / STAND PAT
//   crossing — the percentage, stated plainly; at 100 the hands change
//   refused  — the seal did not verify; the Ledger stands as it is
import { useShipment } from "@/shipment/useShipment";

const { status, editionVersion, crossingPct, visible, takeDelivery, standPat } = useShipment();
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center bg-[rgba(0,0,0,0.6)]"
    data-testid="shipment-popup"
  >
    <!-- The card: the iron plate, an ichor rule across its head, one shadow -->
    <div
      class="w-[420px] max-w-[calc(100vw-48px)] bg-sl-folio border border-sl-rule-strong border-t-2 border-t-sl-lamp shadow-folio p-[24px]"
      role="dialog"
      aria-modal="true"
      aria-label="Shipment notice"
    >
      <template v-if="status === 'cleared'">
        <p class="sl-chrome-label text-sl-lamp mb-[8px]">
          <span aria-hidden="true">⌗</span> Shipment
        </p>
        <p class="font-folio text-[17px] leading-[26px] text-sl-ink mb-[6px]">
          A new edition has cleared the border.
        </p>
        <p class="sl-entry-sub text-sl-ink-soft mb-[20px]">
          Version {{ editionVersion }} is signed and ready. Take delivery and the Ledger reopens
          itself into the new edition; stand pat and it waits until next boot.
        </p>
        <div class="flex items-baseline justify-end gap-[20px]">
          <button
            class="sl-chrome-label text-sl-ink-soft hover:text-sl-ink focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
            data-testid="stand-pat"
            @click="standPat()"
          >
            Stand pat
          </button>
          <button
            class="sl-chrome-label text-sl-ink border-b-2 border-sl-lamp pb-[2px] focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
            data-testid="take-delivery"
            @click="takeDelivery()"
          >
            Take delivery
          </button>
        </div>
      </template>

      <template v-else-if="status === 'crossing'">
        <p class="sl-chrome-label text-sl-lamp mb-[8px]">
          <span aria-hidden="true">⌗</span> Shipment
        </p>
        <p class="font-folio text-[17px] leading-[26px] text-sl-ink" data-testid="crossing">
          {{
            crossingPct >= 100
              ? "Changing hands — the Ledger reopens itself."
              : `Under way — ${crossingPct}%.`
          }}
        </p>
        <p v-if="crossingPct < 100" class="sl-entry-sub text-sl-ink-soft mt-[6px]">
          The seal is checked at the door.
        </p>
        <!-- A drawn progress rule, ember-filled — no spinner -->
        <div class="mt-[16px] h-[3px] bg-sl-folio-shade" aria-hidden="true">
          <div
            class="h-full bg-sl-ember"
            :style="{ width: `${crossingPct}%`, transition: 'width var(--sl-dur-swap)' }"
          />
        </div>
      </template>

      <template v-else-if="status === 'refused'">
        <p class="sl-chrome-label text-sl-wax-lit mb-[8px]">Refused</p>
        <p class="font-folio text-[17px] leading-[26px] text-sl-ink mb-[6px]">
          The seal on that shipment does not verify.
        </p>
        <p class="sl-entry-sub text-sl-ink-soft mb-[20px]" data-testid="refused">
          The Ledger stands as it is — nothing was installed.
        </p>
        <div class="flex justify-end">
          <button
            class="sl-chrome-label text-sl-ink border-b-2 border-sl-lamp pb-[2px] focus-visible:outline focus-visible:outline-2 focus-visible:outline-sl-lamp"
            data-testid="stand-pat"
            @click="standPat()"
          >
            Noted
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
