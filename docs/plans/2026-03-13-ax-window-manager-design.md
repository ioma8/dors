# AX Window Manager Design

**Date:** 2026-03-13

## Goal

Replace the current timer-driven custom zoom/clamp logic with an Accessibility observer based window manager that can reliably preserve native-like double-click title-bar zoom semantics:

- first double-click: native zoom is converted into the custom work area
- second double-click: the original regular frame is restored

## Problem Summary

The current design relies on periodic polling to:

- capture regular frames
- detect native zoom frames
- apply custom clamping
- infer restore intent

This is fundamentally racy. The window can move through regular, native zoomed, and custom-clamped states between polling intervals. Even after improving tracker logic and timer cadence, runtime behavior remains inconsistent because the geometry transitions happen faster than the polling loop can reliably observe.

## Recommended Approach

Use macOS Accessibility observers (`AXObserver`) as the primary signal source for normal resizable windows.

The app will:

- observe focused/main window changes per running app
- observe window move and resize notifications
- read the updated frame immediately from AX
- drive a managed zoom state machine from those events rather than from sampled timers

## Why This Is Better

Compared with the current timer model:

- it reacts to the actual resize/zoom event instead of trying to infer it later
- it removes timing races between separate capture/clamp timers
- it keeps the managed restore frame attached to the real window lifecycle
- it is closer to how existing macOS window managers implement reliable geometry handling

## Scope

The AX-based manager only handles:

- normal visible resizable windows
- non-fullscreen windows
- windows on the main screen work area

It does not attempt to manage:

- true fullscreen spaces
- minimized windows
- nonstandard/tool/utility windows

## Architecture

### 1. AX Window Manager

Add a new native module, tentatively `src/native_app/ax_window_manager.rs`, responsible for:

- creating `AXObserver` instances per relevant application
- registering notifications for:
  - focused window changes
  - main window changes
  - moved
  - resized
- normalizing AX events into internal window events

### 2. Managed Zoom Tracker

Keep a pure state machine for:

- remembered regular frame
- active managed custom-zoom state
- restore frame invalidation after manual resize/move

This logic should stay testable without AppKit/AX.

### 3. Runtime Integration

The native runtime should:

- keep the slow dock-refresh timer for app model/UI refresh only
- stop using periodic polling as the primary geometry mechanism
- feed AX events into the managed zoom tracker
- apply geometry operations immediately when state transitions require them

## Identity

The runtime must stop depending on unstable titles for identity.

Preferred identity order:

1. AX window element identity during runtime
2. document-based key when available
3. app-local window index only as a fallback

Within a single runtime session, the AX element reference should be the authoritative identity.

## Event Handling Rules

For a normal resizable window:

- If it enters the native zoom frame and no managed state exists:
  - save the last regular frame
  - resize to the custom work area
  - mark as managed

- If it enters the native zoom frame and managed state exists:
  - restore the saved regular frame
  - clear managed state

- If it is already in the custom work area and has a saved regular frame:
  - treat it as managed custom-zoomed

- If the user manually resizes or moves after custom zoom:
  - clear managed state
  - store the new regular frame

## Error Handling

- If AX permission is missing, log one clear startup error and skip managed zoom behavior.
- If observing a specific app/window fails, continue observing other apps.
- If a geometry write fails for a given window, log it and keep the manager alive.

## Testing Strategy

### Pure tests

Keep and expand the existing pure window-clamper/managed-zoom tests for:

- native zoom -> custom zoom
- custom zoom -> restore
- rehydration from custom frame
- manual resize invalidation

### Adapter tests

Add focused tests for:

- AX event normalization
- observer registration bookkeeping
- window identity mapping

### Manual smoke tests

Verify:

- double-click title bar on IntelliJ / Firefox / Finder-like windows
- first click lands in custom work area
- second click restores exact regular frame
- manual resize after custom zoom establishes a new restore frame

## Migration Plan

1. Introduce AX manager and pure event model alongside current timer path.
2. Route managed zoom decisions through AX events.
3. Remove timer-based geometry polling once AX path is stable.
4. Keep only the slow dock UI refresh timer.
