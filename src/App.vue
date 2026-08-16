<script setup lang="ts">
// The two grounds (Design System #00012): the soot room you stand in, the
// lit folio you read. The lamp wash is the room's single light source —
// fractionally off the folio's centre, a light that comes from a place.
import { computed, onMounted } from "vue";
import TheWings from "@/components/TheWings.vue";
import TheMarginRail from "@/components/TheMarginRail.vue";
import TheSpine from "@/components/TheSpine.vue";
import LedgerSheet from "@/components/LedgerSheet.vue";
import EmptyShelf from "@/components/EmptyShelf.vue";
import LedgerPanel from "@/pages/LedgerPanel.vue";
import CharacterPanel from "@/pages/CharacterPanel.vue";
import StashPanel from "@/pages/StashPanel.vue";
import { useLedger } from "@/composables/useLedger";
import ShipmentPrompt from "@/shipment/ShipmentPrompt.vue";
import { useShipment } from "@/shipment/useShipment";

const { panel, state, chosenHand, characters } = useLedger();

// THE SHIPMENT's boot check — once, silent when nothing waits at the border.
const { checkShipment } = useShipment();
let shipmentChecked = false;
onMounted(() => {
  if (!shipmentChecked) {
    shipmentChecked = true;
    void checkShipment();
  }
});

const sheetTitle = computed(() => {
  if (panel.value === "manifest") {
    const hand = chosenHand.value ?? characters.value[0]?.name ?? "";
    return hand === "" ? "THE MANIFEST" : `THE MANIFEST — ${hand}`;
  }
  if (panel.value === "warehouse") return "THE WAREHOUSE";
  return "THE HOARD";
});

// The two waiting states and the two empty-machine states replace the whole
// sheet; noResults and wontTurn are panel-level and render inside the panels.
const sheetState = computed(() => {
  if (state.value === "firstRun" || state.value === "noSaves" || state.value === "coldCodex") {
    return state.value;
  }
  return null;
});
</script>

<template>
  <div class="min-h-screen bg-sl-soot text-sl-chalk relative overflow-hidden flex flex-col">
    <div
      class="absolute inset-0 pointer-events-none"
      :style="{ background: 'var(--sl-lamp-wash)' }"
      aria-hidden="true"
    />
    <TheWings />
    <div class="relative flex flex-1 gap-[24px] px-[24px] pb-[24px] min-h-0">
      <TheMarginRail />
      <LedgerSheet class="flex-1 min-w-0" :title="sheetTitle">
        <EmptyShelf v-if="sheetState" :state="sheetState" />
        <template v-else>
          <LedgerPanel v-if="panel === 'hoard'" />
          <CharacterPanel v-else-if="panel === 'manifest'" />
          <StashPanel v-else />
        </template>
        <EmptyShelf v-if="state === 'noInstall'" state="noInstall" muted />
      </LedgerSheet>
      <TheSpine />
    </div>
    <ShipmentPrompt />
  </div>
</template>
