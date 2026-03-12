# Window Clamping Above Custom Dock Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the failed Dock-interception hacks with a simple autohide Dock flow and a native window-clamping loop that keeps normal app windows above the custom dock.

**Architecture:** Revert the blocker/event-tap path, simplify `system_dock` to autohide/restore only, and add a new `window_clamper` native adapter with pure geometry helpers plus a macOS window-clamping runtime hook on the existing refresh loop.

**Tech Stack:** Rust, AppKit/CoreGraphics on macOS, existing native `dors` timer/refresh architecture, cargo test/check.

---

### Task 1: Revert experimental Dock-blocking runtime

**Files:**
- Modify: `src/native_app/app.rs`
- Modify: `src/native_app/mod.rs`
- Modify: `src/native_app/panel.rs`
- Delete: `src/native_app/event_blocker.rs`
- Delete: `src/bin/dock_watch.rs`
- Modify: `tests/native_panel.rs`
- Delete: `tests/native_event_blocker.rs`

**Step 1: Write the failing test**

Update `tests/native_panel.rs` so only the overlay panel configuration remains expected. Remove blocker-specific expectations and keep the panel-level contract explicit.

**Step 2: Run test to verify it fails**

Run: `cargo test --test native_panel -- --nocapture`
Expected: FAIL because blocker panel symbols/config are still present.

**Step 3: Write minimal implementation**

Remove blocker panel and event blocker wiring from the runtime and module tree. Keep only the overlay panel builder/configuration.

**Step 4: Run test to verify it passes**

Run: `cargo test --test native_panel -- --nocapture`
Expected: PASS

**Step 5: Verify crate validity**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/native_app/app.rs src/native_app/mod.rs src/native_app/panel.rs tests/native_panel.rs
git rm src/native_app/event_blocker.rs src/bin/dock_watch.rs tests/native_event_blocker.rs
git commit -m "refactor: remove dock blocking experiments"
```

### Task 2: Simplify Dock control to autohide-only behavior

**Files:**
- Modify: `src/native_app/system_dock.rs`
- Modify: `tests/system_dock.rs`

**Step 1: Write the failing test**

Add tests that assert:
- startup target autohide is `true`
- tilesize is no longer part of the startup plan
- restore behavior only depends on previous autohide state

**Step 2: Run test to verify it fails**

Run: `cargo test --test system_dock -- --nocapture`
Expected: FAIL because the current code still carries tilesize startup logic.

**Step 3: Write minimal implementation**

Refactor `system_dock` so startup prepares `autohide=true` and restores only the preferences that were actually changed. Keep tolerant Dock restart handling and signal restoration.

**Step 4: Run test to verify it passes**

Run: `cargo test --test system_dock -- --nocapture`
Expected: PASS

**Step 5: Verify crate validity**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/native_app/system_dock.rs tests/system_dock.rs
git commit -m "refactor: simplify dock autohide control"
```

### Task 3: Add pure window-clamping geometry

**Files:**
- Create: `src/native_app/window_clamper.rs`
- Create: `tests/window_clamper.rs`
- Modify: `src/native_app/mod.rs`

**Step 1: Write the failing test**

Add tests covering:
- allowed work area from screen frame, top reserve, and dock height
- moving a window upward when only the bottom overflows
- shrinking a window when it is taller than allowed
- leaving compliant windows unchanged

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_clamper -- --nocapture`
Expected: FAIL because the module does not exist yet.

**Step 3: Write minimal implementation**

Implement pure structs/helpers for:
- screen bounds
- allowed work area
- clamp result
- `clamp_window_frame(...)`

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_clamper -- --nocapture`
Expected: PASS

**Step 5: Verify crate validity**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/native_app/window_clamper.rs src/native_app/mod.rs tests/window_clamper.rs
git commit -m "feat: add window clamping geometry"
```

### Task 4: Add macOS window candidate filtering and clamping adapter

**Files:**
- Modify: `src/native_app/window_clamper.rs`
- Modify: `tests/window_clamper.rs`

**Step 1: Write the failing test**

Add tests for filtering helpers that decide whether a discovered window should be clamped:
- normal visible resizable window -> yes
- fullscreen window -> no
- non-resizable/system overlay/tiny transient -> no

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_clamper -- --nocapture`
Expected: FAIL because filtering helpers are not implemented.

**Step 3: Write minimal implementation**

Add macOS-facing candidate types and pure filter predicates. Implement runtime adapter code that enumerates windows and applies frame clamping conservatively.

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_clamper -- --nocapture`
Expected: PASS

**Step 5: Verify crate validity**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/native_app/window_clamper.rs tests/window_clamper.rs
git commit -m "feat: add native window clamp adapter"
```

### Task 5: Wire window clamping into the native refresh loop

**Files:**
- Modify: `src/native_app/app.rs`
- Modify: `src/native_app/interaction.rs`
- Modify: `src/native_app/refresh.rs`
- Modify: `tests/native_refresh.rs`

**Step 1: Write the failing test**

Add/adjust a refresh test to assert the refresh path invokes window clamping alongside dock-state refresh without panicking when no windows need changes.

**Step 2: Run test to verify it fails**

Run: `cargo test --test native_refresh -- --nocapture`
Expected: FAIL because the refresh path does not yet include clamping.

**Step 3: Write minimal implementation**

Invoke the window clamper from the periodic native refresh flow using the main screen frame and current dock height.

**Step 4: Run test to verify it passes**

Run: `cargo test --test native_refresh -- --nocapture`
Expected: PASS

**Step 5: Verify crate validity**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/native_app/app.rs src/native_app/interaction.rs src/native_app/refresh.rs tests/native_refresh.rs
git commit -m "feat: clamp windows above custom dock"
```

### Task 6: Final quality gate

**Files:**
- Verify current working tree only

**Step 1: Run formatting**

Run: `cargo fmt --all`

**Step 2: Run strict linting**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic`

**Step 3: Run full test suite**

Run: `cargo test --workspace --all-features`

**Step 4: Run final compile check**

Run: `cargo check`

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: finalize window clamping runtime"
```
