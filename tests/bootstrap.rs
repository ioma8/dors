use std::path::PathBuf;

use dors::adapters::dock_import::DockImportResult;
use dors::app_state::bootstrap_pinned_apps;
use dors::config::{ConfigLoad, DockConfig};
use dors::domain::{AppIdentity, PinnedApp};

fn sample_import() -> DockImportResult {
    DockImportResult {
        apps: vec![PinnedApp {
            identity: AppIdentity {
                bundle_id: Some("com.apple.Safari".to_string()),
                path: PathBuf::from("/Applications/Safari.app"),
            },
            display_name: "Safari".to_string(),
        }],
        warnings: Vec::new(),
    }
}

#[test]
fn bootstrap_imports_pinned_apps_on_first_run() {
    let imported = sample_import();

    let result = bootstrap_pinned_apps(
        ConfigLoad::Missing,
        || Ok(imported.clone()),
        |_config| Ok(()),
    )
    .expect("bootstrap should succeed");

    assert_eq!(result.config.pinned_apps, imported.apps);
    assert!(result.imported_on_first_run);
}

#[test]
fn bootstrap_uses_persisted_config_without_reimporting() {
    let existing = DockConfig {
        pinned_apps: vec![PinnedApp {
            identity: AppIdentity {
                bundle_id: Some("com.apple.mail".to_string()),
                path: PathBuf::from("/System/Applications/Mail.app"),
            },
            display_name: "Mail".to_string(),
        }],
    };

    let result = bootstrap_pinned_apps(
        ConfigLoad::Loaded(existing.clone()),
        || panic!("import should not be called"),
        |_config| Ok(()),
    )
    .expect("bootstrap should succeed");

    assert_eq!(result.config, existing);
    assert!(!result.imported_on_first_run);
}
