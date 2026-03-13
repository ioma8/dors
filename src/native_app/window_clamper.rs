use std::collections::HashMap;
use std::env;
#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
#[cfg(target_os = "macos")]
use core_foundation::string::{CFString, CFStringRef};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkingArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowCandidate {
    pub owner_name: String,
    pub stable_key: String,
    pub frame: WindowFrame,
    pub is_standard: bool,
    pub is_resizable: bool,
    pub is_fullscreen: bool,
    pub is_visible: bool,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedWindow {
    window_index: usize,
    title: String,
    candidate: WindowCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClampOperation {
    ResizeToArea(WorkingArea),
    Restore(WindowFrame),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowSignal {
    PassiveObservation,
    GeometryChanged,
}

fn describe_operation(operation: ClampOperation) -> String {
    match operation {
        ClampOperation::ResizeToArea(area) => format!(
            "resize-to-area x={} y={} w={} h={}",
            area.x, area.y, area.width, area.height
        ),
        ClampOperation::Restore(frame) => format!(
            "restore x={} y={} w={} h={}",
            frame.x, frame.y, frame.width, frame.height
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomZoomState {
    restore_frame: WindowFrame,
    settling_observations_remaining: u8,
}

#[derive(Clone, Debug, Default)]
pub struct CustomZoomTracker {
    states: HashMap<String, CustomZoomState>,
    last_regular_frames: HashMap<String, WindowFrame>,
    last_seen_frames: HashMap<String, WindowFrame>,
}

pub fn debug_logging_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    #[cfg(target_os = "macos")]
    {
        *ENABLED.get_or_init(|| {
            env::var("DORS_DEBUG_WINDOWS")
                .ok()
                .or_else(|| env::var("DORS_DEBUG").ok())
                .map(|value| value != "0")
                .unwrap_or(false)
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn build_allowed_work_area(
    screen: ScreenFrame,
    top_reserved_height: i32,
    dock_height: i32,
) -> WorkingArea {
    let y = screen.y + top_reserved_height.max(0);
    let height = (screen.height - top_reserved_height.max(0) - dock_height.max(0)).max(0);

    WorkingArea {
        x: screen.x,
        y,
        width: screen.width.max(0),
        height,
    }
}

#[cfg(target_os = "macos")]
type AXError = i32;
#[cfg(target_os = "macos")]
type AXUIElementRef = *const c_void;
#[cfg(target_os = "macos")]
type AXValueRef = *const c_void;
#[cfg(target_os = "macos")]
type AXValueType = u32;
#[cfg(target_os = "macos")]
type CFArrayRef = *const c_void;
#[cfg(target_os = "macos")]
type CFBooleanRef = *const c_void;

#[cfg(target_os = "macos")]
const AX_ERROR_SUCCESS: AXError = 0;
#[cfg(target_os = "macos")]
const AX_VALUE_CGPOINT_TYPE: AXValueType = 1;
#[cfg(target_os = "macos")]
const AX_VALUE_CGSIZE_TYPE: AXValueType = 2;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;
    fn AXValueCreate(value_type: AXValueType, value_ptr: *const c_void) -> AXValueRef;
    fn AXValueGetType(value: AXValueRef) -> AXValueType;
    fn AXValueGetValue(value: AXValueRef, value_type: AXValueType, value_ptr: *mut c_void) -> u8;
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
    fn CFEqual(left: *const c_void, right: *const c_void) -> u8;
    fn CFBooleanGetValue(boolean: CFBooleanRef) -> u8;
}

pub fn clamp_window_frame(frame: WindowFrame, area: WorkingArea) -> Option<WindowFrame> {
    let clamped_height = frame.height.min(area.height).max(0);
    let top = area.y + area.height;
    let max_y = top - clamped_height;
    let clamped_y = frame.y.clamp(area.y, max_y);
    let clamped = WindowFrame {
        x: frame.x,
        y: clamped_y,
        width: frame.width,
        height: clamped_height,
    };

    (clamped != frame).then_some(clamped)
}

pub fn frame_matches_work_area(frame: WindowFrame, area: WorkingArea) -> bool {
    frame.x == area.x
        && frame.y == area.y
        && frame.width == area.width
        && frame.height == area.height
}

fn frame_matches_startup_maximized_heuristic(
    frame: WindowFrame,
    native_area: WorkingArea,
) -> bool {
    let x_matches = (frame.x - native_area.x).abs() <= 4;
    let y_matches = (frame.y - native_area.y).abs() <= 4;
    let width_matches = (frame.width - native_area.width).abs() <= 4;
    let min_height = (f64::from(native_area.height) * 0.8).round() as i32;
    let height_matches = frame.height >= min_height && frame.height <= native_area.height;

    x_matches && y_matches && width_matches && height_matches
}

pub fn normalize_ax_value(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("missing value") {
        ""
    } else {
        trimmed
    }
}

impl CustomZoomTracker {
    pub fn should_skip_frame(&self, candidate: &WindowCandidate) -> bool {
        self.last_seen_frames.get(&candidate.stable_key) == Some(&candidate.frame)
    }

    fn remember_frame(&mut self, candidate: &WindowCandidate) {
        self.last_seen_frames
            .insert(candidate.stable_key.clone(), candidate.frame);
    }

    pub fn plan_startup_operation(
        &mut self,
        candidate: &WindowCandidate,
        native_area: WorkingArea,
        custom_area: WorkingArea,
    ) -> Option<ClampOperation> {
        let key = &candidate.stable_key;
        let is_native_zoomed = frame_matches_work_area(candidate.frame, native_area);
        let is_custom_zoomed = frame_matches_work_area(candidate.frame, custom_area);
        let is_startup_maximized =
            is_native_zoomed || is_custom_zoomed
                || frame_matches_startup_maximized_heuristic(candidate.frame, native_area);

        if !is_startup_maximized {
            self.last_regular_frames.insert(key.clone(), candidate.frame);
            self.remember_frame(candidate);
            return None;
        }

        if is_custom_zoomed {
            self.remember_frame(candidate);
            return None;
        }

        if let Some(restore_frame) = self.last_regular_frames.get(key).copied() {
            self.states.insert(
                key.clone(),
                CustomZoomState {
                    restore_frame,
                    settling_observations_remaining: 4,
                },
            );
        } else {
            self.states.remove(key);
        }

        self.remember_frame(candidate);
        Some(ClampOperation::ResizeToArea(custom_area))
    }

    pub fn handle_window_signal(
        &mut self,
        candidate: &WindowCandidate,
        native_area: WorkingArea,
        custom_area: WorkingArea,
        signal: WindowSignal,
    ) -> Option<ClampOperation> {
        match signal {
            WindowSignal::PassiveObservation => {
                self.observe_window_frame(candidate, native_area, custom_area);
                None
            }
            WindowSignal::GeometryChanged => self.plan_operation(candidate, native_area, custom_area),
        }
    }

    pub fn observe_window_frame(
        &mut self,
        candidate: &WindowCandidate,
        native_area: WorkingArea,
        custom_area: WorkingArea,
    ) {
        let key = &candidate.stable_key;
        let is_native_zoomed = frame_matches_work_area(candidate.frame, native_area);
        let is_custom_zoomed = frame_matches_work_area(candidate.frame, custom_area);

        if !is_native_zoomed && !is_custom_zoomed {
            if let Some(state) = self.states.get_mut(key) {
                if state.settling_observations_remaining > 0 {
                    if debug_logging_enabled() {
                        eprintln!(
                            "[dors-debug] managed zoom observe key={} state=settling remaining={} frame=({}, {}, {}, {})",
                            key,
                            state.settling_observations_remaining,
                            candidate.frame.x,
                            candidate.frame.y,
                            candidate.frame.width,
                            candidate.frame.height
                        );
                    }
                    state.settling_observations_remaining -= 1;
                    self.remember_frame(candidate);
                    return;
                }
                if debug_logging_enabled() {
                    eprintln!(
                        "[dors-debug] managed zoom observe key={} state=clear-managed frame=({}, {}, {}, {})",
                        key,
                        candidate.frame.x,
                        candidate.frame.y,
                        candidate.frame.width,
                        candidate.frame.height
                    );
                }
                self.states.remove(key);
            }
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] managed zoom observe key={} state=record-regular frame=({}, {}, {}, {})",
                    key,
                    candidate.frame.x,
                    candidate.frame.y,
                    candidate.frame.width,
                    candidate.frame.height
                );
            }
            self.last_regular_frames.insert(key.clone(), candidate.frame);
            self.remember_frame(candidate);
        }
    }

    pub fn plan_operation(
        &mut self,
        candidate: &WindowCandidate,
        native_area: WorkingArea,
        custom_area: WorkingArea,
    ) -> Option<ClampOperation> {
        let key = &candidate.stable_key;
        let is_native_zoomed = frame_matches_work_area(candidate.frame, native_area);
        let is_custom_zoomed = frame_matches_work_area(candidate.frame, custom_area);
        let state_snapshot = self.states.get(key).copied();
        let last_regular = self.last_regular_frames.get(key).copied();

        if debug_logging_enabled() {
            eprintln!(
                "[dors-debug] managed zoom decide key={} native={} custom={} state_present={} settling_remaining={} last_regular={:?}",
                key,
                is_native_zoomed,
                is_custom_zoomed,
                state_snapshot.is_some(),
                state_snapshot
                    .map(|state| state.settling_observations_remaining)
                    .unwrap_or(0),
                last_regular
            );
        }

        if let Some(state) = state_snapshot {
            if is_native_zoomed {
                if debug_logging_enabled() {
                    eprintln!(
                        "[dors-debug] managed zoom decide key={} transition=restore restore_frame=({}, {}, {}, {})",
                        key,
                        state.restore_frame.x,
                        state.restore_frame.y,
                        state.restore_frame.width,
                        state.restore_frame.height
                    );
                }
                self.states.remove(key);
                self.last_regular_frames.insert(key.clone(), state.restore_frame);
                self.remember_frame(candidate);
                return Some(ClampOperation::Restore(state.restore_frame));
            }

            if is_custom_zoomed {
                self.remember_frame(candidate);
                return None;
            }

            self.states.remove(key);
            self.last_regular_frames.insert(key.clone(), candidate.frame);
            self.remember_frame(candidate);
            return clamp_window_frame(candidate.frame, custom_area).map(ClampOperation::Restore);
        }

        if is_native_zoomed {
            let restore_frame = self
                .last_regular_frames
                .get(key)
                .copied()
                .unwrap_or(candidate.frame);
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] managed zoom decide key={} transition=set-managed restore_frame=({}, {}, {}, {})",
                    key,
                    restore_frame.x,
                    restore_frame.y,
                    restore_frame.width,
                    restore_frame.height
                );
            }
            self.states.insert(
                key.clone(),
                CustomZoomState {
                    restore_frame,
                    settling_observations_remaining: 4,
                },
            );
            self.remember_frame(candidate);
            return Some(ClampOperation::ResizeToArea(custom_area));
        }

        if is_custom_zoomed {
            if let Some(restore_frame) = last_regular {
                if restore_frame != candidate.frame {
                    if debug_logging_enabled() {
                        eprintln!(
                            "[dors-debug] managed zoom decide key={} transition=rehydrate restore_frame=({}, {}, {}, {})",
                            key,
                            restore_frame.x,
                            restore_frame.y,
                            restore_frame.width,
                            restore_frame.height
                        );
                    }
                    self.states.insert(
                        key.clone(),
                        CustomZoomState {
                            restore_frame,
                            settling_observations_remaining: 4,
                        },
                    );
                }
            }
            self.remember_frame(candidate);
            return None;
        }

        self.last_regular_frames.insert(key.clone(), candidate.frame);
        self.remember_frame(candidate);
        clamp_window_frame(candidate.frame, custom_area).map(ClampOperation::Restore)
    }
}

pub fn should_clamp_candidate(candidate: &WindowCandidate, screen: ScreenFrame) -> bool {
    candidate.is_standard
        && candidate.is_resizable
        && candidate.is_visible
        && !candidate.is_fullscreen
        && candidate.frame.width >= 240
        && candidate.frame.height >= 160
        && intersects_screen(candidate.frame, screen)
}

fn intersects_screen(frame: WindowFrame, screen: ScreenFrame) -> bool {
    let frame_right = frame.x + frame.width;
    let frame_top = frame.y + frame.height;
    let screen_right = screen.x + screen.width;
    let screen_top = screen.y + screen.height;

    frame.x < screen_right && frame_right > screen.x && frame.y < screen_top && frame_top > screen.y
}

#[cfg(target_os = "macos")]
pub fn clamp_windows_in_area(area: WorkingArea) -> Result<(), String> {
    let script = build_clamp_script(area);
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

#[cfg(target_os = "macos")]
pub fn clamp_windows_with_managed_zoom(
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<(), String> {
    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] clamp_windows_with_managed_zoom native=x={} y={} w={} h={} custom=x={} y={} w={} h={}",
            native_area.x,
            native_area.y,
            native_area.width,
            native_area.height,
            custom_area.x,
            custom_area.y,
            custom_area.width,
            custom_area.height
        );
    }
    if let Err(error) = apply_managed_custom_zoom(native_area, custom_area) {
        eprintln!("[dors-debug] managed zoom pass failed: {error}");
    }
    if debug_logging_enabled() {
        eprintln!("[dors-debug] fallback clamp pass running");
    }
    clamp_windows_in_area(custom_area)
}

#[cfg(target_os = "macos")]
pub fn initialize_startup_window_states(
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<(), String> {
    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] initialize_startup_window_states native=x={} y={} w={} h={} custom=x={} y={} w={} h={}",
            native_area.x,
            native_area.y,
            native_area.width,
            native_area.height,
            custom_area.x,
            custom_area.y,
            custom_area.width,
            custom_area.height
        );
    }
    let windows = query_windows()?;
    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] initialize_startup_window_states observed_windows={}",
            windows.len()
        );
    }
    let tracker = custom_zoom_tracker();
    let mut tracker = tracker
        .lock()
        .map_err(|_| "failed to lock zoom tracker".to_string())?;
    let screen = ScreenFrame {
        x: custom_area.x,
        y: 0,
        width: custom_area.width,
        height: native_area.y + native_area.height,
    };

    for window in windows {
        if !should_clamp_candidate(&window.candidate, screen) {
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] startup skip key={} reason=not-clamp-candidate",
                    window.candidate.stable_key
                );
            }
            continue;
        }

        if let Some(operation) =
            tracker.plan_startup_operation(&window.candidate, native_area, custom_area)
        {
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] startup apply key={} action={}",
                    window.candidate.stable_key,
                    describe_operation(operation)
                );
            }
            apply_operation(&window, operation)?;
        } else if debug_logging_enabled() {
            eprintln!(
                "[dors-debug] startup seed key={} frame=({}, {}, {}, {})",
                window.candidate.stable_key,
                window.candidate.frame.x,
                window.candidate.frame.y,
                window.candidate.frame.width,
                window.candidate.frame.height
            );
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clamp_windows_for_pid_with_managed_zoom(
    pid: i32,
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<(), String> {
    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] clamp_windows_for_pid_with_managed_zoom pid={} native=x={} y={} w={} h={} custom=x={} y={} w={} h={}",
            pid,
            native_area.x,
            native_area.y,
            native_area.width,
            native_area.height,
            custom_area.x,
            custom_area.y,
            custom_area.width,
            custom_area.height
        );
    }
    let windows = query_windows_for_pid(pid)?;
    apply_managed_custom_zoom_to_windows(windows, native_area, custom_area)
}

