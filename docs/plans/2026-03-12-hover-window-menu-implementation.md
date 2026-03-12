# Hover Window Menu Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show a delayed native window menu above multi-window app icons and activate the selected window.

**Architecture:** Add a `window_menu` native module for hover delay, window discovery, menu building, and specific-window activation, then wire it into dock item hover handling while preserving existing click activation.

**Tech Stack:** Rust, AppKit `NSMenu`, existing native dock controller, cargo test/check.

---

### Task 1: Add pure window-menu models and activation-script helpers

**Files:**
- Create: `src/native_app/window_menu.rs`
- Create: `tests/window_menu.rs`
- Modify: `src/native_app/mod.rs`

**Step 1: Write the failing test**

Add tests for:
- only showing menu when window count is greater than one
- filtering empty window titles
- building the specific-window activation script

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_menu -- --nocapture`

**Step 3: Write minimal implementation**

Add pure menu model helpers and script builders.

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_menu -- --nocapture`

**Step 5: Verify crate validity**

Run: `cargo check`

**Step 6: Commit**

```bash
git add src/native_app/window_menu.rs src/native_app/mod.rs tests/window_menu.rs
git commit -m "feat: add window menu models"
```

### Task 2: Add delayed hover scheduler

**Files:**
- Modify: `src/native_app/window_menu.rs`
- Modify: `tests/window_menu.rs`

**Step 1: Write the failing test**

Add tests for:
- hover delay default
- cancellation/versioning behavior so stale hover requests do not open menus

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_menu -- --nocapture`

**Step 3: Write minimal implementation**

Add a tiny hover token/scheduler abstraction for delayed menu open decisions.

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_menu -- --nocapture`

**Step 5: Verify crate validity**

Run: `cargo check`

**Step 6: Commit**

```bash
git add src/native_app/window_menu.rs tests/window_menu.rs
git commit -m "feat: add hover menu delay state"
```

### Task 3: Add native window discovery and exact-window activation

**Files:**
- Modify: `src/native_app/window_menu.rs`
- Modify: `tests/window_menu.rs`

**Step 1: Write the failing test**

Add tests for parsing/filtering discovered windows and exact-window activation script generation.

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_menu -- --nocapture`

**Step 3: Write minimal implementation**

Implement the macOS adapter that:
- discovers app windows by process name
- builds a specific-window activation script
- falls back cleanly when needed

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_menu -- --nocapture`

**Step 5: Verify crate validity**

Run: `cargo check`

**Step 6: Commit**

```bash
git add src/native_app/window_menu.rs tests/window_menu.rs
git commit -m "feat: add native window menu discovery"
```

### Task 4: Wire hover events into the dock controller and popup menu

**Files:**
- Modify: `src/native_app/dock_item_view.rs`
- Modify: `src/native_app/interaction.rs`
- Modify: `tests/native_interaction.rs`

**Step 1: Write the failing test**

Add a focused test for the controller hover path:
- hover over multi-window app schedules menu logic
- single-window app does not request menu open

**Step 2: Run test to verify it fails**

Run: `cargo test --test native_interaction -- --nocapture`

**Step 3: Write minimal implementation**

Wire `mouseEntered` / `mouseExited` to controller methods and show the native popup menu above the hovered icon when the delay completes.

**Step 4: Run test to verify it passes**

Run: `cargo test --test native_interaction -- --nocapture`

**Step 5: Verify crate validity**

Run: `cargo check`

**Step 6: Commit**

```bash
git add src/native_app/dock_item_view.rs src/native_app/interaction.rs tests/native_interaction.rs
git commit -m "feat: show hover window menu"
```

### Task 5: Final focused verification

**Files:**
- Verify current working tree only

**Step 1: Run targeted tests**

Run:
- `cargo test --test window_menu -- --nocapture`
- `cargo test --test native_interaction -- --nocapture`

**Step 2: Run compile check**

Run: `cargo check`

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: finalize hover window menu"
```
