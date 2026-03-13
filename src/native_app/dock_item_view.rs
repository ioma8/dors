#[cfg(target_os = "macos")]
use base64::Engine;

#[cfg(target_os = "macos")]
use objc2::AnyThread;
#[cfg(target_os = "macos")]
use objc2::msg_send;
#[cfg(target_os = "macos")]
use objc2::runtime::{AnyObject, Sel};
#[cfg(target_os = "macos")]
use objc2::{MainThreadOnly, define_class};

use crate::native_app::view_model::NativeDockItemModel;

pub const ITEM_WIDTH: f64 = 72.0;
pub const ITEM_HEIGHT: f64 = 56.0;

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(objc2_app_kit::NSButton))]
    #[name = "DorsDockItemButton"]
    #[thread_kind = MainThreadOnly]
    pub struct DockItemButton;

    impl DockItemButton {
        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, _event: &objc2_app_kit::NSEvent) {
            apply_hover_shadow(self, true);
            if let Some(target) = self.target() {
                let _: () = unsafe { msg_send![&*target, hoverEnteredDockItem: self] };
            }
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &objc2_app_kit::NSEvent) {
            apply_hover_shadow(self, false);
            if let Some(target) = self.target() {
                let _: () = unsafe { msg_send![&*target, hoverExitedDockItem: self] };
            }
        }

        #[unsafe(method(rightMouseDown:))]
        fn right_mouse_down(&self, event: &objc2_app_kit::NSEvent) {
            if let Some(target) = self.target() {
                let _: () = unsafe {
                    msg_send![&*target, showContextMenuForDockItem: self, event: event]
                };
            }
        }
    }
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLayout {
    pub button_origin_x: f64,
    pub button_origin_y: f64,
    pub button_width: f64,
    pub button_height: f64,
    pub icon_origin_x: f64,
    pub icon_origin_y: f64,
    pub icon_size: f64,
    pub indicator_origin_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverStyle {
    pub active_ring: bool,
    pub hover_shadow: bool,
}

pub fn item_layout() -> ItemLayout {
    ItemLayout {
        button_origin_x: 6.0,
        button_origin_y: 2.0,
        button_width: 60.0,
        button_height: 48.0,
        icon_origin_x: 9.0,
        icon_origin_y: 3.0,
        icon_size: 42.0,
        indicator_origin_y: 38.0,
    }
}

pub fn hover_style(is_active: bool) -> HoverStyle {
    HoverStyle {
        active_ring: is_active,
        hover_shadow: true,
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
    use objc2_app_kit::{
        NSBox, NSBoxType, NSButtonType, NSColor, NSImageScaling, NSImageView, NSTextAlignment,
        NSTextField, NSTitlePosition, NSTrackingArea, NSTrackingAreaOptions,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "item view creation must run on the main thread".to_string())?;
    let layout = item_layout();
    let hover = hover_style(model.is_active);
    let frame = NSRect::new(
        NSPoint::new(origin_x + layout.button_origin_x, layout.button_origin_y),
        NSSize::new(layout.button_width, layout.button_height),
    );
    let item_view: objc2::rc::Retained<DockItemButton> =
        unsafe { msg_send![DockItemButton::alloc(marker), initWithFrame: frame] };
    item_view.setButtonType(NSButtonType::MomentaryChange);
    item_view.setBordered(false);
    item_view.setTransparent(true);
    item_view.setShowsBorderOnlyWhileMouseInside(false);
    item_view.setTitle(&NSString::from_str(""));
    item_view.setTag(index as isize);
    unsafe {
        item_view.setTarget(target);
        item_view.setAction(action);
    }
    let tracking = unsafe {
        NSTrackingArea::initWithRect_options_owner_userInfo(
            NSTrackingArea::alloc(),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(layout.button_width, layout.button_height)),
            NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::EnabledDuringMouseDrag,
            Some(item_view.as_ref()),
            None,
        )
    };
    item_view.addTrackingArea(&tracking);

    if hover.active_ring {
        let ring = NSBox::initWithFrame(
            NSBox::alloc(marker),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(layout.button_width, layout.button_height),
            ),
        );
        ring.setBoxType(NSBoxType::Custom);
        ring.setTitlePosition(NSTitlePosition::NoTitle);
        ring.setTransparent(false);
        ring.setCornerRadius(16.0);
        ring.setBorderWidth(1.5);
        ring.setBorderColor(&NSColor::colorWithWhite_alpha(1.0, 0.42));
        ring.setFillColor(&NSColor::colorWithWhite_alpha(1.0, 0.08));
        item_view.addSubview(&ring);
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
        if !model.is_running {
            icon_view.setAlphaValue(0.42);
            let mute_overlay = NSBox::initWithFrame(
                NSBox::alloc(marker),
                NSRect::new(
                    NSPoint::new(layout.icon_origin_x, layout.icon_origin_y),
                    NSSize::new(layout.icon_size, layout.icon_size),
                ),
            );
            mute_overlay.setBoxType(NSBoxType::Custom);
            mute_overlay.setTitlePosition(NSTitlePosition::NoTitle);
            mute_overlay.setTransparent(false);
            mute_overlay.setCornerRadius(12.0);
            mute_overlay.setBorderWidth(0.0);
            mute_overlay.setFillColor(&NSColor::colorWithWhite_alpha(0.86, 0.24));
            item_view.addSubview(&mute_overlay);
        }
        item_view.addSubview(&icon_view);
    } else {
        let fallback = NSString::from_str(&placeholder_text(model));
        let placeholder = NSTextField::labelWithString(&fallback, marker);
        placeholder.setFrame(NSRect::new(
            NSPoint::new(layout.icon_origin_x, layout.icon_origin_y + 2.0),
            NSSize::new(layout.icon_size, layout.icon_size - 8.0),
        ));
        placeholder.setAlignment(NSTextAlignment::Center);
        if !model.is_running {
            placeholder.setAlphaValue(0.45);
        }
        item_view.addSubview(&placeholder);
    }

    Ok(item_view.into_super())
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

#[cfg(target_os = "macos")]
fn apply_hover_shadow(button: &DockItemButton, hovered: bool) {
    use objc2_app_kit::{NSColor, NSShadow};
    use objc2_foundation::NSSize;

    if hovered {
        let shadow = NSShadow::new();
        shadow.setShadowOffset(NSSize::new(0.0, -2.0));
        shadow.setShadowBlurRadius(14.0);
        shadow.setShadowColor(Some(&NSColor::colorWithWhite_alpha(0.0, 0.35)));
        button.setShadow(Some(&shadow));
    } else {
        button.setShadow(None);
    }
}
