use std::path::{Path, PathBuf};

use dors_tauri_lib::app_state::AppState;
use dors_tauri_lib::config::DockConfig;
use dors_tauri_lib::domain::{AppIdentity, PinnedApp, RunningApp};
use dors_tauri_lib::native_app::refresh::build_refresh_models;

#[test]
fn refresh_rebuilds_native_items_from_running_apps() {
    let state = AppState::new(DockConfig {
        pinned_apps: vec![PinnedApp {
            identity: AppIdentity {
                bundle_id: Some("com.apple.finder".to_string()),
                path: PathBuf::from("/System/Library/CoreServices/Finder.app"),
            },
            display_name: "Finder".to_string(),
        }],
    });

    let models = build_refresh_models(
        &state,
        vec![
            RunningApp {
                identity: AppIdentity {
                    bundle_id: Some("com.apple.finder".to_string()),
                    path: PathBuf::from("/System/Library/CoreServices/Finder.app"),
                },
                display_name: "Finder".to_string(),
                is_active: true,
            },
            RunningApp {
                identity: AppIdentity {
                    bundle_id: Some("com.github.wez.wezterm".to_string()),
                    path: PathBuf::from("/Applications/WezTerm.app"),
                },
                display_name: "WezTerm".to_string(),
                is_active: false,
            },
        ],
        |path: &Path| {
            if path == Path::new("/System/Library/CoreServices/Finder.app") {
                return Some("data:image/png;base64,ZmFrZQ==".to_string());
            }

            None
        },
    )
    .expect("refresh should succeed");

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].display_name, "Finder");
    assert!(!models[0].uses_placeholder_icon);
    assert!(models[0].is_active);
    assert_eq!(models[1].display_name, "WezTerm");
    assert!(models[1].uses_placeholder_icon);
    assert!(models[1].shows_indicator);
}
