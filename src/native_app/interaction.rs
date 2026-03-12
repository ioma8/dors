use std::cell::RefCell;

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSControl, NSPanel};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSObject, NSTimer};

use crate::native_app::clamp_scheduler::ClampScheduler;
use crate::native_app::dock_view::{build_dock_view, required_height};
use crate::native_app::refresh::load_startup_models;
use crate::native_app::view_model::NativeDockItemModel;
use crate::native_app::window_clamper;
use crate::services::launcher::{self, LaunchRequest};

pub fn launch_request_from_model(model: &NativeDockItemModel) -> LaunchRequest {
    LaunchRequest {
        bundle_id: model.bundle_id.clone(),
        path: model.path.clone(),
        is_running: model.is_running,
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct DockControllerState {
    panel: Retained<NSPanel>,
    models: Vec<NativeDockItemModel>,
    panel_width: u32,
    panel_height: u32,
    clamp_scheduler: ClampScheduler,
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
}
