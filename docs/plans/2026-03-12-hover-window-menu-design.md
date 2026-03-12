# Hover Window Menu Design

## Goal

When hovering an app icon that has more than one open window, show a native menu above that icon after a short delay. Selecting a menu item should activate that specific window.

## Chosen Approach

Use a native `NSMenu` popup with a short hover delay.

- Delay: about `180ms`
- Only show a menu when the app has more than one open window
- Single-window apps keep the existing click activation behavior
- Menu items are the current open window titles for that app
- Clicking a menu item activates that exact window

## Architecture

Add a `window_menu` module responsible for:

- hover-delay scheduling and cancellation
- querying open windows for a specific app
- building an `NSMenu`
- activating a specific window

The existing dock item button hover hooks remain the entry point. They will notify the controller when hover starts or ends, and the controller will delegate to `window_menu`.

## Data Flow

1. Mouse enters dock item
   - controller starts a delayed menu request for that item
2. Delay elapses
   - query open windows for the app
   - if count > 1, show menu above icon
3. User clicks a menu item
   - activate that exact window
4. Mouse leaves icon before delay
   - cancel pending menu
5. Hovering another icon
   - close/cancel previous menu state

## Window Discovery

Use `System Events` / Accessibility to list windows for the specific app process:

- filter out empty-title windows
- keep visible standard windows
- return enough data to activate one window by title/index

## Error Handling

- If window discovery fails, do nothing and keep normal click behavior
- If activation of a specific window fails, fall back to app activation for that app
- If hover scheduling races, newer hover wins and older requests are ignored

## Testing

Add tests for:

- hover-delay config and scheduler behavior
- menu visibility condition (`> 1` window only)
- window title normalization/filtering
- activation script building for a specific app window
