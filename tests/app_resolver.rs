use std::path::PathBuf;

use dors::adapters::app_resolver::{
    AppMetadata, AppResolverRecord, resolve_app_metadata,
};

#[test]
fn app_resolver_returns_name_identity_path_and_icon_reference() {
    let record = AppResolverRecord {
        bundle_id: Some("com.apple.Safari".to_string()),
        display_name: Some("Safari".to_string()),
        path: PathBuf::from("/Applications/Safari.app"),
        icon_path: Some(PathBuf::from(
            "/Applications/Safari.app/Contents/Resources/AppIcon.icns",
        )),
    };

    let metadata = resolve_app_metadata(record);

    assert_eq!(
        metadata,
        AppMetadata {
            bundle_id: Some("com.apple.Safari".to_string()),
            display_name: "Safari".to_string(),
            path: PathBuf::from("/Applications/Safari.app"),
            icon_path: Some(PathBuf::from(
                "/Applications/Safari.app/Contents/Resources/AppIcon.icns",
            )),
        }
    );
}

#[test]
fn app_resolver_falls_back_to_bundle_name_when_display_name_is_missing() {
    let record = AppResolverRecord {
        bundle_id: Some("com.apple.finder".to_string()),
        display_name: None,
        path: PathBuf::from("/System/Library/CoreServices/Finder.app/"),
        icon_path: None,
    };

    let metadata = resolve_app_metadata(record);

    assert_eq!(metadata.display_name, "Finder");
    assert_eq!(
        metadata.path,
        PathBuf::from("/System/Library/CoreServices/Finder.app")
    );
}
