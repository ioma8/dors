import "./styles.css";
import { renderDock } from "./components/dock";
import type { DockItem } from "./components/types";
import { fetchDockState, triggerLaunch } from "./lib/tauri";

type DockAppDependencies = {
  fetchDockState: () => Promise<DockItem[]>;
  triggerLaunch: (item: DockItem) => Promise<void>;
};

type DockAppController = {
  refresh: () => Promise<void>;
};

const REFRESH_INTERVAL_MS = 5000;

export async function startDockApp(
  root: HTMLElement,
  dependencies: DockAppDependencies = {
    fetchDockState,
    triggerLaunch,
  },
): Promise<DockAppController> {
  const render = async (): Promise<void> => {
    const items = await dependencies.fetchDockState();

    renderDock(root, {
      items,
      onActivate: (item) => {
        void dependencies.triggerLaunch(item);
      },
    });
  };

  await render();

  if ("setInterval" in window) {
    window.setInterval(() => {
      void render();
    }, REFRESH_INTERVAL_MS);
  }

  return {
    refresh: render,
  };
}

const app = document.querySelector<HTMLDivElement>("#app");

if (app) {
  void startDockApp(app);
}
