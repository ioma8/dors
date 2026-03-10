use std::process::Command;
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

pub fn read_running_apps() -> Result<Vec<RunningApp>, String> {
    let raw = Command::new("lsappinfo")
        .arg("list")
        .output()
        .map_err(|error| format!("failed to execute lsappinfo: {error}"))?;
    if !raw.status.success() {
        return Err("lsappinfo list failed".to_string());
    }

    let frontmost = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ])
        .output()
        .map_err(|error| format!("failed to execute osascript: {error}"))?;
    if !frontmost.status.success() {
        return Err("failed to determine frontmost app".to_string());
    }

    let list = String::from_utf8_lossy(&raw.stdout);
    let active_name = String::from_utf8_lossy(&frontmost.stdout).trim().to_string();
    Ok(normalize_running_apps(parse_lsappinfo_list(&list, &active_name)))
}

pub fn parse_lsappinfo_list(raw: &str, active_name: &str) -> Vec<RunningAppSnapshot> {
    let mut apps = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_bundle_id: Option<String> = None;
    let mut current_path: Option<PathBuf> = None;
    let mut current_regular = false;

    for line in raw.lines() {
        if let Some(name) = parse_entry_name(line) {
            push_snapshot(
                &mut apps,
                current_name.take(),
                current_bundle_id.take(),
                current_path.take(),
                current_regular,
                active_name,
            );
            current_name = Some(name);
            current_regular = false;
            continue;
        }

        let trimmed = line.trim();
        if let Some(bundle_id) = trimmed.strip_prefix("bundleID=\"") {
            current_bundle_id = bundle_id.strip_suffix('"').map(ToString::to_string);
            continue;
        }
        if let Some(path) = trimmed.strip_prefix("bundle path=\"") {
            current_path = path.strip_suffix('"').map(PathBuf::from);
            continue;
        }
        if trimmed.contains("type=\"Foreground\"") {
            current_regular = true;
        }
    }

    push_snapshot(
        &mut apps,
        current_name,
        current_bundle_id,
        current_path,
        current_regular,
        active_name,
    );

    apps
}

fn normalize_app_path(path: PathBuf) -> PathBuf {
    let path_string = path.to_string_lossy();
    let trimmed = path_string.trim_end_matches('/');
    PathBuf::from(trimmed)
}

fn parse_entry_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.contains("ASN:") || !trimmed.contains(") \"") {
        return None;
    }
    let first_quote = trimmed.find('"')?;
    let rest = &trimmed[first_quote + 1..];
    let second_quote = rest.find('"')?;
    Some(rest[..second_quote].to_string())
}

fn push_snapshot(
    apps: &mut Vec<RunningAppSnapshot>,
    name: Option<String>,
    bundle_id: Option<String>,
    path: Option<PathBuf>,
    activation_policy_regular: bool,
    active_name: &str,
) {
    let Some(display_name) = name else {
        return;
    };
    let Some(path) = path else {
        return;
    };
    if !activation_policy_regular {
        return;
    }

    apps.push(RunningAppSnapshot {
        bundle_id,
        display_name: display_name.clone(),
        path,
        activation_policy_regular,
        is_active: display_name == active_name,
    });
}
