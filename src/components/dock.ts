import { createDockItem } from "./dock-item";
import type { DockItem } from "./types";

type RenderDockOptions = {
  items: DockItem[];
  onActivate: (item: DockItem) => void;
};

export function renderDock(root: HTMLElement, { items, onActivate }: RenderDockOptions): void {
  root.replaceChildren();

  const shell = document.createElement("main");
  shell.className = "dock-shell";

  const panel = document.createElement("section");
  panel.className = "dock-panel";
  shell.append(panel);

  const title = document.createElement("div");
  title.className = "dock-title";
  title.innerHTML =
    '<p class="dock-kicker">dors</p><h1>Living dock</h1><p class="dock-copy">Pinned imports first, active apps glowing above the shelf.</p>';
  panel.append(title);

  if (items.length === 0) {
    const empty = document.createElement("div");
    empty.className = "dock-empty";
    empty.textContent = "No pinned apps were imported yet.";
    panel.append(empty);
    root.append(shell);
    return;
  }

  const rail = document.createElement("div");
  rail.className = "dock-rail";

  items.forEach((item, index) => {
    rail.append(createDockItem({ index, item, onActivate }));
  });

  panel.append(rail);
  root.append(shell);
}
