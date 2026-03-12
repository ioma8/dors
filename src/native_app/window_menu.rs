#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoveredWindow {
    pub index: usize,
    pub title: String,
}

impl HoveredWindow {
    pub fn new(index: usize, title: &str) -> Self {
        Self {
            index,
            title: title.to_string(),
        }
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
