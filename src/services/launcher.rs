use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrontmostApp {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
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
    let frontmost_before = frontmost_app();
    eprintln!(
        "[dors-debug] trigger_launch request bundle_id={:?} path={} is_running={} frontmost_before={:?}",
        request.bundle_id,
        request.path.display(),
        request.is_running,
        frontmost_before.as_ref().and_then(|app| app.name.clone())
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

    let frontmost_after = settled_frontmost_app(frontmost_before.as_ref(), request, action);
    eprintln!(
        "[dors-debug] trigger_launch result action={action:?} frontmost_after={:?}",
        frontmost_after.as_ref().and_then(|app| app.name.clone())
    );
    action
}

pub fn select_authoritative_frontmost(
    before: Option<&FrontmostApp>,
    expected_bundle: Option<&str>,
    samples: &[Option<FrontmostApp>],
) -> Option<FrontmostApp> {
    for sample in samples {
        if sample.as_ref().and_then(|app| app.bundle_id.as_deref()) == expected_bundle {
            return sample.clone();
        }
    }

    for sample in samples.iter().rev() {
        if sample.as_ref() != before {
            return sample.clone();
        }
    }

    samples
        .last()
        .cloned()
        .flatten()
        .or_else(|| before.cloned())
}

pub fn activate_app(request: &LaunchRequest) -> LaunchResult {
    #[cfg(target_os = "macos")]
    {
        return activate_app_natively(request);
    }

    #[cfg(not(target_os = "macos"))]
    let Some(script) = activation_script(request) else {
        return LaunchResult::ActivationFailed;
    };

    #[cfg(not(target_os = "macos"))]
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

fn settled_frontmost_app(
    before: Option<&FrontmostApp>,
    request: &LaunchRequest,
    action: LaunchAction,
) -> Option<FrontmostApp> {
    #[cfg(target_os = "macos")]
    {
        return settled_frontmost_app_natively(before, request, action);
    }

    #[cfg(not(target_os = "macos"))]
    {
    let needs_handoff = matches!(
        action,
        LaunchAction::Activate | LaunchAction::Launch | LaunchAction::LaunchFallback
    );
    if !needs_handoff {
        return frontmost_app();
    }

    let mut samples = Vec::with_capacity(7);
    for _ in 0..7 {
        let sample = frontmost_app();
        samples.push(sample.clone());
        if sample.as_ref().and_then(|app| app.bundle_id.as_deref()) == request.bundle_id.as_deref()
        {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }

    select_authoritative_frontmost(before, request.bundle_id.as_deref(), &samples)
    }
}

fn frontmost_app() -> Option<FrontmostApp> {
    #[cfg(target_os = "macos")]
    if let Some(app) = native_frontmost_app() {
        return Some(app);
    }

    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\"\n\
set frontApp to first application process whose frontmost is true\n\
try\n\
set bundleId to bundle identifier of frontApp\n\
on error\n\
set bundleId to \"\"\n\
end try\n\
return (name of frontApp) & tab & bundleId\n\
end tell",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut parts = raw.split('\t');
    let name = parts.next().unwrap_or_default().trim().to_string();
    let bundle_id = parts.next().unwrap_or_default().trim().to_string();
    if name.is_empty() && bundle_id.is_empty() {
        return None;
    }
    Some(FrontmostApp {
        bundle_id: (!bundle_id.is_empty()).then_some(bundle_id),
        name: (!name.is_empty()).then_some(name),
    })
}

#[cfg(target_os = "macos")]
fn activate_app_natively(request: &LaunchRequest) -> LaunchResult {
    use objc2_app_kit::{
        NSApplicationActivationOptions, NSRunningApplication, NSWorkspace,
    };
    use objc2_foundation::{NSArray, NSString, NSURL};

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
        if bundle_id.to_string() == "com.apple.finder" {
            let workspace = NSWorkspace::sharedWorkspace();
            let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
            let url = NSURL::fileURLWithPath(&NSString::from_str(&home));
            let urls = NSArray::from_retained_slice(&[url]);
            workspace.activateFileViewerSelectingURLs(&urls);
        }
        return LaunchResult::Activated;
    }

    LaunchResult::ActivationFailed
}

#[cfg(target_os = "macos")]
fn settled_frontmost_app_natively(
    before: Option<&FrontmostApp>,
    request: &LaunchRequest,
    action: LaunchAction,
) -> Option<FrontmostApp> {
    let needs_handoff = matches!(
        action,
        LaunchAction::Activate | LaunchAction::Launch | LaunchAction::LaunchFallback
    );
    if !needs_handoff {
        return native_frontmost_app();
    }

    let mut samples = Vec::with_capacity(10);
    for _ in 0..10 {
        if let Some(bundle_id) = request.bundle_id.as_deref() {
            if let Some(target) = native_running_app_info(bundle_id) {
                if target.is_active() {
                    let sample = frontmost_from_running_application_info(&target);
                    samples.push(sample.clone());
                    return sample;
                }
            }
        }

        let sample = native_frontmost_app();
        samples.push(sample.clone());
        if sample.as_ref().and_then(|app| app.bundle_id.as_deref()) == request.bundle_id.as_deref()
        {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }

    select_authoritative_frontmost(before, request.bundle_id.as_deref(), &samples)
}

#[cfg(target_os = "macos")]
struct RunningApplicationInfo {
    bundle_id: Option<String>,
    name: Option<String>,
    is_active: bool,
}

#[cfg(target_os = "macos")]
impl RunningApplicationInfo {
    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(target_os = "macos")]
fn native_running_app_info(bundle_id: &str) -> Option<RunningApplicationInfo> {
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;

    let bundle_id = NSString::from_str(bundle_id);
    let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
    applications.firstObject().map(|application| RunningApplicationInfo {
        bundle_id: application.bundleIdentifier().map(|value| value.to_string()),
        name: application.localizedName().map(|value| value.to_string()),
        is_active: application.isActive(),
    })
}

#[cfg(target_os = "macos")]
fn frontmost_from_running_application_info(application: &RunningApplicationInfo) -> Option<FrontmostApp> {
    if application.name.is_none() && application.bundle_id.is_none() {
        None
    } else {
        Some(FrontmostApp {
            bundle_id: application.bundle_id.clone(),
            name: application.name.clone(),
        })
    }
}

#[cfg(target_os = "macos")]
fn native_frontmost_app() -> Option<FrontmostApp> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let application = workspace.frontmostApplication()?;
    let name = application.localizedName().map(|value| value.to_string());
    let bundle_id = application
        .bundleIdentifier()
        .map(|value| value.to_string());
    Some(FrontmostApp { bundle_id, name })
}
