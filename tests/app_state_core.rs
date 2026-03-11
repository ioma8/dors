use std::path::PathBuf;

use dors::app_state::AppState;
use dors::config::DockConfig;
use dors::domain::{AppIdentity, PinnedApp, RunningApp};

#[test]
fn app_state_refresh_returns_stable_items_for_native_shell() {
    let state = AppState::new(DockConfig {
        pinned_apps: vec![PinnedApp {
            identity: AppIdentity {
                bundle_id: Some("com.apple.finder".to_string()),
                path: PathBuf::from("/System/Library/CoreServices/Finder.app"),
            },
            display_name: "Finder".to_string(),
        }],
    });

    let items = state
        .refresh_snapshot(vec![
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
                    bundle_id: None,
                    path: PathBuf::from("/Applications/WezTerm.app"),
                },
                display_name: "WezTerm".to_string(),
                is_active: false,
            },
        ])
        .expect("refresh should succeed");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].display_name, "Finder");
    assert!(items[0].is_pinned);
    assert!(items[0].is_running);
    assert!(items[0].is_active);
    assert_eq!(items[0].stable_key(), "bundle:com.apple.finder");

    assert_eq!(items[1].display_name, "WezTerm");
    assert!(!items[1].is_pinned);
    assert!(items[1].is_running);
    assert!(!items[1].is_active);
    assert_eq!(items[1].stable_key(), "path:/Applications/WezTerm.app");

    assert_eq!(
        state.dock_items().expect("state should keep the snapshot"),
        items
    );
}
