use std::path::Path;

use crate::config::DockConfig;
use crate::domain::{AppIdentity, DockItemView, RunningApp};

pub fn build_dock_items(config: &DockConfig, running_apps: &[RunningApp]) -> Vec<DockItemView> {
    let mut items = Vec::new();

    for pinned in &config.pinned_apps {
        let running_match = running_apps
            .iter()
            .find(|running| matches_identity(&pinned.identity, &running.identity));

        items.push(DockItemView {
            identity: pinned.identity.clone(),
            display_name: pinned.display_name.clone(),
            icon_src: String::new(),
            is_pinned: true,
            is_running: running_match.is_some(),
            is_active: running_match.is_some_and(|app| app.is_active),
            is_degraded: pinned.identity.path.as_os_str().is_empty(),
        });
    }

    for running in running_apps {
        if config
            .pinned_apps
            .iter()
            .any(|pinned| matches_identity(&pinned.identity, &running.identity))
        {
            continue;
        }

        items.push(DockItemView {
            identity: running.identity.clone(),
            display_name: running.display_name.clone(),
            icon_src: String::new(),
            is_pinned: false,
            is_running: true,
            is_active: running.is_active,
            is_degraded: false,
        });
    }

    items
}

fn matches_identity(left: &AppIdentity, right: &AppIdentity) -> bool {
    match (&left.bundle_id, &right.bundle_id) {
        (Some(left_id), Some(right_id)) => left_id == right_id,
        _ => normalize_path(&left.path) == normalize_path(&right.path),
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().trim_end_matches('/').to_string()
}
