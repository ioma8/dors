import "./styles.css";

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("App root not found");
}

app.innerHTML = `
  <main class="shell">
    <section class="dock-frame">
      <p class="eyebrow">dors</p>
      <h1>macOS dock replacement</h1>
      <p class="lede">
        First-run dock import, running app detection, and launch-or-activate
        flows will be wired in the next tasks.
      </p>
    </section>
  </main>
`;
