use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LaunchRequest {
    pub bundle_id: Option<String>,
    pub path: PathBuf,
    pub is_running: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LaunchResult {
    Activated,
    ActivationFailed,
    Launched,
    LaunchFailed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LaunchAction {
    Activate,
    Launch,
    LaunchFallback,
    NoOp,
}

pub fn launch_or_activate<Activate, Launch>(
    request: &LaunchRequest,
    activate: Activate,
    launch: Launch,
) -> LaunchAction
where
    Activate: Fn(&LaunchRequest) -> LaunchResult,
    Launch: Fn(&LaunchRequest) -> LaunchResult,
{
    eprintln!(
        "[dors-debug] trigger_launch request bundle_id={:?} path={} is_running={} frontmost_before={:?}",
        request.bundle_id,
        request.path.display(),
        request.is_running,
        frontmost_app_name()
    );

    let action = if request.is_running {
        match activate(request) {
            LaunchResult::Activated => LaunchAction::Activate,
            LaunchResult::ActivationFailed => match launch(request) {
                LaunchResult::Launched => LaunchAction::LaunchFallback,
                LaunchResult::LaunchFailed
                | LaunchResult::Activated
                | LaunchResult::ActivationFailed => LaunchAction::NoOp,
            },
            LaunchResult::Launched => LaunchAction::Launch,
            LaunchResult::LaunchFailed => LaunchAction::NoOp,
        }
    } else {
        match launch(request) {
            LaunchResult::Launched => LaunchAction::Launch,
            LaunchResult::Activated => LaunchAction::Activate,
            LaunchResult::ActivationFailed | LaunchResult::LaunchFailed => LaunchAction::NoOp,
        }
    };

    eprintln!(
        "[dors-debug] trigger_launch result action={action:?} frontmost_after={:?}",
        frontmost_app_name()
    );
    action
}

pub fn activate_app(request: &LaunchRequest) -> LaunchResult {
    #[cfg(target_os = "macos")]
    if let LaunchResult::Activated = activate_app_natively(request) {
        return LaunchResult::Activated;
    }

    let Some(script) = activation_script(request) else {
        return LaunchResult::ActivationFailed;
    };

    match Command::new("osascript").args(["-e", &script]).status() {
        Ok(status) if status.success() => LaunchResult::Activated,
        Ok(_) | Err(_) => LaunchResult::ActivationFailed,
    }
}

pub fn launch_app(request: &LaunchRequest) -> LaunchResult {
    match Command::new("open")
        .args(["-a"])
        .arg(&request.path)
        .status()
    {
        Ok(status) if status.success() => LaunchResult::Launched,
        Ok(_) | Err(_) => LaunchResult::LaunchFailed,
    }
}

pub fn activation_script(request: &LaunchRequest) -> Option<String> {
    let bundle_id = request.bundle_id.as_deref()?;

    if bundle_id == "com.apple.finder" {
        return Some(
            "tell application \"Finder\" to activate\n\
tell application \"Finder\" to reopen"
                .to_string(),
        );
    }

    Some(format!(
        "tell application id \"{bundle_id}\" to activate\n\
tell application id \"{bundle_id}\" to reopen"
    ))
}

fn frontmost_app_name() -> Option<String> {
    #[cfg(target_os = "macos")]
    if let Some(name) = native_frontmost_app_name() {
        return Some(name);
    }

    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!name.is_empty()).then_some(name)
}

#[cfg(target_os = "macos")]
fn activate_app_natively(request: &LaunchRequest) -> LaunchResult {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::NSString;

    let Some(bundle_id) = request.bundle_id.as_deref() else {
        return LaunchResult::ActivationFailed;
    };

    let bundle_id = NSString::from_str(bundle_id);
    let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
    let Some(application) = applications.firstObject() else {
        return LaunchResult::ActivationFailed;
    };

    let activation_options = NSApplicationActivationOptions::ActivateAllWindows;
    if application.activateWithOptions(activation_options) {
        if application.isHidden() {
            let _ = application.unhide();
        }
        return LaunchResult::Activated;
    }

    LaunchResult::ActivationFailed
}

#[cfg(target_os = "macos")]
fn native_frontmost_app_name() -> Option<String> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let application = workspace.frontmostApplication()?;
    let name = application.localizedName()?;
    Some(name.to_string())
}
