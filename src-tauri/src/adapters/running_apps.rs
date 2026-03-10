use std::path::PathBuf;

use crate::domain::{AppIdentity, RunningApp};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunningAppSnapshot {
    pub bundle_id: Option<String>,
    pub display_name: String,
    pub path: PathBuf,
    pub activation_policy_regular: bool,
    pub is_active: bool,
}

pub fn normalize_running_apps(apps: Vec<RunningAppSnapshot>) -> Vec<RunningApp> {
    apps.into_iter()
        .filter(|app| app.activation_policy_regular)
        .map(|app| RunningApp {
            identity: AppIdentity {
                bundle_id: app.bundle_id,
                path: normalize_app_path(app.path),
            },
            display_name: app.display_name,
            is_active: app.is_active,
        })
        .collect()
}

fn normalize_app_path(path: PathBuf) -> PathBuf {
    let path_string = path.to_string_lossy();
    let trimmed = path_string.trim_end_matches('/');
    PathBuf::from(trimmed)
}
