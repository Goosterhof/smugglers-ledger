// useShipment — singleton state + plugin bridge for the auto-update flow.
//
// The Ledger renews itself, but never silently. Ported from the Mezzanine's
// Ascent (#00056) into the smuggler's register:
//
//   * checkShipment() — ask whether a newer edition has cleared the border.
//     Runs once on boot (App.vue); silent when nothing waits.
//   * takeDelivery() — the investor consented: stream the signed NSIS bundle,
//     let the plugin verify its minisign seal against the baked-in pubkey,
//     then relaunch into the new edition.
//   * standPat()    — the investor declined; the strip folds for the session
//     and the offer returns on next boot.
//
// The weight of the feature is config + CI + this slice; the Rust side is
// bare plugin registration. The `Update` resource returned by check() is held
// module-side so takeDelivery() acts on the exact bundle the check surfaced.

import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { computed, ref } from "vue";

export type ShipmentStatus =
  | "idle"
  | "checking"
  | "cleared" // a newer edition waits at the border
  | "crossing" // downloading + verifying
  | "refused" // the seal did not verify — a security event, never retried
  | "error";

const status = ref<ShipmentStatus>("idle");
const editionVersion = ref<string | null>(null);
const crossingPct = ref(0);
const lastError = ref<string | null>(null);

// Native plugin handle, not view state.
let activeUpdate: Update | null = null;

/** A seal failure is a security event, not a transient one: no retry. */
function isSealRejection(message: string): boolean {
  const lower = message.toLowerCase();
  return lower.includes("signature") || lower.includes("minisign") || lower.includes("verif");
}

async function checkShipment(): Promise<void> {
  if (status.value === "checking" || status.value === "crossing") return;
  lastError.value = null;
  status.value = "checking";
  try {
    const update = await check();
    if (update === null) {
      activeUpdate = null;
      editionVersion.value = null;
      status.value = "idle";
      return;
    }
    activeUpdate = update;
    editionVersion.value = update.version;
    status.value = "cleared";
  } catch (error) {
    activeUpdate = null;
    lastError.value = error instanceof Error ? error.message : String(error);
    // A failed boot check is not worth a voiced state — the Ledger works
    // offline by design; the offer simply returns next boot.
    status.value = "idle";
  }
}

async function takeDelivery(): Promise<void> {
  if (status.value !== "cleared" || activeUpdate === null) return;
  const update = activeUpdate;
  crossingPct.value = 0;
  lastError.value = null;
  status.value = "crossing";
  let contentLength = 0;
  let downloaded = 0;
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        contentLength = event.data.contentLength ?? 0;
        downloaded = 0;
        crossingPct.value = 0;
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        if (contentLength > 0) {
          crossingPct.value = Math.min(99, Math.round((downloaded / contentLength) * 100));
        }
      } else {
        crossingPct.value = 100;
      }
    });
    await relaunch();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    lastError.value = message;
    status.value = isSealRejection(message) ? "refused" : "error";
  }
}

function standPat(): void {
  activeUpdate = null;
  editionVersion.value = null;
  status.value = "idle";
}

export function useShipment() {
  return {
    status,
    editionVersion,
    crossingPct,
    lastError,

    /** The strip shows only when there is something to say. */
    visible: computed(
      (): boolean =>
        status.value === "cleared" || status.value === "crossing" || status.value === "refused",
    ),

    checkShipment,
    takeDelivery,
    standPat,

    _resetForTests(): void {
      status.value = "idle";
      editionVersion.value = null;
      crossingPct.value = 0;
      lastError.value = null;
      activeUpdate = null;
    },
  };
}
