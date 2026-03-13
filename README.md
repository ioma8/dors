# dors

`dors` is a native Rust/AppKit macOS dock replacement prototype.

## Prerequisites

- macOS
- Rust toolchain

## Run

```bash
cargo run
```

## Project Structure

- `src/main.rs`
  Native app entry point.
- `src/native_app/`
  AppKit shell, dock window, interactions, hover popovers, AX-based window management, and macOS-specific runtime glue.
- `src/services/`
  Higher-level app behavior such as launching/activating apps and merging dock state.
- `src/adapters/`
  IO-facing integrations for Dock import, running apps, icon loading, and app resolution.
- `src/domain/`
  Core dock/app data models.
- `src/config/`
  Persisted config schema and store.
- `src/bin/`
  Standalone experiment binaries.

## Runtime Data Flow

Startup:
- `src/main.rs` boots the native AppKit app.
- `src/native_app/app.rs` creates the dock panel and controller.
- `src/native_app/refresh.rs` loads persisted config or imports the current macOS Dock on first run.
- Running apps are discovered, merged with pinned apps, and converted into native dock view models.
- `src/native_app/interaction.rs` initializes startup window state and installs the live AX observer.

Normal operation:
- Clicking a dock item goes through `src/services/launcher.rs` and activates or launches the target app natively.
- Hovering an app with multiple windows uses `src/native_app/window_menu.rs` and `src/native_app/window_popup.rs` to show the window chooser popover.
- Right-clicking a dock item opens the native context menu for app actions.
- Window resizing/maximize behavior is handled by `src/native_app/ax_window_manager.rs` plus `src/native_app/window_clamper.rs`, which react to AX events and keep windows inside the custom working area above the dock.

## Verify

```bash
cargo check
cargo test
```

## Notes

- The app uses native AppKit for the dock shell and reuses Rust backend logic for import, config, launch, icons, and running-app state.
- Manual macOS checks for the native rewrite are documented in `docs/plans/2026-03-11-native-appkit-manual-checklist.md`.
