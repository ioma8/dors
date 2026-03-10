use std::fs;
use std::path::PathBuf;

use dors_tauri_lib::adapters::icon_loader::resolve_icon_path_from_bundle;

#[test]
fn icon_loader_resolves_cf_bundle_icon_file_without_extension() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let app_path = temp_dir.path().join("Notes.app");
    let contents_path = app_path.join("Contents");
    let resources_path = contents_path.join("Resources");

    fs::create_dir_all(&resources_path).expect("resources dir");
    fs::write(
        contents_path.join("Info.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
</dict>
</plist>
"#,
    )
    .expect("plist");
    fs::write(resources_path.join("AppIcon.icns"), b"fake icon").expect("icon");

    assert_eq!(
        resolve_icon_path_from_bundle(&app_path),
        Some(PathBuf::from("Contents/Resources/AppIcon.icns"))
    );
}
