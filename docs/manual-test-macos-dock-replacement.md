# Manual Test Checklist: macOS Dock Replacement

## Preconditions

- Run the app on macOS with Tauri desktop support available.
- Confirm the current user has a readable `~/Library/Preferences/com.apple.dock.plist`.
- Remove any prior persisted app config if you need to exercise true first-run behavior.

## First-Run Import

1. Start the app with no persisted config present.
2. Confirm the frontend renders imported pinned apps in the same order as the system Dock.
3. Confirm unresolved Dock entries do not crash startup.
4. Confirm an empty-state message is shown if import yields no pinned apps.

## Launch And Activate

1. Click a running app item.
2. Confirm the running app becomes frontmost instead of launching a duplicate instance.
3. Click a non-running pinned app.
4. Confirm the app launches.
5. If native activation fails, confirm the fallback launch path still opens the app.

## Runtime Refresh

1. Start or quit a GUI app while the dock replacement is open.
2. Wait for the refresh interval to elapse.
3. Confirm the dock updates without duplicate items or a full UI breakdown.
4. Confirm active-state styling moves to the newly frontmost app.

## Persistence

1. Close the app after first-run import.
2. Start it again.
3. Confirm the persisted pinned config is used without reimporting from the system Dock.
