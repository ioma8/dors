# Dock Injection Experiment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a minimal Dock injection experiment that attempts to load code into `Dock.app` and hide the Dock visually while leaving the Dock process alive.

**Architecture:** Build an isolated experiment consisting of a minimal payload, a loader/deployer path, and a dedicated experiment binary. Keep all code out of the main `dors` runtime and use restart-based restoration after the experiment exits.

**Tech Stack:** Rust, macOS private APIs, experiment-only binaries, manual macOS verification

---

### Task 1: Scaffold experiment module tree

**Files:**
- Create: `src/dock_injection_experiment/mod.rs`
- Create: `tests/dock_injection_experiment.rs`
- Modify: `src/lib.rs`

**Step 1: Write the failing test**

Add a pure test for experiment configuration / path planning.

**Step 2: Run test to verify it fails**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: FAIL because the module does not exist.

**Step 3: Write minimal implementation**

Add experiment-only types for:

- staging paths
- experiment naming
- restore strategy enum

**Step 4: Run test to verify it passes**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 2: Add loader/payload file planning

**Files:**
- Modify: `src/dock_injection_experiment/mod.rs`
- Test: `tests/dock_injection_experiment.rs`

**Step 1: Write the failing test**

Add tests for path generation and packaging metadata generation.

**Step 2: Run test to verify it fails**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

Add pure helpers for:

- payload bundle paths
- loader paths
- temporary staging directories
- expected restore behavior description

**Step 4: Run test to verify it passes**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 3: Add dedicated experiment binary shell

**Files:**
- Create: `src/bin/dock_inject_experiment.rs`
- Modify: `Cargo.toml`

**Step 1: Write minimal failing shell**

Create the binary target and wire it to the experiment module API.

**Step 2: Run validation**

Run: `cargo check`
Expected: PASS

**Step 3: Print environment/runtime prerequisites**

Make the binary print:

- what it will attempt
- that SIP / Dock protections may block it
- that restore is by restarting Dock

**Step 4: Run validation**

Run: `cargo check`
Expected: PASS

### Task 4: Add minimal payload/loader staging output

**Files:**
- Modify: `src/dock_injection_experiment/mod.rs`
- Test: `tests/dock_injection_experiment.rs`

**Step 1: Write the failing test**

Add a test that the experiment can materialize the expected staged files.

**Step 2: Run test to verify it fails**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

Create staging/materialization logic for:

- payload placeholder
- loader placeholder
- metadata/plist text

This task is still non-invasive: just file generation and planning.

**Step 4: Run test to verify it passes**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: PASS

**Step 5: Run validation**

Run: `cargo check`
Expected: PASS

### Task 5: Add manual execution hooks and logging

**Files:**
- Modify: `src/bin/dock_inject_experiment.rs`
- Modify: `src/dock_injection_experiment/mod.rs`

**Step 1: Implement manual experiment commands**

Add a mode that:

- stages the experiment files
- prints the exact privileged/manual commands the user must run
- explains exactly what logs/results to capture

**Step 2: Run validation**

Run: `cargo check`
Expected: PASS

### Task 6: Final verification and handoff

**Files:**
- Modify: experiment files as needed

**Step 1: Run focused tests**

Run: `cargo test --test dock_injection_experiment -- --nocapture`
Expected: PASS

**Step 2: Run final validation**

Run: `cargo check`
Expected: PASS

**Step 3: Manual experiment**

Run:

```bash
cargo run --bin dock_inject_experiment
```

Follow the printed steps, then capture:

- loader output
- payload output if available
- whether Dock became visually hidden
- whether restarting Dock restored the system

**Step 4: Commit**

```bash
git add Cargo.toml src/lib.rs src/dock_injection_experiment/mod.rs src/bin/dock_inject_experiment.rs tests/dock_injection_experiment.rs
git commit -m "feat: add dock injection experiment"
```
