// THE SHIPMENT contracts: silent when nothing waits at the border, the offer
// carries the edition and both choices, the crossing states its percentage,
// and a bad seal is REFUSED — voiced, never retried, nothing installed.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import { nextTick } from "vue";
import ShipmentPrompt from "@/shipment/ShipmentPrompt.vue";
import { useShipment } from "@/shipment/useShipment";
import { updaterCheckMock } from "./setup";

let wrapper: VueWrapper | undefined;

beforeEach(() => {
  useShipment()._resetForTests();
  updaterCheckMock.mockReset();
});

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
});

describe("THE SHIPMENT", () => {
  it("should stay silent when no newer edition waits", async () => {
    updaterCheckMock.mockResolvedValue(null);
    wrapper = mount(ShipmentPrompt);
    await useShipment().checkShipment();
    await nextTick();
    expect(wrapper.find("[data-testid='shipment-popup']").exists()).toBe(false);
  });

  it("should offer TAKE DELIVERY and STAND PAT when an edition clears the border", async () => {
    updaterCheckMock.mockResolvedValue({ version: "0.4.0", downloadAndInstall: async () => {} });
    wrapper = mount(ShipmentPrompt);
    await useShipment().checkShipment();
    await nextTick();
    const strip = wrapper.get("[data-testid='shipment-popup']");
    expect(strip.text()).toContain("cleared the border");
    expect(strip.text()).toContain("Version 0.4.0");
    expect(wrapper.find("[data-testid='take-delivery']").exists()).toBe(true);
    await wrapper.get("[data-testid='stand-pat']").trigger("click");
    await nextTick();
    expect(wrapper.find("[data-testid='shipment-popup']").exists()).toBe(false);
  });

  it("should REFUSE a shipment whose seal does not verify — voiced, nothing installed", async () => {
    updaterCheckMock.mockResolvedValue({
      version: "0.4.0",
      downloadAndInstall: () =>
        Promise.reject(new Error("the updater signature verification failed")),
    });
    wrapper = mount(ShipmentPrompt);
    const shipment = useShipment();
    await shipment.checkShipment();
    await shipment.takeDelivery();
    await nextTick();
    expect(shipment.status.value).toBe("refused");
    expect(wrapper.get("[data-testid='shipment-popup']").text()).toContain("does not verify");
    expect(wrapper.get("[data-testid='refused']").text()).toContain("nothing was installed");
  });

  it("should treat a failed boot check as silence, not a voiced state", async () => {
    updaterCheckMock.mockRejectedValue(new Error("offline"));
    wrapper = mount(ShipmentPrompt);
    await useShipment().checkShipment();
    await nextTick();
    expect(wrapper.find("[data-testid='shipment-popup']").exists()).toBe(false);
  });
});
