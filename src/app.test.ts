import { beforeEach, describe, expect, it, vi } from "vitest";

import { startDockApp } from "./app";
import type { DockItem } from "./components/types";

const firstState: DockItem[] = [
  {
    bundleId: "com.apple.Safari",
    displayName: "Safari",
    iconSrc: "",
    isActive: false,
    isPinned: true,
    isRunning: true,
    path: "/Applications/Safari.app",
  },
];

const secondState: DockItem[] = [
  {
    bundleId: "com.apple.Terminal",
    displayName: "Terminal",
    iconSrc: "",
    isActive: true,
    isPinned: false,
    isRunning: true,
    path: "/System/Applications/Utilities/Terminal.app",
  },
];

describe("app refresh", () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="app"></div>';
  });

  it("replaces dock state without duplicate items", async () => {
    const root = document.querySelector<HTMLDivElement>("#app");
    const fetchDockState = vi
      .fn<() => Promise<DockItem[]>>()
      .mockResolvedValueOnce(firstState)
      .mockResolvedValueOnce(secondState);

    if (!root) {
      throw new Error("app root not found");
    }

    const app = await startDockApp(root, {
      fetchDockState,
      triggerLaunch: vi.fn(async () => undefined),
    });

    await app.refresh();

    const labels = Array.from(
      root.querySelectorAll<HTMLElement>(".dock-label"),
      (node) => node.textContent,
    );

    expect(labels).toEqual(["Terminal"]);
    expect(root.querySelectorAll("[data-dock-item]")).toHaveLength(1);
  });
});
