use std::path::PathBuf;

use dors_tauri_lib::adapters::running_apps::{RunningAppSnapshot, normalize_running_apps};

#[test]
fn running_apps_filters_out_non_regular_entries() {
    let apps = vec![
        RunningAppSnapshot {
            bundle_id: Some("com.apple.Safari".to_string()),
            display_name: "Safari".to_string(),
            path: PathBuf::from("/Applications/Safari.app"),
            activation_policy_regular: true,
            is_active: true,
        },
        RunningAppSnapshot {
            bundle_id: Some("com.apple.controlcenter".to_string()),
            display_name: "Control Center".to_string(),
            path: PathBuf::from("/System/Library/CoreServices/ControlCenter.app"),
            activation_policy_regular: false,
            is_active: false,
        },
    ];

    let normalized = normalize_running_apps(apps);

    assert_eq!(normalized.len(), 1);
    assert_eq!(normalized[0].display_name, "Safari");
}

#[test]
fn running_apps_normalizes_identity_to_match_pinned_imports() {
    let apps = vec![RunningAppSnapshot {
        bundle_id: Some("com.apple.mail".to_string()),
        display_name: "Mail".to_string(),
        path: PathBuf::from("/System/Applications/Mail.app/"),
        activation_policy_regular: true,
        is_active: false,
    }];

    let normalized = normalize_running_apps(apps);

    assert_eq!(
        normalized[0].identity.path,
        PathBuf::from("/System/Applications/Mail.app")
    );
    assert_eq!(
        normalized[0].identity.bundle_id.as_deref(),
        Some("com.apple.mail")
    );
}
