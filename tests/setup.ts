// Global Tauri mocks — the component tests run against a mocked bridge, not
// a live spine. Individual specs steer `invokeMock` per scenario.
import { vi } from "vitest";

export const invokeMock = vi.fn<(...args: unknown[]) => unknown>();
export const listenMock = vi.fn<(...args: unknown[]) => Promise<() => void>>(() =>
  Promise.resolve(() => undefined),
);
export const openMock = vi.fn<(...args: unknown[]) => unknown>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: unknown[]) => openMock(...args),
}));

export const updaterCheckMock = vi.fn<() => Promise<unknown>>(() => Promise.resolve(null));
export const relaunchMock = vi.fn<() => Promise<void>>(() => Promise.resolve());

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: () => updaterCheckMock(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: () => relaunchMock(),
}));
