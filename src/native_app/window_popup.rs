#[cfg(target_os = "macos")]
use std::cell::RefCell;

use crate::native_app::window_menu::HoveredWindow;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupStyle {
    pub width: f64,
    pub horizontal_padding: f64,
    pub vertical_padding: f64,
    pub row_height: f64,
    pub row_spacing: f64,
    pub corner_radius: f64,
}

pub fn popup_style() -> PopupStyle {
    PopupStyle {
        width: 324.0,
        horizontal_padding: 14.0,
        vertical_padding: 14.0,
        row_height: 34.0,
        row_spacing: 6.0,
        corner_radius: 20.0,
    }
}

pub const WINDOW_POPUP_ITEM_HEIGHT: f64 = 34.0;
pub const WINDOW_POPUP_WIDTH: f64 = 324.0;
pub const WINDOW_POPUP_HORIZONTAL_PADDING: f64 = 14.0;
pub const WINDOW_POPUP_VERTICAL_PADDING: f64 = 14.0;
pub const WINDOW_POPUP_ROW_SPACING: f64 = 6.0;

pub fn popup_height(item_count: usize) -> f64 {
    let rows_height = WINDOW_POPUP_ITEM_HEIGHT * item_count as f64;
    let row_spacing = WINDOW_POPUP_ROW_SPACING * item_count.saturating_sub(1) as f64;
    WINDOW_POPUP_VERTICAL_PADDING * 2.0 + rows_height + row_spacing
}

#[cfg(target_os = "macos")]
use objc2::ClassType;
#[cfg(target_os = "macos")]
use objc2::Message;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(objc2_app_kit::NSView))]
    #[name = "DorsPopupTrackingView"]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Option<objc2::rc::Retained<AnyObject>>>]
    pub struct PopupTrackingView;

    impl PopupTrackingView {
        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, _event: &objc2_app_kit::NSEvent) {
            if let Some(target) = self.ivars().borrow().as_ref() {
                let _: () = unsafe { msg_send![&**target, hoverEnteredPopup: self] };
            }
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &objc2_app_kit::NSEvent) {
            if let Some(target) = self.ivars().borrow().as_ref() {
                let _: () = unsafe { msg_send![&**target, hoverExitedPopup: self] };
            }
        }
    }
);

#[cfg(target_os = "macos")]
pub fn popup_behavior() -> objc2_app_kit::NSPopoverBehavior {
    objc2_app_kit::NSPopoverBehavior::ApplicationDefined
}

