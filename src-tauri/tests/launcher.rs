use std::path::PathBuf;

use dors_tauri_lib::services::launcher::{
    LaunchAction, LaunchRequest, LaunchResult, activation_script, launch_or_activate,
};

#[test]
fn launcher_activates_running_apps() {
    let request = LaunchRequest {
        bundle_id: Some("com.apple.Safari".to_string()),
        path: PathBuf::from("/Applications/Safari.app"),
        is_running: true,
    };

    let result = launch_or_activate(
        &request,
        |_req| LaunchResult::Activated,
        |_req| {
            panic!("launch fallback should not be used");
        },
    );

    assert_eq!(result, LaunchAction::Activate);
}

#[test]
fn launcher_launches_non_running_apps() {
    let request = LaunchRequest {
        bundle_id: Some("com.apple.mail".to_string()),
        path: PathBuf::from("/System/Applications/Mail.app"),
        is_running: false,
    };

    let result = launch_or_activate(
        &request,
        |_req| LaunchResult::ActivationFailed,
        |_req| LaunchResult::Launched,
    );

    assert_eq!(result, LaunchAction::Launch);
}

#[test]
fn launcher_uses_fallback_launch_when_activation_fails() {
    let request = LaunchRequest {
        bundle_id: Some("com.apple.finder".to_string()),
        path: PathBuf::from("/System/Library/CoreServices/Finder.app"),
        is_running: true,
    };

    let result = launch_or_activate(
        &request,
        |_req| LaunchResult::ActivationFailed,
        |_req| LaunchResult::Launched,
    );

    assert_eq!(result, LaunchAction::LaunchFallback);
}

#[test]
fn finder_activation_script_reopens_a_window() {
    let request = LaunchRequest {
        bundle_id: Some("com.apple.finder".to_string()),
        path: PathBuf::from("/System/Library/CoreServices/Finder.app"),
        is_running: true,
    };

    assert_eq!(
        activation_script(&request),
        Some(
            "tell application \"Finder\" to activate\n\
tell application \"Finder\" to reopen"
                .to_string()
        )
    );
}
