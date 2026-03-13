# Dock Hide Experiment Design

**Goal:** Build a separate experimental binary that attempts to make the real macOS Dock visually disappear while keeping Dock alive and preserving its reserved work area.

## Scope

This experiment must not change the main `dors` runtime path. It should live in its own binary and use a narrowly scoped private macOS adapter. The experiment should:

- locate the live visible Dock-related windows
- attempt to make them visually invisible from our own process
- keep Dock running and not autohidden
- restore any changed visual state on exit if restoration is possible

This experiment explicitly does not:

- change the existing dock replacement app runtime
- attempt to stop Dock mouse interaction
- install an osax or other system-wide scripting addition
- require root privileges as part of normal operation

## Architecture

Add a new binary at `src/bin/dock_hide_experiment.rs` and a small private adapter module at `src/private_dock_experiment.rs`.

The binary will:

1. query current WindowServer state for Dock-owned window ids
2. snapshot any visual properties that can be read back
3. apply a private visual suppression strategy
4. wait until interrupted
5. restore on exit / signal

The adapter will be macOS-only and isolated from the existing `native_app` module tree.

## Approaches Considered

### 1. Recommended: Private WindowServer control from our own process

Use private SkyLight / WindowServer APIs to locate Dock-owned visible windows and attempt to modify visual state such as alpha / opacity / ordering from outside the Dock process.

Pros:

- narrowest possible experiment
- no changes to the main app
- avoids immediately becoming a full injection project

Cons:

- private API behavior may not be sufficient
- restoration may be partial depending on what state can be read back
- version-sensitive and unsupported

### 2. Dock injection / scripting-addition path

Inject into `Dock.app` or install an osax in the same class as `yabai`.

Pros:

- highest control ceiling

Cons:

- much larger scope
- likely requires SIP concessions
- not a minimal experiment

### 3. Visual cover only

Cover the Dock with another window.

Pros:

- easy

Cons:

- not actually making Dock invisible
- not a meaningful experiment for the stated goal

## Success Criteria

The experiment is considered successful if:

- the real Dock remains alive
- the system still reserves Dock work area
- the Dock is visually absent or materially near-invisible

It is acceptable if:

- restoration requires restarting Dock
- some Dock windows remain technically present but visually suppressed

It is not acceptable if:

- the main `dors` app behavior changes
- the experiment permanently changes system preferences

## Testing

Testing will be pragmatic:

- pure tests for Dock window candidate selection and suppression planning
- `cargo check` after each code change
- manual runtime test of the binary on macOS

## Risks

- private APIs may not affect Dock-owned windows from an external process
- the visible Dock strip may be composed of multiple windows and subwindows
- some visual state may not be readable/restorable without restarting Dock

If the first private suppression attempt fails, the next step is to decide whether to escalate to a deeper private architecture instead of broadening this experiment indefinitely.
