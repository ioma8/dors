use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use plist::Value;

use dors::adapters::icon_loader::{
    clear_icon_cache_for_tests, load_icon_data_url_with, resolve_icon_path_from_bundle,
};

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

#[test]
fn icon_loader_caches_data_url_per_app_path() {
    clear_icon_cache_for_tests();

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

    let conversions = Arc::new(AtomicUsize::new(0));
    let converter_count = Arc::clone(&conversions);

    let first = load_icon_data_url_with(&app_path, move |_| {
        converter_count.fetch_add(1, Ordering::SeqCst);
        Ok(vec![1, 2, 3, 4])
    });
    let second = load_icon_data_url_with(&app_path, |_| Ok(vec![9, 9, 9]));

    assert_eq!(first, second);
    assert_eq!(conversions.load(Ordering::SeqCst), 1);
}

#[test]
fn icon_loader_resolves_binary_info_plist_bundles() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let app_path = temp_dir.path().join("Xcode.app");
    let contents_path = app_path.join("Contents");
    let resources_path = contents_path.join("Resources");

    fs::create_dir_all(&resources_path).expect("resources dir");
    let plist_path = contents_path.join("Info.plist");
    let plist_value = Value::Dictionary(plist::Dictionary::from_iter([(
        "CFBundleIconFile".to_string(),
        Value::String("Xcode".to_string()),
    )]));
    plist_value
        .to_file_binary(&plist_path)
        .expect("binary plist written");
    fs::write(resources_path.join("Xcode.icns"), b"fake icon").expect("icon");

    assert_eq!(
        resolve_icon_path_from_bundle(&app_path),
        Some(PathBuf::from("Contents/Resources/Xcode.icns"))
    );
}
