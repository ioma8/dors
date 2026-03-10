import "./styles.css";
import { renderDock } from "./components/dock";
import { fetchDockState, triggerLaunch } from "./lib/tauri";

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("App root not found");
}

void fetchDockState().then((items) => {
  renderDock(app, {
    items,
    onActivate: (item) => {
      void triggerLaunch(item);
    },
  });
});
