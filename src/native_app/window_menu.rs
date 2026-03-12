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
