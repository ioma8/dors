use std::path::PathBuf;

use dors_tauri_lib::config::{ConfigLoad, ConfigStore, DockConfig};
use dors_tauri_lib::domain::{AppIdentity, PinnedApp};

fn sample_pinned_app(path: PathBuf) -> PinnedApp {
    PinnedApp {
        identity: AppIdentity {
            bundle_id: Some("com.apple.Safari".to_string()),
            path,
        },
        display_name: "Safari".to_string(),
    }
}

#[test]
fn config_store_reports_missing_config_on_first_run() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(tempdir.path().to_path_buf());

    let load = store.load().expect("load should succeed");

    assert!(matches!(load, ConfigLoad::Missing));
}

#[test]
fn config_store_round_trips_persisted_pinned_apps() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(tempdir.path().to_path_buf());
    let config = DockConfig {
        pinned_apps: vec![sample_pinned_app(PathBuf::from("/Applications/Safari.app"))],
    };

    store.save(&config).expect("save should succeed");

    let load = store.load().expect("load should succeed");

    assert_eq!(load, ConfigLoad::Loaded(config));
}

#[test]
fn config_store_serializes_identity_and_order_stably() {
    let config = DockConfig {
        pinned_apps: vec![
            sample_pinned_app(PathBuf::from("/Applications/Safari.app")),
            PinnedApp {
                identity: AppIdentity {
                    bundle_id: Some("com.apple.mail".to_string()),
                    path: PathBuf::from("/System/Applications/Mail.app"),
                },
                display_name: "Mail".to_string(),
            },
        ],
    };

    let serialized = serde_json::to_string_pretty(&config).expect("serialize");

    assert_eq!(
        serialized,
        r#"{
  "pinned_apps": [
    {
      "identity": {
        "bundle_id": "com.apple.Safari",
        "path": "/Applications/Safari.app"
      },
      "display_name": "Safari"
    },
    {
      "identity": {
        "bundle_id": "com.apple.mail",
        "path": "/System/Applications/Mail.app"
      },
      "display_name": "Mail"
    }
  ]
}"#
    );
}
