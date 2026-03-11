# dors

`dors` is a native Rust/AppKit macOS dock replacement prototype.

## Prerequisites

- macOS
- Rust toolchain

## Run In Development

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

## Verify

Backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Notes

- The current implementation uses native AppKit for the dock shell and reuses Rust backend logic for import, config, launch, icons, and running-app state.
- Manual macOS checks for the native rewrite are documented in `docs/plans/2026-03-11-native-appkit-manual-checklist.md`.
