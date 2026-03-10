use std::path::PathBuf;

use dors_tauri_lib::adapters::dock_import::DockImportReader;

#[test]
fn dock_import_parses_pinned_apps_in_order_from_fixture() {
    let fixture = include_bytes!("fixtures/dock_prefs_valid_sample.plist");

    let result = DockImportReader::default()
        .parse_plist_bytes(fixture)
        .expect("fixture should parse");

    assert_eq!(result.warnings, Vec::<String>::new());
    assert_eq!(result.apps.len(), 2);
    assert_eq!(result.apps[0].display_name, "Safari");
    assert_eq!(
        result.apps[0].identity.path,
        PathBuf::from("/Applications/Safari.app")
    );
    assert_eq!(
        result.apps[0].identity.bundle_id.as_deref(),
        Some("com.apple.Safari")
    );
    assert_eq!(result.apps[1].display_name, "Mail");
    assert_eq!(
        result.apps[1].identity.path,
        PathBuf::from("/System/Applications/Mail.app")
    );
}

#[test]
fn dock_import_skips_invalid_entries_and_collects_warnings() {
    let fixture = include_bytes!("fixtures/dock_prefs_sample.plist");

    let result = DockImportReader::default()
        .parse_plist_bytes(fixture)
        .expect("fixture should parse");

    assert_eq!(result.apps.len(), 2);
    assert_eq!(result.warnings.len(), 1);
    assert!(
        result.warnings[0].contains("broken entry"),
        "unexpected warning: {}",
        result.warnings[0]
    );
}
