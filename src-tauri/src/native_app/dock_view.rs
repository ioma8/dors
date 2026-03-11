use crate::native_app::dock_item_view::{build_item_view, ITEM_HEIGHT, ITEM_WIDTH};
use crate::native_app::view_model::NativeDockItemModel;

const ITEM_SPACING: f64 = 12.0;

#[cfg(target_os = "macos")]
pub fn build_dock_view(
    models: &[NativeDockItemModel],
    panel_width: u32,
    panel_height: u32,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSView>, String> {
    use objc2::MainThreadOnly;
    use objc2_app_kit::NSView;
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

    for (index, model) in models.iter().enumerate() {
        let origin_x = start_x + index as f64 * (ITEM_WIDTH + ITEM_SPACING);
        let item_view = build_item_view(model, origin_x)?;
        root_view.addSubview(&item_view);
    }

    Ok(root_view)
}

#[cfg(not(target_os = "macos"))]
pub fn build_dock_view(
    _models: &[NativeDockItemModel],
    _panel_width: u32,
    _panel_height: u32,
) -> Result<(), String> {
    Err("native dock view is only available on macOS".to_string())
}

pub fn required_height() -> u32 {
    ITEM_HEIGHT as u32 + 24
}
