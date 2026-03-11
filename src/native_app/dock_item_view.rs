#[cfg(target_os = "macos")]
use base64::Engine;

#[cfg(target_os = "macos")]
use objc2::runtime::{AnyObject, Sel};

use crate::native_app::view_model::NativeDockItemModel;

pub const ITEM_WIDTH: f64 = 72.0;
pub const ITEM_HEIGHT: f64 = 92.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLayout {
    pub button_origin_y: f64,
    pub icon_origin_x: f64,
    pub icon_origin_y: f64,
    pub icon_size: f64,
    pub indicator_origin_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverStyle {
    pub bordered: bool,
    pub transparent: bool,
    pub hover_only_border: bool,
}

pub fn item_layout() -> ItemLayout {
    ItemLayout {
        button_origin_y: 18.0,
        icon_origin_x: 8.0,
        icon_origin_y: 18.0,
        icon_size: 56.0,
        indicator_origin_y: 2.0,
    }
}

pub fn hover_style() -> HoverStyle {
    HoverStyle {
        bordered: true,
        transparent: false,
        hover_only_border: true,
    }
}

#[cfg(target_os = "macos")]
pub fn build_item_view(
    model: &NativeDockItemModel,
    origin_x: f64,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSButton>, String> {
    build_item_button(model, origin_x, 0, None, None)
}

#[cfg(target_os = "macos")]
pub fn build_item_button(
    model: &NativeDockItemModel,
    origin_x: f64,
    index: usize,
    target: Option<&AnyObject>,
    action: Option<Sel>,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSButton>, String> {
    use objc2::MainThreadOnly;
    use objc2_app_kit::{
        NSBezelStyle, NSButton, NSButtonType, NSColor, NSImageScaling, NSImageView,
        NSTextAlignment, NSTextField,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "item view creation must run on the main thread".to_string())?;
    let layout = item_layout();
    let hover = hover_style();
    let item_view = NSButton::initWithFrame(
        NSButton::alloc(marker),
        NSRect::new(
            NSPoint::new(origin_x, layout.button_origin_y),
            NSSize::new(ITEM_WIDTH, ITEM_HEIGHT),
        ),
    );
    item_view.setButtonType(NSButtonType::MomentaryChange);
    item_view.setBezelStyle(NSBezelStyle::Glass);
    item_view.setBordered(hover.bordered);
    item_view.setTransparent(hover.transparent);
    item_view.setShowsBorderOnlyWhileMouseInside(hover.hover_only_border);
    item_view.setTitle(&NSString::from_str(""));
    item_view.setTag(index as isize);
    unsafe {
        item_view.setTarget(target);
        item_view.setAction(action);
    }

    if let Some(icon) = icon_from_model(model, marker) {
        let icon_view = NSImageView::initWithFrame(
            NSImageView::alloc(marker),
            NSRect::new(
                NSPoint::new(layout.icon_origin_x, layout.icon_origin_y),
                NSSize::new(layout.icon_size, layout.icon_size),
            ),
        );
        icon_view.setImage(Some(&icon));
        icon_view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
        item_view.addSubview(&icon_view);
    } else {
        let fallback = NSString::from_str(&placeholder_text(model));
        let placeholder = NSTextField::labelWithString(&fallback, marker);
        placeholder.setFrame(NSRect::new(
            NSPoint::new(layout.icon_origin_x, layout.icon_origin_y + 6.0),
            NSSize::new(layout.icon_size, layout.icon_size - 8.0),
        ));
        placeholder.setAlignment(NSTextAlignment::Center);
        item_view.addSubview(&placeholder);
    }

    if model.shows_indicator {
        let indicator_text = NSString::from_str("•");
        let indicator = NSTextField::labelWithString(&indicator_text, marker);
        indicator.setFrame(NSRect::new(
            NSPoint::new((ITEM_WIDTH - 12.0) / 2.0, layout.indicator_origin_y),
            NSSize::new(12.0, 12.0),
        ));
        if model.is_active {
            indicator.setTextColor(Some(&NSColor::whiteColor()));
        } else {
            indicator.setTextColor(Some(&NSColor::lightGrayColor()));
        }
        item_view.addSubview(&indicator);
    }

    Ok(item_view)
}

#[cfg(not(target_os = "macos"))]
pub fn build_item_view(_model: &NativeDockItemModel, _origin_x: f64) -> Result<(), String> {
    Err("native dock item view is only available on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn build_item_button(
    _model: &NativeDockItemModel,
    _origin_x: f64,
    _index: usize,
    _target: Option<&()>,
    _action: Option<()>,
) -> Result<(), String> {
    Err("native dock item view is only available on macOS".to_string())
}

fn placeholder_text(model: &NativeDockItemModel) -> String {
    model.display_name.chars().next().unwrap_or('?').to_string()
}

#[cfg(target_os = "macos")]
fn icon_from_model(
    model: &NativeDockItemModel,
    _marker: objc2_foundation::MainThreadMarker,
) -> Option<objc2::rc::Retained<objc2_app_kit::NSImage>> {
    use objc2::AnyThread;
    use objc2_app_kit::NSImage;
    use objc2_foundation::NSData;

    let (_, encoded) = model.icon_src.split_once(',')?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let data = NSData::with_bytes(&bytes);

    NSImage::initWithData(NSImage::alloc(), &data)
}
