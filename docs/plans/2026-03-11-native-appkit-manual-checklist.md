# Native AppKit Dock Manual Checklist

**Date:** 2026-03-11

Run the native app with:

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

## Manual Checks

- First-run import:
  - remove the persisted config at `~/Library/Application Support/dors/dock-config.json`
  - launch the app
  - confirm pinned apps are imported from the current macOS Dock

- Above-Dock z-order:
  - confirm the custom dock is visible over the system Dock, not behind it

- Hover while another app is active:
  - activate another app
  - move the cursor over the custom dock
  - confirm native controls still react visually

- Single-click activate:
  - click a running app once
  - confirm it becomes frontmost without repeated clicks

- Finder reopen:
  - click Finder
  - confirm a Finder window opens or reopens, not just the desktop focus

- Running-app refresh:
  - launch an app that is not pinned
  - confirm it appears in the dock shortly after launch
  - quit that app
  - confirm it disappears shortly after quitting
