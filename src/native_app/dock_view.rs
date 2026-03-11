#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::sel;

use crate::native_app::dock_item_view::{ITEM_HEIGHT, ITEM_WIDTH, build_item_button};
use crate::native_app::view_model::NativeDockItemModel;

const ITEM_SPACING: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockGlassStyle {
    pub horizontal_padding: f64,
    pub vertical_origin_y: f64,
    pub height: f64,
}

pub fn dock_glass_style() -> DockGlassStyle {
    DockGlassStyle {
        horizontal_padding: 24.0,
        vertical_origin_y: 8.0,
        height: 104.0,
    }
}

#[cfg(target_os = "macos")]
pub fn build_dock_view(
    models: &[NativeDockItemModel],
    panel_width: u32,
    panel_height: u32,
    target: &AnyObject,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSView>, String> {
    use objc2::MainThreadOnly;
    use objc2_app_kit::{
        NSBox, NSBoxType, NSColor, NSTitlePosition, NSView, NSVisualEffectBlendingMode,
        NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "dock view creation must run on the main thread".to_string())?;
    let root_view = NSView::initWithFrame(
        NSView::alloc(marker),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(panel_width as f64, panel_height as f64),
        ),
    );
    let content_width = if models.is_empty() {
        0.0
    } else {
        models.len() as f64 * ITEM_WIDTH + (models.len().saturating_sub(1) as f64 * ITEM_SPACING)
    };
    let start_x = ((panel_width as f64 - content_width) / 2.0).max(0.0);
    let glass_style = dock_glass_style();
    let glass_width = (content_width + glass_style.horizontal_padding * 2.0).max(180.0);
    let glass_origin_x = ((panel_width as f64 - glass_width) / 2.0).max(0.0);
    let glass_frame = NSRect::new(
        NSPoint::new(glass_origin_x, glass_style.vertical_origin_y),
        NSSize::new(glass_width, glass_style.height),
    );
    let glass_box = NSBox::initWithFrame(NSBox::alloc(marker), glass_frame);
    glass_box.setBoxType(NSBoxType::Custom);
    glass_box.setTitlePosition(NSTitlePosition::NoTitle);
    glass_box.setTransparent(false);
    glass_box.setCornerRadius(28.0);
    glass_box.setBorderWidth(1.0);
    glass_box.setBorderColor(&NSColor::colorWithWhite_alpha(1.0, 0.18));
    glass_box.setFillColor(&NSColor::colorWithWhite_alpha(1.0, 0.08));
    root_view.addSubview(&glass_box);
    let glass_view = NSVisualEffectView::initWithFrame(
        NSVisualEffectView::alloc(marker),
        glass_frame,
    );
    glass_view.setMaterial(NSVisualEffectMaterial::HUDWindow);
    glass_view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    glass_view.setState(NSVisualEffectState::Active);
    glass_view.setEmphasized(true);
    root_view.addSubview(&glass_view);

    for (index, model) in models.iter().enumerate() {
        let origin_x = start_x + index as f64 * (ITEM_WIDTH + ITEM_SPACING);
        let item_view = build_item_button(
            model,
            origin_x,
            index,
            Some(target),
            Some(sel!(activateDockItem:)),
        )?;
        root_view.addSubview(&item_view);
    }

    Ok(root_view)
}

#[cfg(not(target_os = "macos"))]
pub fn build_dock_view(
    _models: &[NativeDockItemModel],
    _panel_width: u32,
    _panel_height: u32,
    _target: &(),
) -> Result<(), String> {
    Err("native dock view is only available on macOS".to_string())
}

pub fn required_height() -> u32 {
    ITEM_HEIGHT as u32 + 24
}
