use std::path::PathBuf;

use dors::adapters::dock_import::DockImportReader;

#[test]
fn dock_import_parses_pinned_apps_in_order_from_fixture() {
    let fixture = include_bytes!("fixtures/dock_prefs_valid_sample.plist");

    let reader = DockImportReader;
    let result = reader
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
fn dock_import_normalizes_file_urls_to_application_paths() {
    let fixture = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>persistent-apps</key>
  <array>
    <dict>
      <key>tile-data</key>
      <dict>
        <key>bundle-identifier</key>
        <string>com.apple.Notes</string>
        <key>file-label</key>
        <string>Notes</string>
        <key>file-data</key>
        <dict>
          <key>_CFURLString</key>
          <string>file:///System/Applications/Notes.app/</string>
          <key>_CFURLStringType</key>
          <integer>15</integer>
        </dict>
      </dict>
    </dict>
  </array>
</dict>
</plist>"#;

    let reader = DockImportReader;
    let result = reader
        .parse_plist_bytes(fixture)
        .expect("fixture should parse");

    assert_eq!(result.apps.len(), 1);
    assert_eq!(
        result.apps[0].identity.path,
        PathBuf::from("/System/Applications/Notes.app")
    );
}

#[test]
fn dock_import_skips_invalid_entries_and_collects_warnings() {
    let fixture = include_bytes!("fixtures/dock_prefs_sample.plist");

    let reader = DockImportReader;
    let result = reader
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
