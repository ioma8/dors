# Dock Hide Experiment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a separate experimental binary that tries to make the real macOS Dock visually disappear while keeping Dock alive and reserving work area.

**Architecture:** Create a macOS-only private adapter that enumerates Dock-owned windows and applies a reversible visual suppression attempt from our own process. Keep the experiment isolated from the main `dors` runtime and restore on exit/signals when possible.

**Tech Stack:** Rust, AppKit/Foundation bindings already in repo, private macOS WindowServer/SkyLight FFI, `ctrlc`

---

### Task 1: Add experiment module skeleton

**Files:**
- Create: `src/private_dock_experiment.rs`
- Modify: `src/lib.rs`
- Test: `tests/private_dock_experiment.rs`

**Step 1: Write the failing test**

Add a pure test for Dock candidate filtering / suppression-plan selection.

**Step 2: Run test to verify it fails**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: FAIL because the module/functions do not exist yet.

**Step 3: Write minimal implementation**

Add:

- `DockVisualCandidate`
- pure filter helpers
- minimal `pub mod private_dock_experiment;` export

**Step 4: Run test to verify it passes**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 2: Add private WindowServer bindings needed for the experiment

**Files:**
- Modify: `src/private_dock_experiment.rs`
- Test: `tests/private_dock_experiment.rs`

**Step 1: Write the failing test**

Add pure tests for suppression-plan creation and restoration behavior using fake window ids and fake state snapshots.

**Step 2: Run test to verify it fails**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: FAIL because plan/state types do not exist yet.

**Step 3: Write minimal implementation**

Add:

- private FFI declarations for the first chosen visual API
- pure suppression-plan structs
- snapshot / restore plan helpers

Do not wire the runtime yet.

**Step 4: Run test to verify it passes**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 3: Add runtime query for Dock-owned candidate windows

**Files:**
- Modify: `src/private_dock_experiment.rs`
- Test: `tests/private_dock_experiment.rs`

**Step 1: Write the failing test**

Add a pure test for converting raw candidate rows into chosen Dock suppression targets.

**Step 2: Run test to verify it fails**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

Add runtime query helpers that:

- enumerate current windows
- identify Dock / WindowServer candidates relevant to the Dock strip
- build a suppression target list

**Step 4: Run test to verify it passes**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 4: Add suppression and restoration runtime

**Files:**
- Modify: `src/private_dock_experiment.rs`
- Test: `tests/private_dock_experiment.rs`

**Step 1: Write the failing test**

Add a pure test for restoration planning when some windows have no restorable snapshot.

**Step 2: Run test to verify it fails**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

Add runtime functions that:

- apply the selected private suppression strategy
- retain a restoration guard
- restore on drop / explicit restore

**Step 4: Run test to verify it passes**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 5: Add experimental binary

**Files:**
- Create: `src/bin/dock_hide_experiment.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing integration shape**

Add the new binary target and minimal main function using the experiment module API.

**Step 2: Run build to verify it fails if API is incomplete**

Run: `cargo check`
Expected: FAIL if runtime API is not yet complete.

**Step 3: Write minimal implementation**

Make the binary:

- start the experiment
- print a short status line
- wait for Ctrl-C
- restore state on exit

**Step 4: Run validation**

Run: `cargo check`
Expected: PASS

### Task 6: Manual smoke verification and cleanup

**Files:**
- Modify: `src/private_dock_experiment.rs` as needed

**Step 1: Run focused tests**

Run: `cargo test --test private_dock_experiment -- --nocapture`
Expected: PASS

**Step 2: Run final validation**

Run: `cargo check`
Expected: PASS

**Step 3: Manual smoke test**

Run: `cargo run --bin dock_hide_experiment`

Verify:

- Dock remains alive
- Dock work area remains reserved
- Dock becomes visually absent or near-invisible
- exiting the binary restores state, or clearly documents that Dock restart is required

**Step 4: Commit**

```bash
git add Cargo.toml src/lib.rs src/private_dock_experiment.rs src/bin/dock_hide_experiment.rs tests/private_dock_experiment.rs
git commit -m "feat: add dock hide experiment"
```
