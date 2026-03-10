import { invoke } from "@tauri-apps/api/core";

import type { DockItem } from "../components/types";

type LaunchPayload = {
  bundle_id: string | null;
  path: string;
  is_running: boolean;
};

const fallbackItems: DockItem[] = [
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

function mapDockItem(input: {
  identity: { bundle_id: string | null; path: string };
  display_name: string;
  is_active: boolean;
  is_pinned: boolean;
  is_running: boolean;
}): DockItem {
  return {
    bundleId: input.identity.bundle_id,
    displayName: input.display_name,
    iconSrc: "",
    isActive: input.is_active,
    isPinned: input.is_pinned,
    isRunning: input.is_running,
    path: input.identity.path,
  };
}

export async function fetchDockState(): Promise<DockItem[]> {
  if (!("__TAURI_INTERNALS__" in window)) {
    return fallbackItems;
  }

  const items = await invoke<
    Array<{
      identity: { bundle_id: string | null; path: string };
      display_name: string;
      is_active: boolean;
      is_pinned: boolean;
      is_running: boolean;
    }>
  >("get_dock_state");

  return items.map(mapDockItem);
}

export async function triggerLaunch(item: DockItem): Promise<void> {
  if (!("__TAURI_INTERNALS__" in window)) {
    return;
  }

  const payload: LaunchPayload = {
    bundle_id: item.bundleId,
    path: item.path,
    is_running: item.isRunning,
  };

  await invoke("trigger_launch", { request: payload });
}
