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
perform action \"AXRaise\" of first window whose name is \"{window_title}\"\n\
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

#[cfg(target_os = "macos")]
pub fn read_windows_for_app(
    bundle_id: Option<&str>,
    fallback_process_name: &str,
) -> Result<Vec<HoveredWindow>, String> {
    let process_name = resolve_process_name(bundle_id, fallback_process_name)?;
    let script = format!(
        "tell application \"System Events\"\n\
tell application process \"{process_name}\"\n\
set windowNames to name of every window\n\
end tell\n\
end tell\n\
set AppleScript's text item delimiters to linefeed\n\
return windowNames as text"
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(parse_window_title_lines(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(target_os = "macos")]
pub fn activate_specific_window(
    bundle_id: Option<&str>,
    fallback_process_name: &str,
    window_title: &str,
) -> Result<(), String> {
    let process_name = resolve_process_name(bundle_id, fallback_process_name)?;
    let script = activation_script_for_window(&process_name, window_title);
    let status = Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|error| error.to_string())?;

    if status.success() {
        return Ok(());
    }

    Err("failed to activate specific window".to_string())
}

#[cfg(target_os = "macos")]
fn resolve_process_name(bundle_id: Option<&str>, fallback_process_name: &str) -> Result<String, String> {
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;

    if let Some(bundle_id) = bundle_id {
        let bundle_id = NSString::from_str(bundle_id);
        let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
        if let Some(application) = applications.firstObject() {
            if let Some(name) = application.localizedName() {
                return Ok(name.to_string());
            }
        }
    }

    if fallback_process_name.trim().is_empty() {
        return Err("missing process name".to_string());
    }

    Ok(fallback_process_name.to_string())
}
#[cfg(target_os = "macos")]
use std::process::Command;
