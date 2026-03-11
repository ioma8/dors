use crate::native_app::layout::PanelPlacement;
use crate::window_level::OVERLAY_WINDOW_LEVEL;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanelConfiguration {
    pub level: isize,
    pub transparent: bool,
    pub non_activating: bool,
    pub borderless: bool,
    pub ignores_mouse_events: bool,
}

pub fn panel_configuration() -> PanelConfiguration {
    PanelConfiguration {
        level: OVERLAY_WINDOW_LEVEL,
        transparent: true,
        non_activating: true,
        borderless: true,
        ignores_mouse_events: false,
    }
}

#[cfg(target_os = "macos")]
pub fn build_overlay_panel(
    placement: PanelPlacement,
    width: u32,
    height: u32,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSPanel>, String> {
    use objc2::MainThreadOnly;
    use objc2_app_kit::{
        NSBackingStoreType, NSColor, NSPanel, NSScreen, NSWindowCollectionBehavior,
        NSWindowStyleMask,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "panel creation must run on the main thread".to_string())?;
    let screen = NSScreen::mainScreen(marker)
        .ok_or_else(|| "no main screen available for dock panel".to_string())?;
    let rect = NSRect::new(
        NSPoint::new(placement.x as f64, placement.y as f64),
        NSSize::new(width as f64, height as f64),
    );
    let style_mask = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer_screen(
        NSPanel::alloc(marker),
        rect,
        style_mask,
        NSBackingStoreType::Buffered,
        false,
        Some(&screen),
    );
    let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::Stationary
        | NSWindowCollectionBehavior::FullScreenAuxiliary;

    panel.setFloatingPanel(true);
    panel.setBecomesKeyOnlyIfNeeded(true);
    panel.setWorksWhenModal(true);
    panel.setOpaque(false);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
    panel.setHasShadow(false);
    panel.setHidesOnDeactivate(false);
    panel.setIgnoresMouseEvents(false);
    panel.setLevel(panel_configuration().level);
    panel.setCollectionBehavior(behavior);
    panel.orderFrontRegardless();

    Ok(panel)
}

#[cfg(not(target_os = "macos"))]
pub fn build_overlay_panel(
    _placement: PanelPlacement,
    _width: u32,
    _height: u32,
) -> Result<(), String> {
    Err("dock panel is only available on macOS".to_string())
}