#[cfg(target_os = "macos")]
pub fn clamp_ax_window_with_managed_zoom(
    pid: i32,
    window: *const c_void,
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<bool, String> {
    let Some(observed_window) = direct_ax_observed_window(pid, window.cast())? else {
        return Ok(false);
    };

    let screen = ScreenFrame {
        x: custom_area.x,
        y: 0,
        width: custom_area.width,
        height: native_area.y + native_area.height,
    };

    if !should_clamp_candidate(&observed_window.candidate, screen) {
        eprintln!(
            "[dors-debug] managed zoom skip key={} reason=not-clamp-candidate",
            observed_window.candidate.stable_key
        );
        return Ok(true);
    }

    let tracker = custom_zoom_tracker();
    let mut tracker = tracker
        .lock()
        .map_err(|_| "failed to lock zoom tracker".to_string())?;

    if tracker.should_skip_frame(&observed_window.candidate) {
        return Ok(true);
    }

    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] managed zoom inspect key={} owner={} title={:?} frame=({}, {}, {}, {})",
            observed_window.candidate.stable_key,
            observed_window.candidate.owner_name,
            if observed_window.title.is_empty() {
                None::<&str>
            } else {
                Some(observed_window.title.as_str())
            },
            observed_window.candidate.frame.x,
            observed_window.candidate.frame.y,
            observed_window.candidate.frame.width,
            observed_window.candidate.frame.height
        );
    }

    tracker.observe_window_frame(&observed_window.candidate, native_area, custom_area);

    let Some(operation) =
        tracker.plan_operation(&observed_window.candidate, native_area, custom_area)
    else {
        if debug_logging_enabled() {
            eprintln!(
                "[dors-debug] managed zoom no-op key={} reason=no-planned-operation",
                observed_window.candidate.stable_key
            );
        }
        return Ok(true);
    };

    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] managed zoom apply key={} action={}",
            observed_window.candidate.stable_key,
            describe_operation(operation)
        );
    }
    apply_ax_operation(window.cast(), operation)?;
    Ok(true)
}

