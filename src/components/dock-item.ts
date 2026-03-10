import type { DockItem } from "./types";

type DockItemOptions = {
  index: number;
  item: DockItem;
  onActivate: (item: DockItem) => void;
};

export function createDockItem({ index, item, onActivate }: DockItemOptions): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "dock-item";
  button.dataset.dockItem = String(index);
  button.dataset.name = item.displayName;
  button.dataset.active = String(item.isActive);
  button.dataset.pinned = String(item.isPinned);
  button.dataset.running = String(item.isRunning);
  button.setAttribute("aria-label", item.displayName);

  const icon = document.createElement("span");
  icon.className = "dock-icon";
  icon.textContent = item.displayName.slice(0, 1).toUpperCase();
  button.append(icon);

  const label = document.createElement("span");
  label.className = "dock-label";
  label.textContent = item.displayName;
  button.append(label);

  const indicator = document.createElement("span");
  indicator.className = "dock-indicator";
  indicator.dataset.runningIndicator = String(item.isRunning);
  button.append(indicator);

  button.addEventListener("click", () => {
    onActivate(item);
  });

  return button;
}
