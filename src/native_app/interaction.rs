use std::cell::RefCell;
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::Message;
#[cfg(target_os = "macos")]
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSControl, NSPanel, NSPopover};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSObject, NSRectEdge, NSTimer};

use crate::native_app::clamp_scheduler::ClampScheduler;
use crate::native_app::dock_view::{build_dock_view, required_height};
use crate::native_app::refresh::load_startup_models;
use crate::native_app::view_model::NativeDockItemModel;
use crate::native_app::window_clamper;
use crate::native_app::window_menu::{
    HoverDelayState, HoveredWindow, activate_specific_window, hover_menu_dismiss_delay_millis,
    read_windows_for_app, should_show_window_menu,
};
use crate::native_app::window_popup::build_window_popover;
use crate::services::launcher::{self, LaunchRequest};

pub fn launch_request_from_model(model: &NativeDockItemModel) -> LaunchRequest {
    LaunchRequest {
        bundle_id: model.bundle_id.clone(),
        path: model.path.clone(),
        is_running: model.is_running,
    }
}

pub fn should_present_hover_menu(windows: &[HoveredWindow]) -> bool {
    should_show_window_menu(windows.len())
}

pub fn should_schedule_hover_menu_dismiss(popover_open: bool, hovering_popover: bool) -> bool {
    popover_open && !hovering_popover
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct HoverMenuContext {
    bundle_id: Option<String>,
    fallback_process_name: String,
    windows: Vec<HoveredWindow>,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct DockControllerState {
    panel: Retained<NSPanel>,
    models: Vec<NativeDockItemModel>,
    panel_width: u32,
    panel_height: u32,
    clamp_scheduler: ClampScheduler,
    hover_delay_state: HoverDelayState,
    hover_deadline: Option<Instant>,
    hover_dismiss_deadline: Option<Instant>,
    hover_anchor: Option<Retained<NSControl>>,
    hover_menu_context: Option<HoverMenuContext>,
    hover_popover: Option<Retained<NSPopover>>,
    hovering_popover: bool,
}

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "DorsDockController"]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<DockControllerState>]
    pub struct DockController;

    impl DockController {
        #[unsafe(method(activateDockItem:))]
        fn activate_dock_item(&self, sender: Option<&AnyObject>) {
            let Some(control) = sender.and_then(|value| value.downcast_ref::<NSControl>()) else {
                return;
            };
            let Ok(index) = usize::try_from(control.tag()) else {
                return;
            };
            let model = {
                let state = self.ivars().borrow();
                state.models.get(index).cloned()
            };
            let Some(model) = model else {
                return;
            };
            let request = launch_request_from_model(&model);
            let _ = launcher::launch_or_activate(
                &request,
                launcher::activate_app,
                launcher::launch_app,
            );
            self.refresh_from_system();
        }

        #[unsafe(method(refreshDock:))]
        fn refresh_dock(&self, _sender: Option<&AnyObject>) {
            self.refresh_from_system();
        }

        #[unsafe(method(hoverEnteredDockItem:))]
        fn hover_entered_dock_item(&self, sender: Option<&AnyObject>) {
            let Some(control) = sender.and_then(|value| value.downcast_ref::<NSControl>()) else {
                return;
            };
            let Ok(index) = usize::try_from(control.tag()) else {
                return;
            };
            let delay = {
                let mut state = self.ivars().borrow_mut();
                let _ = state.hover_delay_state.schedule_for_item(index);
                state.hover_deadline = Some(
                    Instant::now()
                        + Duration::from_millis(state.hover_delay_state.delay_millis()),
                );
                state.hover_dismiss_deadline = None;
                state.hovering_popover = false;
                state.hover_anchor = Some(control.retain());
                state.hover_menu_context = None;
                if let Some(popover) = state.hover_popover.take() {
                    popover.close();
                }
                state.hover_delay_state.delay_millis()
            };
            self.schedule_hover_timer(delay);
        }

        #[unsafe(method(hoverExitedDockItem:))]
        fn hover_exited_dock_item(&self, sender: Option<&AnyObject>) {
            let Some(control) = sender.and_then(|value| value.downcast_ref::<NSControl>()) else {
                return;
            };
            let Ok(index) = usize::try_from(control.tag()) else {
                return;
            };
            let mut state = self.ivars().borrow_mut();
            if state.hover_delay_state.current_item() == Some(index) {
                if should_schedule_hover_menu_dismiss(
                    state.hover_popover.is_some(),
                    state.hovering_popover,
                ) {
                    state.hover_dismiss_deadline = Some(
                        Instant::now()
                            + Duration::from_millis(hover_menu_dismiss_delay_millis()),
                    );
                    drop(state);
                    self.schedule_hover_dismiss_timer(hover_menu_dismiss_delay_millis());
                    return;
                }
                state.hover_delay_state.cancel();
                state.hover_deadline = None;
                state.hover_dismiss_deadline = None;
                state.hover_anchor = None;
                state.hover_menu_context = None;
            }
        }

        #[unsafe(method(hoverEnteredPopup:))]
        fn hover_entered_popup(&self, _sender: Option<&AnyObject>) {
            let mut state = self.ivars().borrow_mut();
            state.hovering_popover = true;
            state.hover_dismiss_deadline = None;
        }

        #[unsafe(method(hoverExitedPopup:))]
        fn hover_exited_popup(&self, _sender: Option<&AnyObject>) {
            let should_schedule = {
                let mut state = self.ivars().borrow_mut();
                state.hovering_popover = false;
                if !state.hover_popover.is_some() {
                    false
                } else {
                    state.hover_dismiss_deadline = Some(
                        Instant::now()
                            + Duration::from_millis(hover_menu_dismiss_delay_millis()),
                    );
                    true
                }
            };
            if should_schedule {
                self.schedule_hover_dismiss_timer(hover_menu_dismiss_delay_millis());
            }
        }

        #[unsafe(method(openHoverMenuIfDue:))]
        fn open_hover_menu_if_due_timer(&self, _sender: Option<&AnyObject>) {
            self.open_hover_menu_if_due();
        }

        #[unsafe(method(dismissHoverMenuIfDue:))]
        fn dismiss_hover_menu_if_due_timer(&self, _sender: Option<&AnyObject>) {
            self.dismiss_hover_menu_if_due();
        }

        #[unsafe(method(activateHoverWindow:))]
        fn activate_hover_window(&self, sender: Option<&AnyObject>) {
            let Some(control) = sender.and_then(|value| value.downcast_ref::<NSControl>()) else {
                return;
            };
            let Ok(index) = usize::try_from(control.tag()) else {
                return;
            };
            let context = {
                let state = self.ivars().borrow();
                state.hover_menu_context.clone()
            };
            let Some(context) = context else {
                return;
            };
            let Some(window) = context.windows.get(index) else {
                return;
            };
            let _ = activate_specific_window(
                context.bundle_id.as_deref(),
                &context.fallback_process_name,
                &window.title,
            );
            let mut state = self.ivars().borrow_mut();
            if let Some(popover) = state.hover_popover.take() {
                popover.close();
            }
            state.hover_menu_context = None;
            state.hover_dismiss_deadline = None;
        }
    }
);

