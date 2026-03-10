# dors

`dors` is a Tauri-based macOS dock replacement prototype.

## Prerequisites

- macOS
- Rust toolchain
- Node.js and npm
- Tauri desktop prerequisites for macOS

## Install

```bash
npm install
```

## Run In Development

Start the frontend and Tauri desktop app together:

```bash
npm run tauri dev
```

If you only want the web UI in the browser:

```bash
npm run dev
```

## Verify

Backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Frontend:

```bash
npm test
npm run build
```

## Notes

- The current implementation includes backend state/config/import logic, a custom dock UI, and refresh/test scaffolding.
- Some macOS-native behavior is still prototype-level scaffolding rather than full production integration.
- Manual macOS checks are documented in [`docs/manual-test-macos-dock-replacement.md`](docs/manual-test-macos-dock-replacement.md).
