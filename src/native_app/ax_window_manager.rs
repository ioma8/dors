use std::collections::BTreeSet;

#[cfg(target_os = "macos")]
use core_foundation::base::{CFRelease, CFRetain, TCFType};
#[cfg(target_os = "macos")]
use core_foundation::runloop::{CFRunLoop, CFRunLoopSource, kCFRunLoopDefaultMode};
#[cfg(target_os = "macos")]
use core_foundation::string::{CFString, CFStringRef};
#[cfg(target_os = "macos")]
use std::ffi::c_void;

#[cfg(target_os = "macos")]
use crate::native_app::clamp_scheduler::ClampScheduler;
use crate::native_app::window_clamper::{
    ClampOperation, CustomZoomTracker, WindowCandidate, WindowSignal, WorkingArea,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowEventKind {
    FocusedWindowChanged,
    MainWindowChanged,
    Moved,
    Resized,
    Miniaturized,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObservedWindowId {
    pid: i32,
    window_index: usize,
}

impl ObservedWindowId {
    pub fn new(pid: i32, window_index: usize) -> Self {
        Self { pid, window_index }
    }

    pub fn pid(self) -> i32 {
        self.pid
    }

    pub fn window_index(self) -> usize {
        self.window_index
    }
}

pub fn normalize_notification_name(name: &str) -> Option<WindowEventKind> {
    match name {
        "AXFocusedWindowChanged" => Some(WindowEventKind::FocusedWindowChanged),
        "AXMainWindowChanged" => Some(WindowEventKind::MainWindowChanged),
        "AXMoved" | "AXWindowMoved" => Some(WindowEventKind::Moved),
        "AXResized" | "AXWindowResized" => Some(WindowEventKind::Resized),
        "AXWindowMiniaturized" => Some(WindowEventKind::Miniaturized),
        _ => None,
    }
}

pub fn observer_notification_names() -> [&'static str; 11] {
    [
        "AXCreated",
        "AXFocusedWindowChanged",
        "AXMainWindowChanged",
        "AXTitleChanged",
        "AXUIElementDestroyed",
        "AXWindowDeminiaturized",
        "AXWindowMiniaturized",
        "AXWindowMoved",
        "AXWindowResized",
        "AXMenuOpened",
        "AXMenuClosed",
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedWindowEvent {
    pub window_id: ObservedWindowId,
    pub signal: WindowSignal,
}

pub fn apply_window_event_to_tracker(
    tracker: &mut CustomZoomTracker,
    candidate: &WindowCandidate,
    event: ManagedWindowEvent,
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Option<ClampOperation> {
    let _window_id = event.window_id;
    tracker.handle_window_signal(candidate, native_area, custom_area, event.signal)
}

#[derive(Clone, Debug)]
pub struct AxEventDispatcher {
    native_area: WorkingArea,
    custom_area: WorkingArea,
    tracker: CustomZoomTracker,
}

impl AxEventDispatcher {
    pub fn new(native_area: WorkingArea, custom_area: WorkingArea) -> Self {
        Self {
            native_area,
            custom_area,
            tracker: CustomZoomTracker::default(),
        }
    }

    pub fn apply_event(
        &mut self,
        candidate: &WindowCandidate,
        event: ManagedWindowEvent,
    ) -> Option<ClampOperation> {
        apply_window_event_to_tracker(
            &mut self.tracker,
            candidate,
            event,
            self.native_area,
            self.custom_area,
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AxObserverRegistry {
    registered_pids: BTreeSet<i32>,
}

impl AxObserverRegistry {
    pub fn register_pid(&mut self, pid: i32) -> bool {
        self.registered_pids.insert(pid)
    }

    pub fn remove_pid(&mut self, pid: i32) -> bool {
        self.registered_pids.remove(&pid)
    }

    pub fn is_registered(&self, pid: i32) -> bool {
        self.registered_pids.contains(&pid)
    }
}

#[cfg(target_os = "macos")]
type AXError = i32;
#[cfg(target_os = "macos")]
type AXObserverRef = *const c_void;
#[cfg(target_os = "macos")]
type AXUIElementRef = *const c_void;

#[cfg(target_os = "macos")]
const AX_ERROR_SUCCESS: AXError = 0;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXObserverCreate(
        application: i32,
        callback: extern "C" fn(AXObserverRef, AXUIElementRef, CFStringRef, *mut c_void),
        out_observer: *mut AXObserverRef,
    ) -> AXError;
    fn AXObserverAddNotification(
        observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
        refcon: *mut c_void,
    ) -> AXError;
    fn AXObserverGetRunLoopSource(observer: AXObserverRef) -> core_foundation::runloop::CFRunLoopSourceRef;
}

#[cfg(target_os = "macos")]
struct AxClampContext {
    pid: i32,
    clamp_scheduler: ClampScheduler,
    native_area: WorkingArea,
    custom_area: WorkingArea,
}

#[cfg(target_os = "macos")]
pub struct AxFrontmostObserver {
    pid: i32,
    observer: AXObserverRef,
    application: AXUIElementRef,
    run_loop_source: CFRunLoopSource,
    _callback_context: Box<AxClampContext>,
}

#[cfg(target_os = "macos")]
impl std::fmt::Debug for AxFrontmostObserver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AxFrontmostObserver")
            .field("pid", &self.pid)
            .finish_non_exhaustive()
    }
}

#[cfg(target_os = "macos")]
impl AxFrontmostObserver {
    pub fn start_frontmost(
        clamp_scheduler: ClampScheduler,
        native_area: WorkingArea,
        custom_area: WorkingArea,
    ) -> Result<Option<Self>, String> {
        if !unsafe { AXIsProcessTrusted() != 0 } {
            return Err(
                "dors requires Accessibility permission for AX window management".to_string(),
            );
        }

        let Some(pid) = frontmost_application_pid() else {
            return Ok(None);
        };

        let application = unsafe { AXUIElementCreateApplication(pid) };
        if application.is_null() {
            return Err(format!("failed to create AX application element for pid {pid}"));
        }

        let mut observer = std::ptr::null();
        let create_error = unsafe {
            AXObserverCreate(pid, ax_observer_callback, &mut observer)
        };
        if create_error != AX_ERROR_SUCCESS || observer.is_null() {
            unsafe {
                CFRelease(application.cast());
            }
            return Err(format!(
                "failed to create AX observer for pid {pid}: error {create_error}"
            ));
        }

        let run_loop_source_ref = unsafe { AXObserverGetRunLoopSource(observer) };
        if run_loop_source_ref.is_null() {
            unsafe {
                CFRelease(observer.cast());
                CFRelease(application.cast());
            }
            return Err(format!("failed to obtain AX run loop source for pid {pid}"));
        }

        let run_loop_source = unsafe { CFRunLoopSource::wrap_under_get_rule(run_loop_source_ref) };
        let run_loop = CFRunLoop::get_main();
        run_loop.add_source(&run_loop_source, unsafe { kCFRunLoopDefaultMode });

        let callback_context = Box::new(AxClampContext {
            pid,
            clamp_scheduler,
            native_area,
            custom_area,
        });
        let refcon = (&*callback_context as *const AxClampContext).cast_mut().cast();

        let mut any_notification_registered = false;
        for notification_name in observer_notification_names() {
            let notification = CFString::new(notification_name);
            let add_error = unsafe {
                AXObserverAddNotification(
                    observer,
                    application,
                    notification.as_concrete_TypeRef(),
                    refcon,
                )
            };
            if add_error == AX_ERROR_SUCCESS {
                any_notification_registered = true;
            } else {
                if crate::native_app::window_clamper::debug_logging_enabled() {
                    eprintln!(
                        "[dors-debug] ax observer notification registration failed pid={} notification={} error={}",
                        pid, notification_name, add_error
                    );
                }
            }
        }

        if !any_notification_registered {
            run_loop.remove_source(&run_loop_source, unsafe { kCFRunLoopDefaultMode });
            unsafe {
                CFRelease(observer.cast());
                CFRelease(application.cast());
            }
            return Err(format!(
                "failed to register any AX notifications for pid {pid}"
            ));
        }

        Ok(Some(Self {
            pid,
            observer,
            application,
            run_loop_source,
            _callback_context: callback_context,
        }))
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }
}

#[cfg(target_os = "macos")]
impl Drop for AxFrontmostObserver {
    fn drop(&mut self) {
        let run_loop = CFRunLoop::get_main();
        run_loop.remove_source(&self.run_loop_source, unsafe { kCFRunLoopDefaultMode });
        unsafe {
            CFRelease(self.observer.cast());
            CFRelease(self.application.cast());
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" fn ax_observer_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut c_void,
) {
    let Some(context) = (unsafe { (refcon as *const AxClampContext).as_ref() }) else {
        return;
    };
    let notification_name = unsafe { CFString::wrap_under_get_rule(notification) }.to_string();
    if crate::native_app::window_clamper::debug_logging_enabled() {
        eprintln!("[dors-debug] ax event notification={notification_name}");
    }
    let Some(kind) = normalize_notification_name(&notification_name) else {
        return;
    };
    if !matches!(kind, WindowEventKind::Resized | WindowEventKind::Moved) {
        return;
    }
    let clamp_scheduler = context.clamp_scheduler.clone();
    let pid = context.pid;
    let native_area = context.native_area;
    let custom_area = context.custom_area;
    let retained_element = if matches!(kind, WindowEventKind::Resized | WindowEventKind::Moved)
        && !element.is_null()
    {
        unsafe {
            CFRetain(element.cast());
        }
        Some(element as usize)
    } else {
        None
    };
    clamp_scheduler
        .schedule_coalesced(move || {
            let direct_result = retained_element.map(|retained| {
                let retained = retained as *const c_void;
                let result = crate::native_app::window_clamper::clamp_ax_window_with_managed_zoom(
                    pid,
                    retained,
                    native_area,
                    custom_area,
                );
                unsafe {
                    CFRelease(retained.cast());
                }
                result
            });

            match direct_result {
                Some(Ok(true)) => Ok(()),
                Some(Ok(false)) | Some(Err(_)) | None => {
                    crate::native_app::window_clamper::clamp_windows_for_pid_with_managed_zoom(
                        pid,
                        native_area,
                        custom_area,
                    )
                }
            }
        });
}

#[cfg(target_os = "macos")]
pub fn frontmost_application_pid() -> Option<i32> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let application = workspace.frontmostApplication()?;
    Some(application.processIdentifier())
}
