# Window Clamping Above Custom Dock Design

## Goal

Keep the system Dock simple by autohiding it while `dors` runs, and preserve usable desktop space by clamping normal resizable app windows so they stay above the custom dock height.

## Context

The previous attempt kept the system Dock visible to preserve the macOS work area, then tried to suppress Dock interaction with blocker panels and event taps. That path is brittle and did not stop Dock hover behavior reliably. The new design removes those hacks and makes `dors` responsible for reserving space by resizing normal app windows into a custom allowed area.

## Approaches Considered

### 1. Poll and clamp normal windows

Periodically enumerate visible windows, compute the allowed frame for the main screen, and move/resize only normal resizable windows that extend into the custom dock strip.

Pros:
- Simple and additive
- Reuses the existing native refresh/timer model
- Does not require AX observers or private hooks

Cons:
- Reactive rather than perfectly event-driven

### 2. Accessibility observers

Watch app/window resize events via AX APIs and clamp immediately.

Pros:
- More event-driven

Cons:
- Requires more permissions
- Much more code and app-specific edge cases

### 3. Continue Dock interception hacks

Keep the visible Dock for work area and try to block its interaction.

Pros:
- Preserves native work area if it worked

Cons:
- Already demonstrated as unreliable
- Increases unsupported native complexity

## Chosen Architecture

Use approach 1.

- `system_dock` returns to a minimal responsibility:
  - snapshot current Dock prefs
  - force `autohide=true` on startup
  - restore original prefs on normal exit and handled signals
- Add a new `window_clamper` native adapter that:
  - enumerates app windows on the main screen
  - filters to normal, visible, resizable, non-fullscreen windows
  - computes an allowed working area from the screen frame, visible top reserve, and custom dock height
  - resizes/repositions offending windows back into that area
- Run window clamping from the same periodic refresh timer used by the dock controller.

## Components

### `src/native_app/system_dock.rs`

Remove the tilesize-sizing path from startup. Keep:
- preference snapshot parsing
- autohide restore planning
- Dock restart handling
- signal restore wiring

Change startup behavior to always target `autohide=true`.

### `src/native_app/window_clamper.rs`

New module with:
- pure geometry helpers
- macOS window enumeration/filtering adapter
- clamp application function

The pure portion will calculate:
- allowed working area
- whether a window needs clamping
- the corrected frame

### `src/native_app/app.rs`

Remove:
- blocker panel creation
- event blocker installation

Add:
- installation of the window-clamping timer or hook through the existing controller refresh path

### `src/native_app/refresh.rs` / `src/native_app/interaction.rs`

Wire the clamper into the periodic native refresh cycle so dock state refresh and window clamping happen together.

## Data Flow

1. Startup
   - read Dock prefs
   - enable Dock autohide
   - restart Dock
   - create the overlay dock panel
   - start periodic refresh

2. Refresh tick
   - refresh dock models as today
   - compute custom allowed work area
   - enumerate candidate app windows
   - clamp any window extending below the allowed bottom edge

3. Shutdown
   - restore original Dock prefs
   - restart Dock if needed

## Rules For Window Clamping

Clamp only windows that are:
- on the main screen
- normal app windows
- visible on screen
- resizable
- not fullscreen

Do not clamp:
- menu bar items
- system overlays
- fullscreen spaces
- tiny transient panels/tooltips

When clamping:
- preserve current width/height when possible
- only reduce height if the window is taller than allowed
- move the window upward if only its bottom edge violates the allowed area

## Error Handling

- If Dock autohide change fails, app startup fails with a clear error.
- If one window cannot be resized, skip it and continue processing others.
- If window enumeration fails on a tick, skip that tick and continue future ticks.

## Testing

Add tests first for:
- autohide-only Dock preference plan
- allowed work area geometry
- frame clamping behavior
- candidate-window filtering helpers

Manual verification:
- Dock autohides when `dors` starts
- Dock restores on exit
- normal resizable windows cannot overlap the custom dock strip
- fullscreen apps remain untouched
