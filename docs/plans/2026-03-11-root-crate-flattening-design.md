# Root Crate Flattening Design

**Date:** 2026-03-11

## Goal

Move the native Rust/AppKit application from `src-tauri/` into the repository root so the project becomes a plain Rust crate that builds and runs with `cargo run` from the root, without retaining Tauri-shaped directory structure.

## Why Change The Layout

The runtime has already been rewritten away from Tauri, but the repository layout still communicates the old architecture:

- the active Rust crate still lives under `src-tauri/`
- commands still need `--manifest-path src-tauri/Cargo.toml`
- the repository still looks like a migrated Tauri project instead of a native Rust app

This change removes that structural mismatch without changing product behavior.

## Scope

### Included

- move the active crate from `src-tauri/` to the repository root
- move source files to `src/`
- move integration tests to `tests/`
- merge crate metadata into the root `Cargo.toml`
- update README and commands to plain root Cargo usage
- delete the now-obsolete `src-tauri/` directory once the root crate is compiling

### Excluded

- behavior changes to the dock
- redesign of the module architecture
- changes to persisted data format
- changes to the native AppKit interaction model beyond path updates

## Target Layout

After the flattening, the repository should look like a normal Rust app:

- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `src/main.rs`
- `src/lib.rs`
- `src/adapters/*`
- `src/config/*`
- `src/domain/*`
- `src/native_app/*`
- `src/services/*`
- `src/window_level.rs`
- `src/window_position.rs`
- `tests/*`
- `docs/plans/*`

The old `src-tauri/` directory should no longer contain active code.

## Architecture

This is a crate-boundary migration, not an application rewrite.

### Reused As-Is

Keep the existing native architecture intact:

- backend adapters
- config store
- domain models
- native AppKit shell
- launcher and refresh logic
- integration tests

### Structural Changes

- move crate source files from `src-tauri/src/` to `src/`
- move integration tests from `src-tauri/tests/` to `tests/`
- move any still-needed fixtures under `tests/fixtures/`
- merge `src-tauri/Cargo.toml` into root `Cargo.toml`
- remove workspace indirection once the root crate exists

## Migration Strategy

Use a small-step migration so the crate remains buildable at each checkpoint:

1. Prepare the root manifest for a real crate.
2. Move the test suite and fixtures to root-level `tests/`.
3. Move the library modules to root `src/`.
4. Move the binary entry point to root `src/main.rs`.
5. Delete obsolete `src-tauri/` support files and empty directories.
6. Update docs and commands.

This order minimizes broken path churn and keeps verification straightforward.

## Error Handling

- If a moved test or fixture path breaks, fix the path immediately before additional moves.
- If the root manifest temporarily conflicts with the nested crate, prefer a short additive overlap rather than deleting the nested crate first.
- If any module path stops compiling after a move, repair imports before continuing.

## Testing Strategy

### During Migration

Run `cargo check` after every file move batch.

Run focused tests after each moved area, such as:

- `cargo test --test native_layout`
- `cargo test --test native_refresh`
- `cargo test --test launcher`

### Final Gate

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo check`

## Cleanup Rules

- Remove obsolete empty directories after the root crate is verified.
- Remove dead references to `src-tauri` from docs and commands.
- Leave unrelated untracked plan files alone unless explicitly requested.
