#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkingArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowCandidate {
    pub owner_name: String,
    pub frame: WindowFrame,
    pub is_standard: bool,
    pub is_resizable: bool,
    pub is_fullscreen: bool,
    pub is_visible: bool,
}

pub fn build_allowed_work_area(
    screen: ScreenFrame,
    top_reserved_height: i32,
    dock_height: i32,
) -> WorkingArea {
    let y = screen.y + top_reserved_height.max(0);
    let height = (screen.height - top_reserved_height.max(0) - dock_height.max(0)).max(0);

    WorkingArea {
        x: screen.x,
        y,
        width: screen.width.max(0),
        height,
    }
}

pub fn clamp_window_frame(frame: WindowFrame, area: WorkingArea) -> Option<WindowFrame> {
    let clamped_height = frame.height.min(area.height).max(0);
    let top = area.y + area.height;
    let max_y = top - clamped_height;
    let clamped_y = frame.y.clamp(area.y, max_y);
    let clamped = WindowFrame {
        x: frame.x,
        y: clamped_y,
        width: frame.width,
        height: clamped_height,
    };

    (clamped != frame).then_some(clamped)
}

pub fn should_clamp_candidate(candidate: &WindowCandidate, screen: ScreenFrame) -> bool {
    candidate.is_standard
        && candidate.is_resizable
        && candidate.is_visible
        && !candidate.is_fullscreen
        && candidate.frame.width >= 240
        && candidate.frame.height >= 160
        && intersects_screen(candidate.frame, screen)
}

fn intersects_screen(frame: WindowFrame, screen: ScreenFrame) -> bool {
    let frame_right = frame.x + frame.width;
    let frame_top = frame.y + frame.height;
    let screen_right = screen.x + screen.width;
    let screen_top = screen.y + screen.height;

    frame.x < screen_right && frame_right > screen.x && frame.y < screen_top && frame_top > screen.y
}

#[cfg(target_os = "macos")]
pub fn clamp_windows_in_area(area: WorkingArea) -> Result<(), String> {
    let script = build_clamp_script(area);
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

pub fn build_clamp_script_preview(area: WorkingArea) -> String {
    build_clamp_script(area)
}

#[cfg(target_os = "macos")]
pub fn main_screen_allowed_work_area(dock_height: i32) -> Result<WorkingArea, String> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "screen measurement requires main thread".to_string())?;
    let screen =
        NSScreen::mainScreen(marker).ok_or_else(|| "no main screen available".to_string())?;
    let frame = screen.frame();
    let visible = screen.visibleFrame();
    let bottom_reserved = (visible.origin.y - frame.origin.y).round() as i32;
    let total_reserved = (frame.size.height - visible.size.height).round() as i32;
    let top_reserved = (total_reserved - bottom_reserved).max(0);

    Ok(build_allowed_work_area(
        ScreenFrame {
            x: 0,
            y: 0,
            width: frame.size.width.round() as i32,
            height: frame.size.height.round() as i32,
        },
        top_reserved,
        dock_height,
    ))
}

#[cfg(target_os = "macos")]
fn build_clamp_script(area: WorkingArea) -> String {
    let screen_right = area.x + area.width;
    let screen_bottom = area.y + area.height;

    format!(
        "tell application \"System Events\"\n\
repeat with proc in application processes\n\
if background only of proc is false and visible of proc is true and name of proc is not \"dors\" then\n\
repeat with win in windows of proc\n\
try\n\
set isStandard to true\n\
try\n\
set isStandard to (value of attribute \"AXSubrole\" of win is \"AXStandardWindow\")\n\
end try\n\
if isStandard then\n\
set isFullScreen to false\n\
try\n\
set isFullScreen to value of attribute \"AXFullScreen\" of win\n\
end try\n\
set {{xPos, yPos}} to position of win\n\
set {{winWidth, winHeight}} to size of win\n\
if isFullScreen is false and winWidth >= 240 and winHeight >= 160 and xPos < {screen_right} and (xPos + winWidth) > {screen_left} and yPos < {screen_bottom} and (yPos + winHeight) > {screen_top} then\n\
set newHeight to winHeight\n\
if newHeight > {area_height} then set newHeight to {area_height}\n\
set newY to yPos\n\
set maxY to {max_y_base} - newHeight\n\
if newY < {screen_top} then set newY to {screen_top}\n\
if newY > maxY then set newY to maxY\n\
if newHeight is not winHeight then set size of win to {{winWidth, newHeight}}\n\
if newY is not yPos then set position of win to {{xPos, newY}}\n\
end if\n\
end if\n\
end try\n\
end repeat\n\
end if\n\
end repeat\n\
end tell",
        screen_left = area.x,
        screen_top = area.y,
        area_height = area.height,
        max_y_base = area.y + area.height,
    )
}
#[cfg(target_os = "macos")]
use std::process::Command;
