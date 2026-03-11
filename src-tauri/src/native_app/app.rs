#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSScreen};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;

use crate::native_app::layout::bottom_center_panel_placement;
use crate::native_app::panel::build_overlay_panel;
use crate::window_level::OVERLAY_WINDOW_LEVEL;

const DEFAULT_PANEL_WIDTH: u32 = 1180;
const DEFAULT_PANEL_HEIGHT: u32 = 168;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupConfiguration {
    pub activation_policy: &'static str,
    pub panel_level: isize,
    pub panel_width: u32,
    pub panel_height: u32,
}

pub fn startup_configuration() -> StartupConfiguration {
    StartupConfiguration {
        activation_policy: "accessory",
        panel_level: OVERLAY_WINDOW_LEVEL,
        panel_width: DEFAULT_PANEL_WIDTH,
        panel_height: DEFAULT_PANEL_HEIGHT,
    }
}

#[cfg(target_os = "macos")]
pub fn run() -> Result<(), String> {
    let marker = MainThreadMarker::new()
        .ok_or_else(|| "native app bootstrap must run on the main thread".to_string())?;
    let app = NSApplication::sharedApplication(marker);
    let screen =
        NSScreen::mainScreen(marker).ok_or_else(|| "no main screen available".to_string())?;
    let frame = screen.frame();
    let config = startup_configuration();
    let placement = bottom_center_panel_placement(
        frame.origin.x as i32,
        frame.origin.y as i32,
        frame.size.width as u32,
        frame.size.height as u32,
        config.panel_width,
        config.panel_height,
    );
    let _panel = build_overlay_panel(placement, config.panel_width, config.panel_height)?;

    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    app.run();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn run() -> Result<(), String> {
    Err("native dock runtime is only available on macOS".to_string())
}
