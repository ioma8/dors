# Native AppKit Dock Rewrite Design

**Date:** 2026-03-11

## Goal

Replace the Tauri and WKWebView shell with a native macOS AppKit shell built in Rust using `objc2`, while preserving the existing Rust domain, config, import, running-app, merge, icon, and launch logic wherever it already works.

## Why Rewrite The Shell

The current Tauri implementation already proved that the backend logic is mostly serviceable, but the shell is the blocker:

- hover behavior is unreliable once another app is active
- first-click activation is inconsistent
- the overlay competes incorrectly with macOS Dock and app focus semantics

These failures happen at the macOS window, responder, and webview interaction boundary. Rewriting only the shell removes the broken layer without redoing the tested backend logic.

## Scope

### Included

- native `NSApplication` bootstrapping in Rust
- native dock panel positioned over the system Dock
- native icon strip rendering with hover and click handling
- reuse of existing config bootstrap, Dock import, running-app discovery, dock merge, icon loading, and launcher services
- periodic runtime refresh of running apps

### Excluded

- modifying or hiding the system Dock
- drag-to-reorder
- window management beyond launch and activate
- keyboard shortcut support
- cross-platform support

## Architecture

The rewritten app keeps the current backend as the application core and replaces the composition root and UI.

### Reused Core

Keep these modules, with only the smallest interface cleanup needed:

- `src-tauri/src/adapters/dock_import.rs`
- `src-tauri/src/adapters/running_apps.rs`
- `src-tauri/src/adapters/app_resolver.rs`
- `src-tauri/src/adapters/icon_loader.rs`
- `src-tauri/src/config/*`
- `src-tauri/src/domain/*`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/services/dock_state.rs`
- `src-tauri/src/services/launcher.rs`

### Replaced Shell

Delete the Tauri and frontend dependency path from the runtime:

- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`
- `src/`
- `package.json` and related frontend build tooling

Replace them with a native shell:

- `src-tauri/src/native_app/mod.rs`
- `src-tauri/src/native_app/app_delegate.rs`
- `src-tauri/src/native_app/panel.rs`
- `src-tauri/src/native_app/dock_view.rs`
- `src-tauri/src/native_app/dock_item_view.rs`
- `src-tauri/src/native_app/refresh.rs`

The exact file split can move slightly during implementation, but the shell should stay isolated under one `native_app` module tree.

## Native UI Model

### Application

- Start a native `NSApplication` on the main thread.
- Set activation policy to accessory so the app does not appear as a normal foreground desktop app.
- Install a small Rust app delegate for startup, timer wiring, and termination behavior.

### Window

- Use a borderless transparent panel-like window.
- Place it bottom-center on the primary monitor, flush with the bottom edge.
- Set a native level above the system Dock.
- Configure the panel to behave as a dock overlay, not a normal app window.

### View Hierarchy

- root visual effect or transparent container view
- horizontal stack-like dock row
- one native item view per dock item
- image view for the app icon
- running indicator view below the icon
- active-state styling for the frontmost app

### Interaction

- hover uses AppKit mouse enter and exit tracking instead of CSS or webview hover
- click uses native mouse-up handling directly in the item view
- clicking an item calls the existing launcher service synchronously on the Rust side

## Data Flow

### Startup

1. Build the config store from the app data directory.
2. Load config.
3. If missing, import the current macOS Dock and persist it.
4. Read running apps.
5. Merge pinned and running apps into `DockItemView` values.
6. Load icon data for visible items.
7. Create native item views and show the dock panel.

### Refresh

- Poll running apps on a short interval, initially 500ms to keep iteration fast.
- Rebuild dock items from the existing `AppState`.
- Diff the new item list against the last rendered list.
- Update only changed item views when possible.
- If a refresh fails, keep the last rendered state.

### Click

- Native item view creates a `LaunchRequest`.
- Existing `services::launcher` decides activate versus launch.
- After a click, schedule a near-term refresh to reflect running and active state changes quickly.

## State Model

Keep `domain::DockItemView` as the UI-facing normalized item. Extend only if the native shell needs more explicit fields, for example:

- cached decoded icon bytes
- a stable item key for native diffing

Do not introduce a second competing UI state model unless the native view layer strictly requires it.

## Error Handling

- If initial Dock import fails, continue with empty pinned items and still show running apps.
- If running-app discovery fails temporarily, keep the last good view model.
- If icon conversion fails for one app, use a generated placeholder tile for that item only.
- If activate fails, use the existing launch fallback behavior.
- Log native shell failures to stderr with the same debug prefix already used by the project.

## Testing Strategy

### Keep Existing Tests

Preserve the current backend tests for:

- config load and save
- Dock import parsing
- running-app normalization
- dock-state merge logic
- launcher behavior
- icon loading

### Add Native-Focused Tests

Add small deterministic tests for:

- bottom-edge placement math
- native item diffing or reconciliation logic
- icon placeholder fallback mapping

### Manual Verification

Manual macOS checks remain required for:

- hover while another app is focused
- single-click activation from the inactive overlay
- Finder behavior
- z-order above the system Dock
- runtime refresh when apps open and close

## Migration Strategy

Use a fast, narrow migration rather than a long-lived dual runtime:

1. Stabilize shared backend interfaces so they no longer depend on Tauri state types.
2. Introduce a native app entry point beside the current Tauri shell.
3. Stand up a minimal native panel showing static test content.
4. Connect real dock state from the existing backend.
5. Add native click handling and hover behavior.
6. Remove Tauri and frontend runtime code once the native shell reaches feature parity.

This keeps the rewrite incremental while preventing the old shell from shaping the new design.
