#[cfg(target_os = "macos")]
use std::cell::RefCell;

use crate::native_app::window_menu::HoveredWindow;

pub const WINDOW_POPUP_ITEM_HEIGHT: f64 = 26.0;
pub const WINDOW_POPUP_WIDTH: f64 = 320.0;
pub const WINDOW_POPUP_HORIZONTAL_PADDING: f64 = 10.0;
pub const WINDOW_POPUP_VERTICAL_PADDING: f64 = 10.0;

pub fn popup_height(item_count: usize) -> f64 {
    WINDOW_POPUP_VERTICAL_PADDING * 2.0 + WINDOW_POPUP_ITEM_HEIGHT * item_count as f64
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
        NSButton, NSPopover, NSTrackingArea, NSTrackingAreaOptions, NSView, NSViewController,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "popover creation must run on the main thread".to_string())?;
    let content_height = popup_height(windows.len());
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

    for window in windows {
        let y = content_height
            - WINDOW_POPUP_VERTICAL_PADDING
            - WINDOW_POPUP_ITEM_HEIGHT * (window.index as f64 + 1.0);
        let button = NSButton::initWithFrame(
            NSButton::alloc(marker),
            NSRect::new(
                NSPoint::new(WINDOW_POPUP_HORIZONTAL_PADDING, y),
                NSSize::new(
                    WINDOW_POPUP_WIDTH - WINDOW_POPUP_HORIZONTAL_PADDING * 2.0,
                    WINDOW_POPUP_ITEM_HEIGHT,
                ),
            ),
        );
        button.setTitle(&NSString::from_str(&window.title));
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
