import { describe, expect, it, vi } from "vitest";

const windowApi = vi.hoisted(() => ({
  getCurrentWindow: vi.fn(() => ({
    setTitle: vi.fn(),
    startDragging: vi.fn(),
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    isMaximized: vi.fn(async () => false),
  })),
}));

vi.mock("@tauri-apps/api/window", () => windowApi);

import {
  canUseTauriWindow,
  closeWindow,
  isWindowMaximized,
  minimizeWindow,
  setWindowTitle,
  startWindowDrag,
  toggleWindowMaximize,
} from "./window";

describe("window seam", () => {
  it("no-ops safely in browser mode", async () => {
    expect(canUseTauriWindow()).toBe(false);

    await expect(setWindowTitle("devapp")).resolves.toBeUndefined();
    await expect(startWindowDrag()).resolves.toBeUndefined();
    await expect(minimizeWindow()).resolves.toBeUndefined();
    await expect(toggleWindowMaximize()).resolves.toBeUndefined();
    await expect(closeWindow()).resolves.toBeUndefined();
    await expect(isWindowMaximized()).resolves.toBe(false);

    expect(windowApi.getCurrentWindow).not.toHaveBeenCalled();
  });
});
