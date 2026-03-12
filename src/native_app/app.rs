#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSScreen};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;

use crate::native_app::dock_view::required_height;
use crate::native_app::interaction::DockController;
use crate::native_app::layout::bottom_center_panel_placement;
use crate::native_app::panel::build_overlay_panel;
use crate::native_app::refresh::load_startup_models;
use crate::native_app::system_dock::{
    install_restore_signal_handler, prepare_overlay_dock_mode, restore_shared_guard, shared_guard,
};
use crate::window_level::OVERLAY_WINDOW_LEVEL;

const DEFAULT_PANEL_WIDTH: u32 = 1180;

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
        panel_height: required_height(),
    }
}

#[cfg(target_os = "macos")]
pub fn run() -> Result<(), String> {
    let marker = MainThreadMarker::new()
        .ok_or_else(|| "native app bootstrap must run on the main thread".to_string())?;
    let config = startup_configuration();
    let dock_guard =
        prepare_overlay_dock_mode(config.panel_height as i32).map_err(|error| error.to_string())?;
    let shared_dock_guard = shared_guard(dock_guard);
    install_restore_signal_handler(shared_dock_guard.clone()).map_err(|error| error.to_string())?;
    let app = NSApplication::sharedApplication(marker);
    let screen =
        NSScreen::mainScreen(marker).ok_or_else(|| "no main screen available".to_string())?;
    let frame = screen.frame();
    let placement = bottom_center_panel_placement(
        frame.origin.x as i32,
        frame.origin.y as i32,
        frame.size.width as u32,
        frame.size.height as u32,
        config.panel_width,
        config.panel_height,
    );
    let panel = build_overlay_panel(placement, config.panel_width, config.panel_height)?;
    let models = load_startup_models().unwrap_or_default();
    let controller = DockController::new(
        marker,
        panel,
        config.panel_width,
        config.panel_height,
        models,
    );
    controller.render_current_models()?;
    let _timer = controller.install_refresh_timer();

    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    app.run();
    restore_shared_guard(&shared_dock_guard).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn run() -> Result<(), String> {
    Err("native dock runtime is only available on macOS".to_string())
}