#[cfg(target_os = "macos")]
impl DockController {
    pub fn new(
        marker: MainThreadMarker,
        panel: Retained<NSPanel>,
        panel_width: u32,
        panel_height: u32,
        models: Vec<NativeDockItemModel>,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(RefCell::new(DockControllerState {
            panel,
            models,
            panel_width,
            panel_height,
            clamp_scheduler: ClampScheduler::new(),
            hover_delay_state: HoverDelayState::new(),
            hover_deadline: None,
            hover_dismiss_deadline: None,
            hover_anchor: None,
            hover_menu_context: None,
            hover_popover: None,
            hovering_popover: false,
        }));
        unsafe { msg_send![super(this), init] }
    }

    pub fn render_current_models(&self) -> Result<(), String> {
        let state = self.ivars().borrow();
        let content_view =
            build_dock_view(&state.models, state.panel_width, state.panel_height, self)?;
        state.panel.setContentView(Some(&content_view));
        Ok(())
    }

    pub fn install_refresh_timer(&self) -> Retained<NSTimer> {
        unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                0.75,
                self,
                sel!(refreshDock:),
                None,
                true,
            )
        }
    }

    fn refresh_from_system(&self) {
        let models = match load_startup_models() {
            Ok(models) => models,
            Err(error) => {
                eprintln!("[dors-debug] native refresh failed: {error}");
                return;
            }
        };
        let clamp_scheduler = {
            let mut state = self.ivars().borrow_mut();
            let clamp_scheduler = state.clamp_scheduler.clone();
            if state.models == models {
                drop(state);
                let _ = clamp_scheduler.try_schedule(|| {
                    window_clamper::clamp_main_screen_windows(required_height() as i32)
                });
                return;
            }
            state.models = models;
            clamp_scheduler
        };
        let _ = clamp_scheduler.try_schedule(|| {
            window_clamper::clamp_main_screen_windows(required_height() as i32)
        });
        if let Err(error) = self.render_current_models() {
            eprintln!("[dors-debug] native render failed: {error}");
        }
    }

    fn schedule_hover_timer(&self, delay_millis: u64) {
        unsafe {
            let _ = NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                delay_millis as f64 / 1000.0,
                self,
                sel!(openHoverMenuIfDue:),
                None,
                false,
            );
        }
    }

    fn schedule_hover_dismiss_timer(&self, delay_millis: u64) {
        unsafe {
            let _ = NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                delay_millis as f64 / 1000.0,
                self,
                sel!(dismissHoverMenuIfDue:),
                None,
                false,
            );
        }
    }

    fn open_hover_menu_if_due(&self) {
        let (index, anchor, model) = {
            let state = self.ivars().borrow();
            let Some(deadline) = state.hover_deadline else {
                return;
            };
            if Instant::now() < deadline {
                return;
            }
            let Some(index) = state.hover_delay_state.current_item() else {
                return;
            };
            let Some(anchor) = state.hover_anchor.clone() else {
                return;
            };
            let Some(model) = state.models.get(index).cloned() else {
                return;
            };
            (index, anchor, model)
        };

        let windows =
            match read_windows_for_app(model.bundle_id.as_deref(), &model.display_name) {
                Ok(windows) => windows,
                Err(_) => return,
            };
        if !should_present_hover_menu(&windows) {
            return;
        }

        let popover = match build_window_popover(
            &windows,
            self,
            sel!(activateHoverWindow:),
            self,
        ) {
            Ok(popover) => popover,
            Err(_) => return,
        };
        {
            let mut state = self.ivars().borrow_mut();
            if state.hover_delay_state.current_item() != Some(index) {
                return;
            }
            state.hover_dismiss_deadline = None;
            state.hover_menu_context = Some(HoverMenuContext {
                bundle_id: model.bundle_id.clone(),
                fallback_process_name: model.display_name.clone(),
                windows: windows.clone(),
            });
            state.hovering_popover = false;
            if let Some(existing) = state.hover_popover.take() {
                existing.close();
            }
            state.hover_popover = Some(popover.retain());
        }
        let (menu_view, positioning_rect) = {
            let frame = anchor.frame();
            let Some(superview) = (unsafe { anchor.superview() }) else {
                return;
            };
            (superview, frame)
        };
        popover.showRelativeToRect_ofView_preferredEdge(
            positioning_rect,
            &menu_view,
            NSRectEdge::MaxY,
        );
    }

    fn dismiss_hover_menu_if_due(&self) {
        let popover = {
            let mut state = self.ivars().borrow_mut();
            let Some(deadline) = state.hover_dismiss_deadline else {
                return;
            };
            if Instant::now() < deadline {
                return;
            }
            state.hover_delay_state.cancel();
            state.hover_deadline = None;
            state.hover_dismiss_deadline = None;
            state.hover_anchor = None;
            state.hover_menu_context = None;
            state.hovering_popover = false;
            state.hover_popover.take()
        };
        if let Some(popover) = popover {
            popover.close();
        }
    }
}
