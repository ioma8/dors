#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::sel;

use crate::native_app::dock_item_view::{ITEM_WIDTH, build_item_button};
use crate::native_app::view_model::NativeDockItemModel;

const ITEM_SPACING: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockGlassStyle {
    pub horizontal_padding: f64,
    pub vertical_origin_y: f64,
    pub height: f64,
    pub corner_radius: f64,
    pub container_spacing: f64,
    pub tint_alpha: f64,
}

pub fn dock_glass_style() -> DockGlassStyle {
    DockGlassStyle {
        horizontal_padding: 26.0,
        vertical_origin_y: 40.0,
        height: 56.0,
        corner_radius: 28.0,
        container_spacing: 0.0,
        tint_alpha: 0.22,
    }
}

pub fn content_start_x(content_width: f64, glass_width: f64) -> f64 {
    ((glass_width - content_width) / 2.0).max(0.0)
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
        NSColor, NSGlassEffectContainerView, NSGlassEffectView, NSGlassEffectViewStyle, NSView,
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
    let glass_style = dock_glass_style();
    let glass_width = (content_width + glass_style.horizontal_padding * 2.0).max(180.0);
    let glass_origin_x = ((panel_width as f64 - glass_width) / 2.0).max(0.0);
    let start_x = content_start_x(content_width, glass_width);
    let glass_frame = NSRect::new(
        NSPoint::new(glass_origin_x, glass_style.vertical_origin_y),
        NSSize::new(glass_width, glass_style.height),
    );
    let container_view = NSGlassEffectContainerView::initWithFrame(
        NSGlassEffectContainerView::alloc(marker),
        glass_frame,
    );
    container_view.setSpacing(glass_style.container_spacing);
    let container_content = NSView::initWithFrame(
        NSView::alloc(marker),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(glass_width, glass_style.height),
        ),
    );
    container_view.setContentView(Some(&container_content));
    root_view.addSubview(&container_view);

    let glass_view = NSGlassEffectView::initWithFrame(NSGlassEffectView::alloc(marker), glass_frame);
    glass_view.setStyle(NSGlassEffectViewStyle::Regular);
    glass_view.setCornerRadius(glass_style.corner_radius);
    glass_view.setTintColor(Some(&NSColor::colorWithWhite_alpha(0.16, glass_style.tint_alpha)));
    root_view.addSubview(&glass_view);

    let glass_content = NSView::initWithFrame(
        NSView::alloc(marker),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(glass_width, glass_style.height),
        ),
    );
    glass_view.setContentView(Some(&glass_content));

    for (index, model) in models.iter().enumerate() {
        let origin_x = start_x + index as f64 * (ITEM_WIDTH + ITEM_SPACING);
        let item_view = build_item_button(
            model,
            origin_x,
            index,
            Some(target),
            Some(sel!(activateDockItem:)),
        )?;
        glass_content.addSubview(&item_view);
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
    (dock_glass_style().vertical_origin_y + dock_glass_style().height).ceil() as u32
}
