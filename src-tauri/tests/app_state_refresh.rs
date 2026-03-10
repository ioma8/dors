use std::path::PathBuf;

use dors_tauri_lib::app_state::AppState;
use dors_tauri_lib::config::DockConfig;
use dors_tauri_lib::domain::{AppIdentity, DockItemView};

fn dock_item(name: &str, path: &str) -> DockItemView {
    DockItemView {
        identity: AppIdentity {
            bundle_id: Some(format!("com.example.{name}")),
            path: PathBuf::from(path),
        },
        display_name: name.to_string(),
        icon_src: String::new(),
        is_pinned: true,
        is_running: true,
        is_active: false,
        is_degraded: false,
    }
}

#[test]
fn app_state_replaces_dock_items_when_running_apps_change() {
    let state = AppState::new(DockConfig::default());
    let replacement = vec![dock_item(
        "Terminal",
        "/System/Applications/Utilities/Terminal.app",
    )];

    state
        .replace_dock_items(replacement.clone())
        .expect("replace should succeed");

    assert_eq!(
        state.dock_items().expect("load should succeed"),
        replacement
    );
}
