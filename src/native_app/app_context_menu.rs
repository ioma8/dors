#[cfg(target_os = "macos")]
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString, NSRunningApplication, NSWorkspace};
#[cfg(target_os = "macos")]
use objc2_foundation::NSString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppContextAction {
    Kill,
    ForceKill,
    CopyPid,
}

pub fn context_action_tag(action: AppContextAction) -> isize {
    match action {
        AppContextAction::Kill => 1,
        AppContextAction::ForceKill => 2,
        AppContextAction::CopyPid => 3,
    }
}

pub fn action_from_tag(tag: isize) -> Option<AppContextAction> {
    match tag {
        1 => Some(AppContextAction::Kill),
        2 => Some(AppContextAction::ForceKill),
        3 => Some(AppContextAction::CopyPid),
        _ => None,
    }
}

pub fn context_action_title(action: AppContextAction) -> &'static str {
    match action {
        AppContextAction::Kill => "Kill App",
        AppContextAction::ForceKill => "Force Kill App",
        AppContextAction::CopyPid => "Copy PID",
    }
}

#[cfg(target_os = "macos")]
pub fn running_application_pid(bundle_id: Option<&str>, path: &std::path::Path) -> Option<i32> {
    if let Some(bundle_id) = bundle_id {
        let bundle_id = NSString::from_str(bundle_id);
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
        if let Some(app) = apps.firstObject() {
            return Some(app.processIdentifier());
        }
    }

    let target_path = path.to_string_lossy().to_string();
    let workspace = NSWorkspace::sharedWorkspace();
    for candidate in workspace.runningApplications().iter() {
        if let Some(url) = candidate.bundleURL()
            && let Some(candidate_path) = url.path()
            && candidate_path.to_string() == target_path
        {
            return Some(candidate.processIdentifier());
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn perform_context_action(action: AppContextAction, pid: i32) -> Result<(), String> {
    match action {
        AppContextAction::Kill => terminate_application(pid, false),
        AppContextAction::ForceKill => terminate_application(pid, true),
        AppContextAction::CopyPid => copy_pid_to_clipboard(pid),
    }
}

#[cfg(target_os = "macos")]
fn terminate_application(pid: i32, force: bool) -> Result<(), String> {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return Err(format!("running app for pid {pid} not found"));
    };
    let success = if force {
        app.forceTerminate()
    } else {
        app.terminate()
    };
    if success {
        Ok(())
    } else {
        Err(format!("failed to terminate pid {pid}"))
    }
}

#[cfg(target_os = "macos")]
fn copy_pid_to_clipboard(pid: i32) -> Result<(), String> {
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    let value = NSString::from_str(&pid.to_string());
    let string_type = unsafe { NSPasteboardTypeString };
    if pasteboard.setString_forType(&value, string_type) {
        Ok(())
    } else {
        Err("failed to copy pid to clipboard".to_string())
    }
}