#[cfg(target_os = "macos")]
pub fn capture_regular_window_frames(
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<(), String> {
    let windows = query_windows()?;
    let tracker = custom_zoom_tracker();
    let mut tracker = tracker
        .lock()
        .map_err(|_| "failed to lock zoom tracker".to_string())?;
    let screen = ScreenFrame {
        x: custom_area.x,
        y: 0,
        width: custom_area.width,
        height: native_area.y + native_area.height,
    };

    for window in windows {
        if should_clamp_candidate(&window.candidate, screen) {
            tracker.observe_window_frame(&window.candidate, native_area, custom_area);
        }
    }

    Ok(())
}

pub fn build_clamp_script_preview(area: WorkingArea) -> String {
    build_clamp_script(area)
}

pub fn build_query_windows_script_preview() -> String {
    build_query_windows_script().to_string()
}

pub fn build_query_windows_for_pid_script_preview(pid: i32) -> String {
    build_query_windows_for_pid_script(pid)
}

pub fn build_apply_operation_script_preview(
    owner_name: &str,
    window_index: usize,
    target_frame: WindowFrame,
) -> String {
    build_apply_operation_script(
        owner_name,
        window_index,
        target_frame,
    )
}

#[cfg(target_os = "macos")]
pub fn main_screen_work_areas(dock_height: i32) -> Result<(WorkingArea, WorkingArea), String> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let marker = MainThreadMarker::new()
        .ok_or_else(|| "screen measurement requires main thread".to_string())?;
    let screen =
        NSScreen::mainScreen(marker).ok_or_else(|| "no main screen available".to_string())?;
    let frame = screen.frame();
    let visible = screen.visibleFrame();
    let bottom_reserved = (visible.origin.y - frame.origin.y).round() as i32;
    let total_reserved = (frame.size.height - visible.size.height).round() as i32;
    let top_reserved = (total_reserved - bottom_reserved).max(0);
    let screen_frame = ScreenFrame {
        x: 0,
        y: 0,
        width: frame.size.width.round() as i32,
        height: frame.size.height.round() as i32,
    };

    Ok((
        build_allowed_work_area(screen_frame, top_reserved, 0),
        build_allowed_work_area(screen_frame, top_reserved, dock_height),
    ))
}

