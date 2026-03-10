import { beforeEach, describe, expect, it, vi } from "vitest";

import { renderDock } from "./dock";
import type { DockItem } from "./types";

const dockItems: DockItem[] = [
  {
    bundleId: "com.apple.Safari",
    displayName: "Safari",
    iconSrc: "",
    isActive: false,
    isPinned: true,
    isRunning: true,
    path: "/Applications/Safari.app",
  },
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

describe("dock", () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="root"></div>';
  });

  it("renders pinned and running items in order with state indicators", () => {
    const root = document.querySelector<HTMLDivElement>("#root");

    if (!root) {
      throw new Error("root not found");
    }

    renderDock(root, {
      items: dockItems,
      onActivate: vi.fn(),
    });

    const buttons = Array.from(root.querySelectorAll<HTMLButtonElement>("[data-dock-item]"));

    expect(buttons).toHaveLength(2);
    expect(buttons.map((button) => button.dataset.name)).toEqual(["Safari", "Terminal"]);
    expect(buttons[0]?.dataset.pinned).toBe("true");
    expect(buttons[1]?.dataset.active).toBe("true");
    expect(root.querySelectorAll("[data-running-indicator='true']")).toHaveLength(2);
  });

  it("dispatches launch-or-activate clicks", async () => {
    const root = document.querySelector<HTMLDivElement>("#root");
    const onActivate = vi.fn();

    if (!root) {
      throw new Error("root not found");
    }

    renderDock(root, {
      items: dockItems,
      onActivate,
    });

    root.querySelector<HTMLButtonElement>("[data-dock-item='0']")?.click();

    expect(onActivate).toHaveBeenCalledWith(dockItems[0]);
  });

  it("renders an empty state when no pinned items import successfully", () => {
    const root = document.querySelector<HTMLDivElement>("#root");

    if (!root) {
      throw new Error("root not found");
    }

    renderDock(root, {
      items: [],
      onActivate: vi.fn(),
    });

    expect(root.textContent).toContain("No pinned apps");
  });
});
