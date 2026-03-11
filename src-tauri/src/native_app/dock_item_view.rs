#[cfg(target_os = "macos")]
use base64::Engine;

#[cfg(target_os = "macos")]
use objc2::runtime::{AnyObject, Sel};

use crate::native_app::view_model::NativeDockItemModel;

pub const ITEM_WIDTH: f64 = 72.0;
pub const ITEM_HEIGHT: f64 = 92.0;

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
        NSButton, NSButtonType, NSCellImagePosition, NSColor, NSImageScaling,
        NSTextField,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "item view creation must run on the main thread".to_string())?;
    let item_view = NSButton::initWithFrame(
        NSButton::alloc(marker),
        NSRect::new(
            NSPoint::new(origin_x, 24.0),
            NSSize::new(ITEM_WIDTH, ITEM_HEIGHT),
        ),
    );
    item_view.setButtonType(NSButtonType::MomentaryChange);
    item_view.setBordered(true);
    item_view.setTransparent(true);
    item_view.setShowsBorderOnlyWhileMouseInside(true);
    item_view.setTag(index as isize);
    unsafe {
        item_view.setTarget(target);
        item_view.setAction(action);
    }

    if let Some(icon) = icon_from_model(model, marker) {
        item_view.setImage(Some(&icon));
        item_view.setImagePosition(NSCellImagePosition::ImageOnly);
        item_view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
        item_view.setImageHugsTitle(false);
    } else {
        let fallback = NSString::from_str(&placeholder_text(model));
        item_view.setTitle(&fallback);
    }

    if model.shows_indicator {
        let indicator_text = NSString::from_str("•");
        let indicator = NSTextField::labelWithString(&indicator_text, marker);
        indicator.setFrame(NSRect::new(
            NSPoint::new((ITEM_WIDTH - 12.0) / 2.0, 0.0),
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
pub fn build_item_view(
    _model: &NativeDockItemModel,
    _origin_x: f64,
) -> Result<(), String> {
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