#[cfg(target_os = "macos")]
pub fn main_screen_allowed_work_area(dock_height: i32) -> Result<WorkingArea, String> {
    let (_, custom_area) = main_screen_work_areas(dock_height)?;
    Ok(custom_area)
}

#[cfg(target_os = "macos")]
fn custom_zoom_tracker() -> &'static Mutex<CustomZoomTracker> {
    static TRACKER: OnceLock<Mutex<CustomZoomTracker>> = OnceLock::new();
    TRACKER.get_or_init(|| Mutex::new(CustomZoomTracker::default()))
}

#[cfg(target_os = "macos")]
fn apply_managed_custom_zoom(native_area: WorkingArea, custom_area: WorkingArea) -> Result<(), String> {
    let windows = query_windows()?;
    apply_managed_custom_zoom_to_windows(windows, native_area, custom_area)
}

#[cfg(target_os = "macos")]
fn apply_managed_custom_zoom_to_windows(
    windows: Vec<ObservedWindow>,
    native_area: WorkingArea,
    custom_area: WorkingArea,
) -> Result<(), String> {
    if debug_logging_enabled() {
        eprintln!(
            "[dors-debug] managed zoom observed_windows={}",
            windows.len()
        );
    }
    let tracker = custom_zoom_tracker();
    let mut tracker = tracker.lock().map_err(|_| "failed to lock zoom tracker".to_string())?;
    let screen = ScreenFrame {
        x: custom_area.x,
        y: 0,
        width: custom_area.width,
        height: native_area.y + native_area.height,
    };

    for window in windows {
        if debug_logging_enabled() {
            eprintln!(
                "[dors-debug] managed zoom inspect key={} owner={} title={:?} frame=({}, {}, {}, {})",
                window.candidate.stable_key,
                window.candidate.owner_name,
                if window.title.is_empty() {
                    None::<&str>
                } else {
                    Some(window.title.as_str())
                },
                window.candidate.frame.x,
                window.candidate.frame.y,
                window.candidate.frame.width,
                window.candidate.frame.height
            );
        }
        if !should_clamp_candidate(&window.candidate, screen) {
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] managed zoom skip key={} reason=not-clamp-candidate",
                    window.candidate.stable_key
                );
            }
            continue;
        }
        if tracker.should_skip_frame(&window.candidate) {
            continue;
        }

        tracker.observe_window_frame(&window.candidate, native_area, custom_area);

        let Some(operation) = tracker.plan_operation(&window.candidate, native_area, custom_area) else {
            if debug_logging_enabled() {
                eprintln!(
                    "[dors-debug] managed zoom no-op key={} reason=no-planned-operation",
                    window.candidate.stable_key
                );
            }
            continue;
        };
        if debug_logging_enabled() {
            eprintln!(
                "[dors-debug] managed zoom apply key={} action={}",
                window.candidate.stable_key,
                describe_operation(operation)
            );
        }
        apply_operation(&window, operation)?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn query_windows() -> Result<Vec<ObservedWindow>, String> {
    let script = build_query_windows_script();
    query_windows_with_script(script.to_string())
}

