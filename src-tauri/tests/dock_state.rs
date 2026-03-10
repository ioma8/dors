use std::path::PathBuf;

use dors_tauri_lib::config::DockConfig;
use dors_tauri_lib::domain::{AppIdentity, PinnedApp, RunningApp};
use dors_tauri_lib::services::dock_state::build_dock_items;

fn pinned(bundle_id: Option<&str>, path: &str, display_name: &str) -> PinnedApp {
    PinnedApp {
        identity: AppIdentity {
            bundle_id: bundle_id.map(ToString::to_string),
            path: PathBuf::from(path),
        },
        display_name: display_name.to_string(),
    }
}

fn running(bundle_id: Option<&str>, path: &str, display_name: &str, is_active: bool) -> RunningApp {
    RunningApp {
        identity: AppIdentity {
            bundle_id: bundle_id.map(ToString::to_string),
            path: PathBuf::from(path),
        },
        display_name: display_name.to_string(),
        is_active,
    }
}

#[test]
fn dock_state_keeps_pinned_items_first_in_persisted_order() {
    let items = build_dock_items(
        &DockConfig {
            pinned_apps: vec![
                pinned(
                    Some("com.apple.mail"),
                    "/System/Applications/Mail.app",
                    "Mail",
                ),
                pinned(
                    Some("com.apple.Safari"),
                    "/Applications/Safari.app",
                    "Safari",
                ),
            ],
        },
        &[],
    );

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].display_name, "Mail");
    assert_eq!(items[1].display_name, "Safari");
}

#[test]
fn dock_state_collapses_running_pinned_apps_and_appends_unpinned_running_apps() {
    let items = build_dock_items(
        &DockConfig {
            pinned_apps: vec![pinned(
                Some("com.apple.Safari"),
                "/Applications/Safari.app",
                "Safari",
            )],
        },
        &[
            running(
                Some("com.apple.Safari"),
                "/Applications/Safari.app",
                "Safari",
                false,
            ),
            running(
                Some("com.apple.Terminal"),
                "/System/Applications/Utilities/Terminal.app",
                "Terminal",
                false,
            ),
        ],
    );

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].display_name, "Safari");
    assert!(items[0].is_pinned);
    assert!(items[0].is_running);
    assert_eq!(items[1].display_name, "Terminal");
    assert!(!items[1].is_pinned);
    assert!(items[1].is_running);
}

#[test]
fn dock_state_surfaces_active_state_and_degraded_placeholders() {
    let items = build_dock_items(
        &DockConfig {
            pinned_apps: vec![pinned(None, "", "Missing App")],
        },
        &[running(
            Some("com.apple.finder"),
            "/System/Library/CoreServices/Finder.app",
            "Finder",
            true,
        )],
    );

    assert_eq!(items.len(), 2);
    assert!(items[0].is_degraded);
    assert!(items[1].is_active);
    assert_eq!(items[1].display_name, "Finder");
}
