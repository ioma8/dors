use std::path::PathBuf;

use dors::adapters::running_apps::{
    RunningAppSnapshot, normalize_running_apps, parse_lsappinfo_list,
};

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

#[test]
fn running_apps_parse_visible_foreground_entries_from_lsappinfo() {
    let raw = r#"
42) "Firefox" ASN:0x0-0x40040:
    bundleID="org.mozilla.firefox"
    bundle path="/Applications/Firefox.app"
    executable path="/Applications/Firefox.app/Contents/MacOS/firefox"
    pid = 2785 type="Foreground" flavor=3 Version="14826.2.16"
43) "ControlCenter" ASN:0x0-0x50050:
    bundleID="com.apple.controlcenter"
    bundle path="/System/Library/CoreServices/ControlCenter.app"
    executable path="/System/Library/CoreServices/ControlCenter.app/Contents/MacOS/ControlCenter"
    pid = 901 type="UIElement" flavor=3 Version="1"
"#;

    let apps = parse_lsappinfo_list(raw, "Firefox");

    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].display_name, "Firefox");
    assert_eq!(apps[0].bundle_id.as_deref(), Some("org.mozilla.firefox"));
    assert_eq!(apps[0].path, PathBuf::from("/Applications/Firefox.app"));
    assert!(apps[0].is_active);
}