#[cfg(target_os = "macos")]
fn query_windows_for_pid(pid: i32) -> Result<Vec<ObservedWindow>, String> {
    query_windows_with_script(build_query_windows_for_pid_script(pid))
}

#[cfg(target_os = "macos")]
fn query_windows_with_script(script: String) -> Result<Vec<ObservedWindow>, String> {
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(parse_window_rows(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(target_os = "macos")]
fn parse_window_rows(raw: &str) -> Vec<ObservedWindow> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let owner_name = parts.next()?.trim().to_string();
            let pid: i32 = parts.next()?.trim().parse().ok()?;
            let window_index: usize = parts.next()?.trim().parse().ok()?;
            let title = normalize_ax_value(parts.next()?).to_string();
            let document = normalize_ax_value(parts.next()?).to_string();
            let is_standard = parts.next()?.trim().eq_ignore_ascii_case("true");
            let is_fullscreen = parts.next()?.trim().eq_ignore_ascii_case("true");
            let x = parts.next()?.trim().parse().ok()?;
            let y = parts.next()?.trim().parse().ok()?;
            let width = parts.next()?.trim().parse().ok()?;
            let height = parts.next()?.trim().parse().ok()?;
            let stable_key = if !document.is_empty() {
                format!("pid-{pid}::{document}")
            } else {
                format!("pid-{pid}::window-{window_index}")
            };

            Some(ObservedWindow {
                window_index,
                title,
                candidate: WindowCandidate {
                    owner_name,
                    stable_key,
                    frame: WindowFrame { x, y, width, height },
                    is_standard,
                    is_resizable: true,
                    is_fullscreen,
                    is_visible: true,
                },
            })
        })
        .collect()
}

