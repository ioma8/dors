# AX Window Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace timer-driven window zoom/clamp handling with an Accessibility observer based manager that reliably toggles between regular frame and custom work area for normal resizable windows.

**Architecture:** Add a dedicated AX window manager adapter that emits window geometry events into the existing pure managed zoom state machine. Keep the dock UI refresh timer, but remove periodic geometry polling as the primary control path.

**Tech Stack:** Rust, AppKit, Accessibility APIs (`AXObserver`, `AXUIElement`), existing native app runtime and pure window-clamper tests.

---

### Task 1: Introduce AX event model

**Files:**
- Create: `src/native_app/ax_window_manager.rs`
- Modify: `src/native_app/mod.rs`
- Test: `tests/ax_window_manager.rs`

**Step 1: Write the failing test**

Add pure tests for a small event model:

- `WindowEvent::Focused`
- `WindowEvent::Moved`
- `WindowEvent::Resized`

Include a test that maps native notification names into the internal event enum.

**Step 2: Run test to verify it fails**

Run: `cargo test --test ax_window_manager -- --nocapture`

Expected: FAIL because the new module and types do not exist yet.

**Step 3: Write minimal implementation**

Create `src/native_app/ax_window_manager.rs` with:

- `WindowEventKind`
- `ObservedWindowId`
- notification-name normalization helper

Expose the module from `src/native_app/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test ax_window_manager -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/native_app/ax_window_manager.rs src/native_app/mod.rs tests/ax_window_manager.rs
git commit -m "feat: add ax window event model"
```

### Task 2: Extract pure managed zoom reducer

**Files:**
- Modify: `src/native_app/window_clamper.rs`
- Test: `tests/window_clamper.rs`

**Step 1: Write the failing test**

Add tests that drive the state machine from explicit event-style frame transitions instead of timer sampling:

- regular -> native zoom -> custom zoom
- custom zoom -> native zoom -> restore
- manual resize after custom zoom clears managed state

**Step 2: Run test to verify it fails**

Run: `cargo test --test window_clamper -- --nocapture`

Expected: FAIL because the reducer API does not yet exist.

**Step 3: Write minimal implementation**

Refactor `CustomZoomTracker` to expose a pure event-driven reducer that:

- accepts current frame transitions
- returns optional geometry operation
- keeps regular/managed state

Keep old helpers only if needed temporarily.

**Step 4: Run test to verify it passes**

Run: `cargo test --test window_clamper -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/native_app/window_clamper.rs tests/window_clamper.rs
git commit -m "refactor: extract managed zoom reducer"
```

### Task 3: Implement AX observer registration

**Files:**
- Modify: `src/native_app/ax_window_manager.rs`
- Test: `tests/ax_window_manager.rs`

**Step 1: Write the failing test**

Add tests for observer bookkeeping:

- registering an app observer once
- tracking known observed apps
- removing dead observers cleanly

**Step 2: Run test to verify it fails**

Run: `cargo test --test ax_window_manager -- --nocapture`

Expected: FAIL because observer bookkeeping is incomplete.

**Step 3: Write minimal implementation**

Add:

- observer registry
- app PID tracking
- notification subscription bookkeeping

Keep AX callbacks thin; only normalize and dispatch.

**Step 4: Run test to verify it passes**

Run: `cargo test --test ax_window_manager -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/native_app/ax_window_manager.rs tests/ax_window_manager.rs
git commit -m "feat: add ax observer registration"
```

### Task 4: Wire AX events into geometry operations

**Files:**
- Modify: `src/native_app/ax_window_manager.rs`
- Modify: `src/native_app/interaction.rs`
- Modify: `src/native_app/app.rs`
- Test: `tests/ax_window_manager.rs`
- Test: `tests/window_clamper.rs`

**Step 1: Write the failing test**

Add a test that simulates:

- focused window regular frame
- native zoom event
- expected custom work area resize
- second native zoom event
- expected restore frame

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test ax_window_manager -- --nocapture
cargo test --test window_clamper -- --nocapture
```

Expected: FAIL because runtime wiring is not yet present.

**Step 3: Write minimal implementation**

In runtime:

- create/start AX manager at startup
- feed events to the reducer
- apply returned geometry changes immediately

Remove the fast timer-driven geometry loop from `interaction.rs`.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test ax_window_manager -- --nocapture
cargo test --test window_clamper -- --nocapture
cargo check
```

Expected: PASS

**Step 5: Commit**

```bash
git add src/native_app/ax_window_manager.rs src/native_app/interaction.rs src/native_app/app.rs tests/ax_window_manager.rs tests/window_clamper.rs
git commit -m "feat: drive managed zoom from ax events"
```

### Task 5: Remove timer-based geometry path

**Files:**
- Modify: `src/native_app/interaction.rs`
- Modify: `src/native_app/app.rs`
- Modify: `src/native_app/window_clamper.rs`
- Test: `tests/clamp_scheduler.rs`

**Step 1: Write the failing test**

Add or update tests to reflect that:

- geometry work is no longer scheduled from the old fast timer
- only dock UI refresh remains timer-driven

**Step 2: Run test to verify it fails**

Run: `cargo test --test clamp_scheduler -- --nocapture`

Expected: FAIL because the old scheduling path still exists.

**Step 3: Write minimal implementation**

Delete:

- fast window-management timer path
- timer-based capture path
- dead scheduler wiring used only for geometry polling

Keep any remaining scheduler only if still needed elsewhere.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test --test clamp_scheduler -- --nocapture
cargo check
```

Expected: PASS

**Step 5: Commit**

```bash
git add src/native_app/interaction.rs src/native_app/app.rs src/native_app/window_clamper.rs tests/clamp_scheduler.rs
git commit -m "refactor: remove timer-driven geometry management"
```

### Task 6: Final verification and manual checklist update

**Files:**
- Modify: `docs/plans/2026-03-11-native-appkit-manual-checklist.md`

**Step 1: Update manual checklist**

Add explicit checks for:

- first title-bar double-click -> custom work area
- second title-bar double-click -> restore prior frame
- manual resize after custom zoom establishes new restore frame
- behavior on IntelliJ and Firefox

**Step 2: Run final verification**

Run:

```bash
cargo check
cargo test --test ax_window_manager -- --nocapture
cargo test --test window_clamper -- --nocapture
cargo test --test clamp_scheduler -- --nocapture
cargo test
```

Expected: all pass

**Step 3: Commit**

```bash
git add docs/plans/2026-03-11-native-appkit-manual-checklist.md
git commit -m "chore: finalize ax window manager integration"
```
