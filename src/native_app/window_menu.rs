#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoveredWindow {
    pub index: usize,
    pub title: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HoverToken {
    version: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoverDelayState {
    delay_millis: u64,
    current_item: Option<usize>,
    version: u64,
}

impl HoveredWindow {
    pub fn new(index: usize, title: &str) -> Self {
        Self {
            index,
            title: title.to_string(),
        }
    }
}

impl HoverDelayState {
    pub fn new() -> Self {
        Self {
            delay_millis: 180,
            current_item: None,
            version: 0,
        }
    }

    pub fn delay_millis(&self) -> u64 {
        self.delay_millis
    }

    pub fn current_item(&self) -> Option<usize> {
        self.current_item
    }

    pub fn schedule_for_item(&mut self, item_index: usize) -> HoverToken {
        self.version += 1;
        self.current_item = Some(item_index);
        HoverToken {
            version: self.version,
        }
    }

    pub fn cancel(&mut self) {
        self.version += 1;
        self.current_item = None;
    }

    pub fn is_current(&self, token: HoverToken, item_index: usize) -> bool {
        self.current_item == Some(item_index) && self.version == token.version
    }
}

pub fn should_show_window_menu(window_count: usize) -> bool {
    window_count > 1
}

pub fn filtered_hovered_windows(windows: &[HoveredWindow]) -> Vec<HoveredWindow> {
    windows
        .iter()
        .filter(|window| !window.title.trim().is_empty())
        .cloned()
        .collect()
}

pub fn activation_script_for_window(process_name: &str, window_title: &str) -> String {
    format!(
        "tell application \"System Events\"\n\
tell application process \"{process_name}\"\n\
set frontmost to true\n\
set targetWindow to first window whose name is \"{window_title}\"\n\
set value of attribute \"AXMain\" of targetWindow to true\n\
set value of attribute \"AXFocused\" of targetWindow to true\n\
perform action \"AXRaise\" of targetWindow\n\
end tell\n\
end tell"
    )
}

pub fn parse_window_title_lines(raw: &str) -> Vec<HoveredWindow> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(index, title)| HoveredWindow::new(index, title))
        .collect()
}

pub fn popup_anchor_y(button_origin_y: f64, button_height: f64) -> f64 {
    button_origin_y + button_height + 8.0
}

pub fn hover_menu_dismiss_delay_millis() -> u64 {
    260
}

#[cfg(target_os = "macos")]
pub fn read_windows_for_app(
    bundle_id: Option<&str>,
    fallback_process_name: &str,
) -> Result<Vec<HoveredWindow>, String> {
    let pid = resolve_process_id(bundle_id, fallback_process_name)?;
    let windows = crate::native_app::accessibility::copy_windows(pid)?;
    let mut hovered_windows = Vec::new();

    for (index, window) in windows.iter().enumerate() {
        let title = crate::native_app::accessibility::copy_string_attribute(window.as_ptr(), "AXTitle")?
            .unwrap_or_default();
        if !title.trim().is_empty() {
            hovered_windows.push(HoveredWindow::new(index, &title));
        }
    }

    Ok(hovered_windows)
}

#[cfg(target_os = "macos")]
pub fn activate_specific_window(
    bundle_id: Option<&str>,
    fallback_process_name: &str,
    window_title: &str,
) -> Result<(), String> {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
    use objc2_foundation::NSString;

    let pid = resolve_process_id(bundle_id, fallback_process_name)?;
    let windows = crate::native_app::accessibility::copy_windows(pid)?;
    let target_window = windows
        .iter()
        .find_map(|window| {
            let title =
                crate::native_app::accessibility::copy_string_attribute(window.as_ptr(), "AXTitle").ok()??;
            (title == window_title).then_some(window.as_ptr())
        })
        .ok_or_else(|| "failed to find target window".to_string())?;

    if let Some(bundle_id) = bundle_id {
        let bundle_id = NSString::from_str(bundle_id);
        let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
        if let Some(application) = applications.firstObject() {
            let _ = application.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
            if application.isHidden() {
                let _ = application.unhide();
            }
        }
    } else {
        let workspace = NSWorkspace::sharedWorkspace();
        for application in workspace.runningApplications().iter() {
            if application.processIdentifier() == pid {
                let _ = application.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
                if application.isHidden() {
                    let _ = application.unhide();
                }
                break;
            }
        }
    }

    crate::native_app::accessibility::set_bool_attribute(target_window, "AXMain", true)?;
    crate::native_app::accessibility::set_bool_attribute(target_window, "AXFocused", true)?;
    crate::native_app::accessibility::perform_action(target_window, "AXRaise")?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn resolve_process_id(bundle_id: Option<&str>, fallback_process_name: &str) -> Result<i32, String> {
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};
    use objc2_foundation::NSString;

    if let Some(bundle_id) = bundle_id {
        let bundle_id = NSString::from_str(bundle_id);
        let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
        if let Some(application) = applications.firstObject() {
            return Ok(application.processIdentifier());
        }
    }

    if fallback_process_name.trim().is_empty() {
        return Err("missing process name".to_string());
    }

    let workspace = NSWorkspace::sharedWorkspace();
    for application in workspace.runningApplications().iter() {
        if application
            .localizedName()
            .map(|name| name.to_string() == fallback_process_name)
            .unwrap_or(false)
        {
            return Ok(application.processIdentifier());
        }
    }

    Err("missing process id".to_string())
}