fn build_query_windows_script() -> &'static str {
    "tell application \"System Events\"\n\
set rowTexts to {}\n\
set procNames to name of every application process whose background only is false and visible is true\n\
repeat with procName in procNames\n\
if procName as text is not \"dors\" then\n\
set proc to application process (procName as text)\n\
set procPid to unix id of proc\n\
set winIndex to 0\n\
repeat with win in windows of proc\n\
set winIndex to winIndex + 1\n\
try\n\
set isStandard to true\n\
try\n\
set isStandard to (value of attribute \"AXSubrole\" of win is \"AXStandardWindow\")\n\
end try\n\
set isFullScreen to false\n\
try\n\
set isFullScreen to value of attribute \"AXFullScreen\" of win\n\
end try\n\
set {xPos, yPos} to position of win\n\
set {winWidth, winHeight} to size of win\n\
set windowTitle to \"\"\n\
try\n\
set windowTitle to name of win\n\
end try\n\
set windowDocument to \"\"\n\
try\n\
set windowDocument to value of attribute \"AXDocument\" of win\n\
end try\n\
copy ((procName as text) & tab & (procPid as text) & tab & (winIndex as text) & tab & windowTitle & tab & windowDocument & tab & (isStandard as text) & tab & (isFullScreen as text) & tab & (xPos as text) & tab & (yPos as text) & tab & (winWidth as text) & tab & (winHeight as text)) to end of rowTexts\n\
end try\n\
end repeat\n\
end if\n\
end repeat\n\
set AppleScript's text item delimiters to linefeed\n\
return rowTexts as text\n\
end tell"
}

fn build_query_windows_for_pid_script(pid: i32) -> String {
    format!(
        "tell application \"System Events\"\n\
set rowTexts to {{}}\n\
set proc to first application process whose unix id is {pid}\n\
set procName to name of proc\n\
set procPid to unix id of proc\n\
set winIndex to 0\n\
repeat with win in windows of proc\n\
set winIndex to winIndex + 1\n\
try\n\
set isStandard to true\n\
try\n\
set isStandard to (value of attribute \"AXSubrole\" of win is \"AXStandardWindow\")\n\
end try\n\
set isFullScreen to false\n\
try\n\
set isFullScreen to value of attribute \"AXFullScreen\" of win\n\
end try\n\
set {{xPos, yPos}} to position of win\n\
set {{winWidth, winHeight}} to size of win\n\
set windowTitle to \"\"\n\
try\n\
set windowTitle to name of win\n\
end try\n\
set windowDocument to \"\"\n\
try\n\
set windowDocument to value of attribute \"AXDocument\" of win\n\
end try\n\
copy ((procName as text) & tab & (procPid as text) & tab & (winIndex as text) & tab & windowTitle & tab & windowDocument & tab & (isStandard as text) & tab & (isFullScreen as text) & tab & (xPos as text) & tab & (yPos as text) & tab & (winWidth as text) & tab & (winHeight as text)) to end of rowTexts\n\
end try\n\
end repeat\n\
set AppleScript's text item delimiters to linefeed\n\
return rowTexts as text\n\
end tell",
        pid = pid
    )
}

