# Native AppKit Dock Rewrite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Tauri and webview shell with a native macOS AppKit shell in Rust while preserving the existing Rust dock import, config, running-app discovery, merge, icon, and launch logic.

**Architecture:** Keep the current backend modules as the application core and introduce a `native_app` module tree that owns `NSApplication`, the overlay panel, native dock views, and periodic refresh. Remove Tauri runtime dependencies only after the native panel can bootstrap, render real dock items, and handle launch and activate directly.

**Tech Stack:** Rust 2024, `objc2`, `objc2-app-kit`, `objc2-foundation`, existing serde/plist/base64/thiserror backend modules, macOS AppKit APIs

---

### Task 1: Stabilize The Shared Core

**Files:**
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/domain/dock_item.rs`
- Modify: `src-tauri/src/services/dock_state.rs`
- Create: `src-tauri/tests/app_state_core.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn app_state_refresh_returns_stable_items_for_native_shell() {
    // Build AppState from pinned config, refresh with running apps, assert
    // pinned-first ordering and active/running flags survive without Tauri.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test app_state_core -- --nocapture`
Expected: FAIL because the native-shell-facing helper does not exist yet.

**Step 3: Write minimal implementation**

- Keep `AppState` free of Tauri types.
- Add any small helper needed by the native shell to fetch and refresh state safely.
- Keep `DockItemView` as the normalized UI item model.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test app_state_core -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/app_state.rs src-tauri/src/domain/dock_item.rs src-tauri/src/services/dock_state.rs src-tauri/tests/app_state_core.rs
git commit -m "refactor: stabilize native shell app state"
```

### Task 2: Add Native Placement And View-Model Diff Helpers

**Files:**
- Create: `src-tauri/src/native_app/view_model.rs`
- Create: `src-tauri/src/native_app/layout.rs`
- Modify: `src-tauri/src/native_app/mod.rs`
- Create: `src-tauri/tests/native_layout.rs`
- Create: `src-tauri/tests/native_view_model.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn bottom_center_panel_anchors_to_full_monitor_bounds() {}

#[test]
fn reconcile_items_detects_insert_remove_and_update() {}
```

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_layout -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_view_model -- --nocapture`

Expected: FAIL because the native modules do not exist yet.

**Step 3: Write minimal implementation**

- Move placement math into a native-focused helper.
- Add a small diff/reconciliation helper for native item views keyed by bundle ID or path.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_layout -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_view_model -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/native_app src-tauri/tests/native_layout.rs src-tauri/tests/native_view_model.rs
git commit -m "feat: add native dock layout helpers"
```

### Task 3: Bootstrap A Minimal Native App

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`
- Create: `src-tauri/src/native_app/app.rs`
- Create: `src-tauri/src/native_app/app_delegate.rs`
- Modify: `src-tauri/src/native_app/mod.rs`
- Modify: `src-tauri/tests/native_layout.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn native_app_builds_startup_configuration() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test native_layout -- --nocapture`
Expected: FAIL because native startup configuration helpers are missing.

**Step 3: Write minimal implementation**

- Introduce a native entry point that no longer calls `dors_tauri_lib::run()`.
- Build `NSApplication` startup configuration on the main thread.
- Keep the runtime thin and defer actual dock rendering to later tasks.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_layout -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs src-tauri/src/native_app
git commit -m "feat: bootstrap native appkit app"
```

### Task 4: Create The Overlay Panel

**Files:**
- Create: `src-tauri/src/native_app/panel.rs`
- Modify: `src-tauri/src/native_app/app.rs`
- Modify: `src-tauri/src/native_app/layout.rs`
- Create: `src-tauri/tests/native_panel.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn panel_configuration_uses_bottom_overlay_window_semantics() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test native_panel -- --nocapture`
Expected: FAIL because panel configuration is not implemented.

**Step 3: Write minimal implementation**

- Build a transparent borderless panel.
- Apply level above the Dock.
- Apply non-activating semantics where AppKit allows it.
- Use full monitor bounds for bottom-edge placement.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_panel -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/native_app/panel.rs src-tauri/src/native_app/app.rs src-tauri/src/native_app/layout.rs src-tauri/tests/native_panel.rs
git commit -m "feat: add native dock overlay panel"
```

### Task 5: Render Native Dock Items

**Files:**
- Create: `src-tauri/src/native_app/dock_view.rs`
- Create: `src-tauri/src/native_app/dock_item_view.rs`
- Modify: `src-tauri/src/native_app/view_model.rs`
- Modify: `src-tauri/src/native_app/app.rs`
- Create: `src-tauri/tests/native_view_model.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn view_model_maps_dock_item_state_to_native_item_state() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test native_view_model -- --nocapture`
Expected: FAIL because native item rendering state is incomplete.

**Step 3: Write minimal implementation**

- Map `DockItemView` into native view models.
- Render one native item view per dock item.
- Show real icons when available and placeholders otherwise.
- Add running and active indicators.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_view_model -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/native_app/dock_view.rs src-tauri/src/native_app/dock_item_view.rs src-tauri/src/native_app/view_model.rs src-tauri/src/native_app/app.rs src-tauri/tests/native_view_model.rs
git commit -m "feat: render native dock items"
```

### Task 6: Wire Startup Data And Refresh

**Files:**
- Create: `src-tauri/src/native_app/refresh.rs`
- Modify: `src-tauri/src/native_app/app.rs`
- Modify: `src-tauri/src/adapters/icon_loader.rs`
- Modify: `src-tauri/src/app_state.rs`
- Create: `src-tauri/tests/native_refresh.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn refresh_rebuilds_native_items_from_running_apps() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test native_refresh -- --nocapture`
Expected: FAIL because native refresh orchestration is missing.

**Step 3: Write minimal implementation**

- Bootstrap config using the existing first-run import flow.
- Read running apps on startup.
- Schedule periodic polling refresh.
- Diff and update native views instead of recreating the panel every tick.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_refresh -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/native_app/refresh.rs src-tauri/src/native_app/app.rs src-tauri/src/adapters/icon_loader.rs src-tauri/src/app_state.rs src-tauri/tests/native_refresh.rs
git commit -m "feat: add native dock refresh loop"
```

### Task 7: Wire Native Hover And Click Activation

**Files:**
- Modify: `src-tauri/src/native_app/dock_item_view.rs`
- Modify: `src-tauri/src/native_app/dock_view.rs`
- Modify: `src-tauri/src/services/launcher.rs`
- Create: `src-tauri/tests/native_interaction.rs`
- Modify: `src-tauri/tests/launcher.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn interaction_model_builds_launch_request_for_clicked_item() {}

#[test]
fn launcher_reopens_running_apps_when_activation_needs_window_restore() {}
```

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_interaction -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml --test launcher -- --nocapture`

Expected: FAIL because native interaction wiring is incomplete.

**Step 3: Write minimal implementation**

- Handle hover natively with mouse enter and exit tracking.
- Trigger launch and activate from native mouse-up.
- Keep existing launcher fallback behavior and tighten it only if tests require.

**Step 4: Run verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml --test native_interaction -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml --test launcher -- --nocapture`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/native_app/dock_item_view.rs src-tauri/src/native_app/dock_view.rs src-tauri/src/services/launcher.rs src-tauri/tests/native_interaction.rs src-tauri/tests/launcher.rs
git commit -m "feat: add native dock interaction"
```

### Task 8: Remove Tauri And Frontend Runtime

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Delete: `src-tauri/src/lib.rs`
- Delete: `src/`
- Modify: `Cargo.toml`
- Modify: `README.md`
- Modify: `.gitignore` if needed

**Step 1: Write the failing test**

```rust
#[test]
fn crate_builds_without_tauri_runtime() {}
```

**Step 2: Run verification to show the old runtime is still present**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS before deletion, then temporary FAIL while removing Tauri, then PASS again after cleanup.

**Step 3: Write minimal implementation**

- Remove Tauri dependencies and unused frontend tooling.
- Make the native app the only desktop runtime.
- Update docs to describe native startup.

**Step 4: Run verification**

Run:
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml Cargo.toml README.md .gitignore src-tauri/src/main.rs src-tauri/src/native_app
git rm src-tauri/src/lib.rs
git rm -r src
git commit -m "refactor: remove tauri runtime"
```

### Task 9: Final Native Quality Gate

**Files:**
- Modify: files touched by previous tasks as required
- Create: `docs/plans/2026-03-11-native-appkit-manual-checklist.md`

**Step 1: Run full verification**

Run:
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 2: Write manual macOS checklist**

Create a short checklist covering:
- first-run Dock import
- above-Dock z-order
- hover while another app is active
- single-click activate
- Finder reopen
- running-app refresh

**Step 3: Re-run final verification**

Run:
- `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-11-native-appkit-manual-checklist.md
git add src-tauri
git commit -m "chore: finalize native appkit dock rewrite"
```