#[cfg(target_os = "macos")]
pub fn build_window_popover(
    windows: &[HoveredWindow],
    target: &objc2::runtime::AnyObject,
    action: objc2::runtime::Sel,
    hover_target: &objc2::runtime::AnyObject,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSPopover>, String> {
    use objc2::{AnyThread, MainThreadOnly};
    use objc2_app_kit::{
        NSBox, NSBoxType, NSButton, NSButtonType, NSColor, NSFont, NSPopover, NSTextAlignment,
        NSTextField, NSTrackingArea, NSTrackingAreaOptions, NSView, NSViewController,
        NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
        NSVisualEffectView,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "popover creation must run on the main thread".to_string())?;
    let content_height = popup_height(windows.len());
    let style = popup_style();
    let content_view: objc2::rc::Retained<PopupTrackingView> = unsafe {
        msg_send![
            super(
                PopupTrackingView::alloc(marker).set_ivars(RefCell::new(Some(hover_target.retain())))
            ),
            initWithFrame: NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(WINDOW_POPUP_WIDTH, content_height),
            )
        ]
    };
    let tracking_view: &NSView = content_view.as_super();
    let tracking_area = unsafe {
        NSTrackingArea::initWithRect_options_owner_userInfo(
            NSTrackingArea::alloc(),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WINDOW_POPUP_WIDTH, content_height)),
            NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::EnabledDuringMouseDrag,
            Some(tracking_view),
            None,
        )
    };
    tracking_view.addTrackingArea(&tracking_area);
    let content_view: objc2::rc::Retained<NSView> = unsafe {
        Retained::cast_unchecked(content_view)
    };

    let glass = NSVisualEffectView::initWithFrame(
        NSVisualEffectView::alloc(marker),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(WINDOW_POPUP_WIDTH, content_height),
        ),
    );
    glass.setMaterial(NSVisualEffectMaterial::HUDWindow);
    glass.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    glass.setState(NSVisualEffectState::Active);
    content_view.addSubview(&glass);

    let frame_box = NSBox::initWithFrame(
        NSBox::alloc(marker),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(WINDOW_POPUP_WIDTH, content_height),
        ),
    );
    frame_box.setBoxType(NSBoxType::Custom);
    frame_box.setTransparent(false);
    frame_box.setCornerRadius(style.corner_radius);
    frame_box.setBorderWidth(1.0);
    frame_box.setBorderColor(&NSColor::colorWithWhite_alpha(1.0, 0.12));
    frame_box.setFillColor(&NSColor::colorWithWhite_alpha(0.06, 0.38));
    content_view.addSubview(&frame_box);

    for window in windows {
        let y = content_height
            - WINDOW_POPUP_VERTICAL_PADDING
            - WINDOW_POPUP_ITEM_HEIGHT * (window.index as f64 + 1.0)
            - WINDOW_POPUP_ROW_SPACING * window.index as f64;
        let row_width = WINDOW_POPUP_WIDTH - WINDOW_POPUP_HORIZONTAL_PADDING * 2.0;

        let row = NSBox::initWithFrame(
            NSBox::alloc(marker),
            NSRect::new(
                NSPoint::new(WINDOW_POPUP_HORIZONTAL_PADDING, y),
                NSSize::new(row_width, WINDOW_POPUP_ITEM_HEIGHT),
            ),
        );
        row.setBoxType(NSBoxType::Custom);
        row.setTransparent(false);
        row.setCornerRadius(11.0);
        row.setBorderWidth(1.0);
        row.setBorderColor(&NSColor::colorWithWhite_alpha(1.0, 0.08));
        row.setFillColor(&NSColor::colorWithWhite_alpha(1.0, 0.10));
        content_view.addSubview(&row);

        let label = NSTextField::labelWithString(&NSString::from_str(&window.title), marker);
        label.setFrame(NSRect::new(
            NSPoint::new(WINDOW_POPUP_HORIZONTAL_PADDING + 14.0, y + 6.0),
            NSSize::new(row_width - 28.0, WINDOW_POPUP_ITEM_HEIGHT - 12.0),
        ));
        label.setAlignment(NSTextAlignment::Left);
        label.setTextColor(Some(&NSColor::colorWithWhite_alpha(1.0, 0.92)));
        label.setFont(Some(&NSFont::systemFontOfSize(16.0)));
        content_view.addSubview(&label);

        let button = NSButton::initWithFrame(
            NSButton::alloc(marker),
            NSRect::new(
                NSPoint::new(WINDOW_POPUP_HORIZONTAL_PADDING, y),
                NSSize::new(
                    row_width,
                    WINDOW_POPUP_ITEM_HEIGHT,
                ),
            ),
        );
        button.setButtonType(NSButtonType::MomentaryChange);
        button.setBordered(false);
        button.setTransparent(true);
        button.setTitle(&NSString::from_str(""));
        button.setTag(window.index as isize);
        unsafe {
            button.setTarget(Some(target));
            button.setAction(Some(action));
        }
        content_view.addSubview(&button);
    }

    let controller = NSViewController::new(marker);
    controller.setView(&content_view);
    controller.setPreferredContentSize(NSSize::new(WINDOW_POPUP_WIDTH, content_height));

    let popover = NSPopover::new(marker);
    popover.setBehavior(popup_behavior());
    popover.setAnimates(true);
    popover.setContentSize(NSSize::new(WINDOW_POPUP_WIDTH, content_height));
    popover.setContentViewController(Some(&controller));

    Ok(popover)
}