#[cfg(target_os = "macos")]
fn apply_operation(window: &ObservedWindow, operation: ClampOperation) -> Result<(), String> {
    let target_frame = match operation {
        ClampOperation::ResizeToArea(area) => WindowFrame {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        },
        ClampOperation::Restore(frame) => frame,
    };
    let script = build_apply_operation_script(
        &window.candidate.owner_name,
        window.window_index,
        target_frame,
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

#[cfg(target_os = "macos")]
fn direct_ax_observed_window(
    pid: i32,
    window: AXUIElementRef,
) -> Result<Option<ObservedWindow>, String> {
    if window.is_null() {
        return Ok(None);
    }

    let application = unsafe { AXUIElementCreateApplication(pid) };
    if application.is_null() {
        return Ok(None);
    }

    let result = (|| {
        let windows_value = copy_ax_attribute(application, "AXWindows")?;
        let Some(windows_value) = windows_value else {
            return Ok(None);
        };

        let window_index = find_window_index_in_array(windows_value.cast(), window)?;
        unsafe {
            CFRelease(windows_value.cast());
        }
        let Some(window_index) = window_index else {
            return Ok(None);
        };

        let title = copy_ax_string_attribute(window, "AXTitle")?.unwrap_or_default();
        let document = copy_ax_string_attribute(window, "AXDocument")?.unwrap_or_default();
        let is_standard = copy_ax_string_attribute(window, "AXSubrole")?
            .map(|value| value == "AXStandardWindow")
            .unwrap_or(true);
        let is_fullscreen = copy_ax_bool_attribute(window, "AXFullScreen")?.unwrap_or(false);
        let frame = copy_ax_window_frame(window)?;
        let owner_name = format!("pid-{pid}");
        let stable_key = if !document.is_empty() {
            format!("pid-{pid}::{document}")
        } else {
            format!("pid-{pid}::window-{window_index}")
        };

        Ok(Some(ObservedWindow {
            window_index,
            title,
            candidate: WindowCandidate {
                owner_name,
                stable_key,
                frame,
                is_standard,
                is_resizable: true,
                is_fullscreen,
                is_visible: true,
            },
        }))
    })();

    unsafe {
        CFRelease(application.cast());
    }
    result
}

#[cfg(target_os = "macos")]
fn apply_ax_operation(window: AXUIElementRef, operation: ClampOperation) -> Result<(), String> {
    let target_frame = match operation {
        ClampOperation::ResizeToArea(area) => WindowFrame {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        },
        ClampOperation::Restore(frame) => frame,
    };

    let size = CGSize {
        width: f64::from(target_frame.width),
        height: f64::from(target_frame.height),
    };
    let position = CGPoint {
        x: f64::from(target_frame.x),
        y: f64::from(target_frame.y),
    };

    let size_value =
        unsafe { AXValueCreate(AX_VALUE_CGSIZE_TYPE, (&size as *const CGSize).cast()) };
    let position_value =
        unsafe { AXValueCreate(AX_VALUE_CGPOINT_TYPE, (&position as *const CGPoint).cast()) };
    if size_value.is_null() || position_value.is_null() {
        if !size_value.is_null() {
            unsafe {
                CFRelease(size_value.cast());
            }
        }
        if !position_value.is_null() {
            unsafe {
                CFRelease(position_value.cast());
            }
        }
        return Err("failed to create AX values for target frame".to_string());
    }

    let size_attribute = CFString::new("AXSize");
    let position_attribute = CFString::new("AXPosition");

    let size_error = unsafe {
        AXUIElementSetAttributeValue(
            window,
            size_attribute.as_concrete_TypeRef(),
            size_value.cast(),
        )
    };
    let position_error = unsafe {
        AXUIElementSetAttributeValue(
            window,
            position_attribute.as_concrete_TypeRef(),
            position_value.cast(),
        )
    };

    unsafe {
        CFRelease(size_value.cast());
        CFRelease(position_value.cast());
    }

    if size_error != AX_ERROR_SUCCESS {
        return Err(format!("failed to set AXSize: error {size_error}"));
    }
    if position_error != AX_ERROR_SUCCESS {
        return Err(format!("failed to set AXPosition: error {position_error}"));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn copy_ax_attribute(element: AXUIElementRef, attribute_name: &str) -> Result<Option<CFTypeRef>, String> {
    let attribute = CFString::new(attribute_name);
    let mut value = std::ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };
    if error == AX_ERROR_SUCCESS {
        return Ok((!value.is_null()).then_some(value));
    }
    Ok(None)
}

#[cfg(target_os = "macos")]
fn copy_ax_string_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<Option<String>, String> {
    let Some(value) = copy_ax_attribute(element, attribute_name)? else {
        return Ok(None);
    };
    let string = unsafe { core_foundation::string::CFString::wrap_under_create_rule(value.cast()) };
    Ok(Some(string.to_string()))
}

#[cfg(target_os = "macos")]
fn copy_ax_bool_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<Option<bool>, String> {
    let Some(value) = copy_ax_attribute(element, attribute_name)? else {
        return Ok(None);
    };
    let boolean = unsafe { CFBooleanGetValue(value.cast()) != 0 };
    unsafe {
        CFRelease(value.cast());
    }
    Ok(Some(boolean))
}

#[cfg(target_os = "macos")]
fn copy_ax_window_frame(element: AXUIElementRef) -> Result<WindowFrame, String> {
    let position_value = copy_ax_attribute(element, "AXPosition")?
        .ok_or_else(|| "AXPosition missing".to_string())?;
    let size_value =
        copy_ax_attribute(element, "AXSize")?.ok_or_else(|| "AXSize missing".to_string())?;

    let position = ax_value_to_point(position_value.cast(), AX_VALUE_CGPOINT_TYPE)?;
    let size = ax_value_to_size(size_value.cast(), AX_VALUE_CGSIZE_TYPE)?;

    unsafe {
        CFRelease(position_value.cast());
        CFRelease(size_value.cast());
    }

    Ok(WindowFrame {
        x: position.x.round() as i32,
        y: position.y.round() as i32,
        width: size.width.round() as i32,
        height: size.height.round() as i32,
    })
}

#[cfg(target_os = "macos")]
fn ax_value_to_point(value: AXValueRef, expected_type: AXValueType) -> Result<CGPoint, String> {
    if unsafe { AXValueGetType(value) } != expected_type {
        return Err("AXValue type mismatch for CGPoint".to_string());
    }
    let mut point = CGPoint { x: 0.0, y: 0.0 };
    let success =
        unsafe { AXValueGetValue(value, expected_type, (&mut point as *mut CGPoint).cast()) };
    if success == 0 {
        return Err("failed to read CGPoint from AXValue".to_string());
    }
    Ok(point)
}

#[cfg(target_os = "macos")]
fn ax_value_to_size(value: AXValueRef, expected_type: AXValueType) -> Result<CGSize, String> {
    if unsafe { AXValueGetType(value) } != expected_type {
        return Err("AXValue type mismatch for CGSize".to_string());
    }
    let mut size = CGSize {
        width: 0.0,
        height: 0.0,
    };
    let success =
        unsafe { AXValueGetValue(value, expected_type, (&mut size as *mut CGSize).cast()) };
    if success == 0 {
        return Err("failed to read CGSize from AXValue".to_string());
    }
    Ok(size)
}

#[cfg(target_os = "macos")]
fn find_window_index_in_array(array: CFArrayRef, target: AXUIElementRef) -> Result<Option<usize>, String> {
    if array.is_null() {
        return Ok(None);
    }

    let count = unsafe { CFArrayGetCount(array) };
    for index in 0..count {
        let candidate = unsafe { CFArrayGetValueAtIndex(array, index) };
        if candidate.is_null() {
            continue;
        }
        if unsafe { CFEqual(candidate, target.cast()) } != 0 {
            return Ok(Some(index as usize + 1));
        }
    }

    Ok(None)
}

#[cfg(target_os = "macos")]
fn build_apply_operation_script(
    owner_name: &str,
    window_index: usize,
    target_frame: WindowFrame,
) -> String {
    format!(
        "tell application \"System Events\"\n\
tell application process \"{owner}\"\n\
try\n\
set win to item {window_index} of windows\n\
try\n\
set size of win to {{{new_width}, {new_height}}}\n\
set position of win to {{{new_x}, {new_y}}}\n\
end try\n\
end try\n\
end tell\n\
end tell",
        owner = escape_applescript_string(owner_name),
        window_index = window_index,
        new_x = target_frame.x,
        new_y = target_frame.y,
        new_width = target_frame.width,
        new_height = target_frame.height,
    )
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
fn build_clamp_script(area: WorkingArea) -> String {
    let screen_right = area.x + area.width;
    let screen_bottom = area.y + area.height;

    format!(
        "tell application \"System Events\"\n\
repeat with proc in application processes\n\
if background only of proc is false and visible of proc is true and name of proc is not \"dors\" then\n\
repeat with win in windows of proc\n\
try\n\
set isStandard to true\n\
try\n\
set isStandard to (value of attribute \"AXSubrole\" of win is \"AXStandardWindow\")\n\
end try\n\
if isStandard then\n\
set isFullScreen to false\n\
try\n\
set isFullScreen to value of attribute \"AXFullScreen\" of win\n\
end try\n\
set {{xPos, yPos}} to position of win\n\
set {{winWidth, winHeight}} to size of win\n\
if isFullScreen is false and winWidth >= 240 and winHeight >= 160 and xPos < {screen_right} and (xPos + winWidth) > {screen_left} and yPos < {screen_bottom} and (yPos + winHeight) > {screen_top} then\n\
set newHeight to winHeight\n\
if newHeight > {area_height} then set newHeight to {area_height}\n\
set newY to yPos\n\
set maxY to {max_y_base} - newHeight\n\
if newY < {screen_top} then set newY to {screen_top}\n\
if newY > maxY then set newY to maxY\n\
if newHeight is not winHeight then set size of win to {{winWidth, newHeight}}\n\
if newY is not yPos then set position of win to {{xPos, newY}}\n\
end if\n\
end if\n\
end try\n\
end repeat\n\
end if\n\
end repeat\n\
end tell",
        screen_left = area.x,
        screen_top = area.y,
        area_height = area.height,
        max_y_base = area.y + area.height,
    )
}
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
