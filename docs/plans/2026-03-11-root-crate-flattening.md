# Root Crate Flattening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Flatten the native Rust/AppKit dock app into a single root Rust crate so the repository builds and runs with plain root Cargo commands.

**Architecture:** Keep the current native AppKit application logic unchanged while moving the crate boundary from `src-tauri/` to the repository root. Migrate tests, source files, and manifest metadata in small verified steps, then delete the obsolete nested crate structure.

**Tech Stack:** Rust 2024, `objc2`, `objc2-app-kit`, `objc2-foundation`, existing serde/plist/base64/thiserror modules, Cargo integration tests

---

### Task 1: Prepare The Root Crate Manifest

**Files:**
- Modify: `Cargo.toml`
- Modify: `README.md`
- Test: root cargo metadata and check commands

**Step 1: Write the failing verification**

Run:

```bash
cargo check
```

Expected: FAIL because the root manifest does not yet define the full application crate.

**Step 2: Add the minimal root crate manifest**

- Merge package metadata and dependencies from `src-tauri/Cargo.toml` into `Cargo.toml`
- Keep paths temporary if needed during transition
- Update README commands toward plain `cargo run`

**Step 3: Run verification**

Run:

```bash
cargo check
```

Expected: PASS or a narrower path-related failure that points to the next move.

**Step 4: Commit**

```bash
git add Cargo.toml README.md
git commit -m "build: prepare root crate manifest"
```

### Task 2: Move The Integration Tests To Root

**Files:**
- Create: `tests/`
- Move: `src-tauri/tests/*` -> `tests/*`
- Move: `src-tauri/tests/fixtures/*` -> `tests/fixtures/*`

**Step 1: Write the failing verification**

Run:

```bash
cargo test --test native_layout -- --nocapture
```

Expected: FAIL after removing or relocating the old path until the root `tests/` tree is wired correctly.

**Step 2: Move the tests with minimal path fixes**

- Preserve test contents
- Fix fixture includes and import paths only where required

**Step 3: Run verification**

Run:

```bash
cargo test --test native_layout -- --nocapture
cargo test --test native_refresh -- --nocapture
cargo check
```

Expected: PASS

**Step 4: Commit**

```bash
git add tests
git rm -r src-tauri/tests
git commit -m "test: move integration tests to root"
```

### Task 3: Move The Library Modules To Root `src/`

**Files:**
- Create: `src/lib.rs`
- Move: `src-tauri/src/adapters/*` -> `src/adapters/*`
- Move: `src-tauri/src/config/*` -> `src/config/*`
- Move: `src-tauri/src/domain/*` -> `src/domain/*`
- Move: `src-tauri/src/native_app/*` -> `src/native_app/*`
- Move: `src-tauri/src/services/*` -> `src/services/*`
- Move: `src-tauri/src/app_state.rs` -> `src/app_state.rs`
- Move: `src-tauri/src/window_level.rs` -> `src/window_level.rs`
- Move: `src-tauri/src/window_position.rs` -> `src/window_position.rs`

**Step 1: Write the failing verification**

Run:

```bash
cargo check
```

Expected: FAIL while files are partially moved.

**Step 2: Move the library code minimally**

- Preserve module structure
- Fix `mod` declarations and crate paths
- Do not change behavior

**Step 3: Run verification**

Run:

```bash
cargo test --test launcher -- --nocapture
cargo test --test native_interaction -- --nocapture
cargo check
```

Expected: PASS

**Step 4: Commit**

```bash
git add src
git rm -r src-tauri/src
git commit -m "refactor: move app library to root src"
```

### Task 4: Move The Native Binary Entry Point

**Files:**
- Create: `src/main.rs`
- Remove: `src-tauri/src/main.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing verification**

Run:

```bash
cargo run --help
```

Expected: FAIL or point at the missing root binary until `src/main.rs` exists.

**Step 2: Move the binary entry point**

- Keep startup behavior identical
- Ensure root `cargo run` targets the native app

**Step 3: Run verification**

Run:

```bash
cargo check
cargo test --test native_panel -- --nocapture
```

Expected: PASS

**Step 4: Commit**

```bash
git add src/main.rs Cargo.toml
git rm src-tauri/src/main.rs
git commit -m "feat: run native dock from root crate"
```

### Task 5: Remove The Remaining Nested Crate Structure

**Files:**
- Delete: `src-tauri/Cargo.toml`
- Delete: `src-tauri/build.rs`
- Delete: `src-tauri/`
- Modify: `.gitignore` if needed

**Step 1: Write the failing verification**

Run:

```bash
find src-tauri -type f
```

Expected: old crate files still exist before cleanup.

**Step 2: Remove obsolete nested-crate files**

- Delete the empty or obsolete nested crate files
- Keep only the root crate structure

**Step 3: Run verification**

Run:

```bash
cargo check
cargo test
```

Expected: PASS

**Step 4: Commit**

```bash
git rm -r src-tauri
git commit -m "refactor: remove nested crate structure"
```

### Task 6: Final Docs And Quality Gate

**Files:**
- Modify: `README.md`
- Modify: docs that still reference `src-tauri`
- Create or update: any root-oriented manual command references

**Step 1: Update docs**

- Make all run and verification commands use plain root Cargo commands
- Remove stale `src-tauri` wording

**Step 2: Run final verification**

Run:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check
```

Expected: PASS

**Step 3: Commit**

```bash
git add README.md docs src tests Cargo.toml Cargo.lock
git commit -m "chore: finalize root crate flattening"
```
