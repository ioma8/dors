use std::fmt::{Display, Formatter};
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "macos")]
use core_foundation::array::CFArray;
#[cfg(target_os = "macos")]
use core_foundation::base::{CFType, TCFType};
#[cfg(target_os = "macos")]
use core_foundation::dictionary::CFDictionary;
#[cfg(target_os = "macos")]
use core_foundation::number::CFNumber;
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;
#[cfg(target_os = "macos")]
use core_graphics2::geometry::CGRect;
#[cfg(target_os = "macos")]
use core_graphics2::window::{CGWindowListOption, WindowKeys, copy_window_info, kCGNullWindowID};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DockPreferencesSnapshot {
    pub autohide_before: bool,
    pub tilesize_before: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DockPreferencePlan {
    snapshot: DockPreferencesSnapshot,
    target_tilesize: i32,
}

impl DockPreferencePlan {
    pub fn from_snapshot(snapshot: DockPreferencesSnapshot, target_tilesize: i32) -> Self {
        Self {
            snapshot,
            target_tilesize,
        }
    }

    pub fn target_autohide(self) -> bool {
        false
    }

    pub fn target_tilesize(self) -> i32 {
        self.target_tilesize
    }

    pub fn restore_autohide(self) -> Option<bool> {
        (self.snapshot.autohide_before != self.target_autohide()).then_some(self.snapshot.autohide_before)
    }

    pub fn restore_tilesize(self) -> Option<Option<i32>> {
        (self.snapshot.tilesize_before != Some(self.target_tilesize)).then_some(self.snapshot.tilesize_before)
    }
}

pub fn parse_autohide_output(output: &str) -> Option<bool> {
    match output.trim() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

pub fn parse_tilesize_output(output: &str) -> Option<i32> {
    output.trim().parse().ok()
}

pub fn adjust_tilesize_guess(current_tilesize: i32, observed_reserved_height: i32, target_reserved_height: i32) -> i32 {
    if observed_reserved_height <= 0 || target_reserved_height <= 0 {
        return current_tilesize.max(16);
    }

    let scaled =
        ((current_tilesize as f64) * (target_reserved_height as f64)
            / (observed_reserved_height as f64))
            .round() as i32;

    scaled.clamp(16, 128)
}

pub fn is_missing_dock_process_error(status: Option<i32>, stderr: &str) -> bool {
    status == Some(1) && stderr.contains("No matching processes belonging to you were found")
}

pub fn bottom_reserved_height_from_frames(frame_origin_y: f64, visible_origin_y: f64) -> i32 {
    (visible_origin_y - frame_origin_y).round().max(0.0) as i32
}

pub fn tilesize_for_desired_real_height(desired_height: i32) -> i32 {
    interpolate_tilesize(desired_height, (52, 32), (90, 64), (156, 128)).clamp(16, 128)
}

fn interpolate_tilesize(
    desired_height: i32,
    first_anchor: (i32, i32),
    second_anchor: (i32, i32),
    third_anchor: (i32, i32),
) -> i32 {
    let (height_a, tilesize_a) = first_anchor;
    let (height_b, tilesize_b) = second_anchor;
    let (height_c, tilesize_c) = third_anchor;

    if desired_height <= height_b {
        interpolate_segment(desired_height, height_a, tilesize_a, height_b, tilesize_b)
    } else {
        interpolate_segment(desired_height, height_b, tilesize_b, height_c, tilesize_c)
    }
}

fn interpolate_segment(
    desired_height: i32,
    start_height: i32,
    start_tilesize: i32,
    end_height: i32,
    end_tilesize: i32,
) -> i32 {
    if desired_height <= start_height {
        return start_tilesize;
    }
    if desired_height >= end_height {
        return end_tilesize;
    }

    let height_span = (end_height - start_height) as f64;
    let tilesize_span = (end_tilesize - start_tilesize) as f64;
    let progress = (desired_height - start_height) as f64 / height_span;

    (start_tilesize as f64 + progress * tilesize_span).round() as i32
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockWindowCandidate {
    pub owner_name: String,
    pub window_name: Option<String>,
    pub layer: i32,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

pub fn select_dock_window_height(windows: &[DockWindowCandidate]) -> Option<i32> {
    windows
        .iter()
        .filter(|window| window.owner_name == "Dock" && window.layer == 20)
        .min_by(|left, right| {
            left.y
                .partial_cmp(&right.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .width
                        .partial_cmp(&left.width)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .map(|window| window.height.round() as i32)
}

#[derive(Debug)]
pub enum SystemDockError {
    CommandFailed {
        program: &'static str,
        status: Option<i32>,
        stderr: String,
    },
    InvalidAutohideOutput(String),
    InvalidTilesizeOutput(String),
    MainThreadUnavailable,
    ScreenUnavailable,
    DockWindowUnavailable,
    SignalHandlerInstall(String),
}

impl Display for SystemDockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed {
                program,
                status,
                stderr,
            } => write!(
                f,
                "{program} failed with status {:?}: {}",
                status,
                stderr.trim()
            ),
            Self::InvalidAutohideOutput(output) => {
                write!(f, "unexpected Dock autohide output: {}", output.trim())
            }
            Self::InvalidTilesizeOutput(output) => {
                write!(f, "unexpected Dock tilesize output: {}", output.trim())
            }
            Self::MainThreadUnavailable => write!(f, "main thread unavailable for screen measurement"),
            Self::ScreenUnavailable => write!(f, "main screen unavailable for Dock measurement"),
            Self::DockWindowUnavailable => write!(f, "Dock window unavailable for measurement"),
            Self::SignalHandlerInstall(error) => {
                write!(f, "failed to install Dock restore signal handler: {error}")
            }
        }
    }
}

impl std::error::Error for SystemDockError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DockPreferencesGuard {
    restore_autohide: Option<bool>,
    restore_tilesize: Option<Option<i32>>,
}

impl DockPreferencesGuard {
    pub fn noop() -> Self {
        Self {
            restore_autohide: None,
            restore_tilesize: None,
        }
    }

    pub fn restore(self) -> Result<(), SystemDockError> {
        if let Some(restore_autohide) = self.restore_autohide {
            set_autohide(restore_autohide)?;
        }
        if let Some(restore_tilesize) = self.restore_tilesize {
            match restore_tilesize {
                Some(value) => set_tilesize(value)?,
                None => delete_tilesize()?,
            }
        }
        if self.restore_autohide.is_some() || self.restore_tilesize.is_some() {
            restart_dock()?;
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub type SharedDockPreferencesGuard = Arc<Mutex<Option<DockPreferencesGuard>>>;

#[cfg(target_os = "macos")]
pub fn read_preferences_snapshot() -> Result<DockPreferencesSnapshot, SystemDockError> {
    let autohide_output = run_command("defaults", &["read", "com.apple.dock", "autohide"])?;
    let autohide_before = parse_autohide_output(&autohide_output)
        .ok_or_else(|| SystemDockError::InvalidAutohideOutput(autohide_output.clone()))?;
    let tilesize_before = read_optional_tilesize()?;

    Ok(DockPreferencesSnapshot {
        autohide_before,
        tilesize_before,
    })
}

#[cfg(target_os = "macos")]
pub fn set_autohide(enabled: bool) -> Result<(), SystemDockError> {
    let value = if enabled { "true" } else { "false" };
    let _ = run_command(
        "defaults",
        &["write", "com.apple.dock", "autohide", "-bool", value],
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_tilesize(tilesize: i32) -> Result<(), SystemDockError> {
    let value = tilesize.to_string();
    let _ = run_command(
        "defaults",
        &["write", "com.apple.dock", "tilesize", "-int", &value],
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn delete_tilesize() -> Result<(), SystemDockError> {
    let _ = run_command("defaults", &["delete", "com.apple.dock", "tilesize"])?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn restart_dock() -> Result<(), SystemDockError> {
    match run_command("killall", &["Dock"]) {
        Ok(_) => Ok(()),
        Err(SystemDockError::CommandFailed {
            status, stderr, ..
        }) if is_missing_dock_process_error(status, &stderr) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(target_os = "macos")]
pub fn prepare_overlay_dock_mode(
    target_tilesize: i32,
) -> Result<DockPreferencesGuard, SystemDockError> {
    let snapshot = read_preferences_snapshot()?;
    let target_tilesize = tilesize_for_desired_real_height(target_tilesize);
    let plan = DockPreferencePlan::from_snapshot(snapshot, target_tilesize);

    set_autohide(plan.target_autohide())?;
    set_tilesize(plan.target_tilesize())?;
    restart_dock()?;

    Ok(DockPreferencesGuard {
        restore_autohide: plan.restore_autohide(),
        restore_tilesize: plan.restore_tilesize(),
    })
}

#[cfg(target_os = "macos")]
pub fn calibrate_tilesize_for_target(target_reserved_height: i32) -> Result<(), SystemDockError> {
    let tilesize = tilesize_for_desired_real_height(target_reserved_height);
    set_tilesize(tilesize)?;
    restart_dock()
}

#[cfg(target_os = "macos")]
pub fn measured_reserved_height_for_main_screen() -> Result<i32, SystemDockError> {
    measured_reserved_height()
}

#[cfg(target_os = "macos")]
pub fn measured_dock_window_height_for_main_screen() -> Result<i32, SystemDockError> {
    let candidates = dock_window_candidates_for_main_screen()?;

    select_dock_window_height(&candidates).ok_or(SystemDockError::DockWindowUnavailable)
}

#[cfg(target_os = "macos")]
pub fn dock_window_candidates_for_main_screen() -> Result<Vec<DockWindowCandidate>, SystemDockError> {
    let window_info = copy_window_info(
        CGWindowListOption::OnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
        kCGNullWindowID,
    )
    .ok_or(SystemDockError::DockWindowUnavailable)?;
    let typed_windows: CFArray<CFType> =
        unsafe { TCFType::wrap_under_get_rule(window_info.as_concrete_TypeRef()) };
    Ok(typed_windows
        .iter()
        .filter_map(|entry| dock_window_candidate_from_type(&entry))
        .collect::<Vec<_>>())
}

#[cfg(target_os = "macos")]
pub fn install_restore_signal_handler(
    shared_guard: SharedDockPreferencesGuard,
) -> Result<(), SystemDockError> {
    ctrlc::set_handler(move || {
        let _ = restore_shared_guard(&shared_guard);
        std::process::exit(0);
    })
    .map_err(|error| SystemDockError::SignalHandlerInstall(error.to_string()))
}

#[cfg(target_os = "macos")]
pub fn shared_guard(guard: DockPreferencesGuard) -> SharedDockPreferencesGuard {
    Arc::new(Mutex::new(Some(guard)))
}

#[cfg(target_os = "macos")]
pub fn restore_shared_guard(
    shared_guard: &SharedDockPreferencesGuard,
) -> Result<(), SystemDockError> {
    let guard = shared_guard
        .lock()
        .map_err(|error| SystemDockError::SignalHandlerInstall(error.to_string()))?
        .take()
        .unwrap_or_else(DockPreferencesGuard::noop);

    guard.restore()
}

#[cfg(target_os = "macos")]
fn read_optional_tilesize() -> Result<Option<i32>, SystemDockError> {
    match run_command("defaults", &["read", "com.apple.dock", "tilesize"]) {
        Ok(output) => parse_tilesize_output(&output)
            .map(Some)
            .ok_or(SystemDockError::InvalidTilesizeOutput(output)),
        Err(SystemDockError::CommandFailed {
            program,
            status,
            stderr,
        }) if status == Some(1) && stderr.contains("does not exist") => {
            let _ = program;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[cfg(target_os = "macos")]
fn run_command(program: &'static str, args: &[&str]) -> Result<String, SystemDockError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| SystemDockError::CommandFailed {
            program,
            status: None,
            stderr: error.to_string(),
        })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    Err(SystemDockError::CommandFailed {
        program,
        status: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[cfg(target_os = "macos")]
fn measured_reserved_height() -> Result<i32, SystemDockError> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let marker = MainThreadMarker::new().ok_or(SystemDockError::MainThreadUnavailable)?;
    let screen = NSScreen::mainScreen(marker).ok_or(SystemDockError::ScreenUnavailable)?;
    let frame = screen.frame();
    let visible_frame = screen.visibleFrame();
    let reserved = (frame.size.height - visible_frame.size.height).round() as i32;

    Ok(reserved.max(0))
}

#[cfg(target_os = "macos")]
pub fn measured_bottom_reserved_height_for_main_screen() -> Result<i32, SystemDockError> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let marker = MainThreadMarker::new().ok_or(SystemDockError::MainThreadUnavailable)?;
    let screen = NSScreen::mainScreen(marker).ok_or(SystemDockError::ScreenUnavailable)?;
    let frame = screen.frame();
    let visible_frame = screen.visibleFrame();

    Ok(bottom_reserved_height_from_frames(
        frame.origin.y,
        visible_frame.origin.y,
    ))
}

#[cfg(target_os = "macos")]
fn dock_window_candidate_from_type(entry: &CFType) -> Option<DockWindowCandidate> {
    let dictionary = entry.downcast::<CFDictionary>()?;
    let owner_name = dictionary_string_value(&dictionary, WindowKeys::OwnerName)?;
    let layer = dictionary_number_value(&dictionary, WindowKeys::Layer)?;
    let bounds = dictionary_rect_value(&dictionary, WindowKeys::Bounds)?;

    Some(DockWindowCandidate {
        owner_name,
        window_name: dictionary_string_value(&dictionary, WindowKeys::Name),
        layer,
        width: bounds.width(),
        height: bounds.height(),
        x: bounds.origin.x,
        y: bounds.origin.y,
    })
}

#[cfg(target_os = "macos")]
fn dictionary_string_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<String> {
    let value = dictionary_value(dictionary, key)?;
    value.downcast::<CFString>().map(|string| string.to_string())
}

#[cfg(target_os = "macos")]
fn dictionary_number_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<i32> {
    let value = dictionary_value(dictionary, key)?;
    value.downcast::<CFNumber>()?.to_i32()
}

#[cfg(target_os = "macos")]
fn dictionary_rect_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<CGRect> {
    let value = dictionary_value(dictionary, key)?;
    let bounds = value.downcast::<CFDictionary>()?;
    CGRect::from_dict_representation(&bounds)
}

#[cfg(target_os = "macos")]
fn dictionary_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<CFType> {
    let key = CFString::from(key);
    let raw = dictionary.find(key.as_CFTypeRef() as *const _)?;
    let raw_ptr = *raw;

    Some(unsafe { CFType::wrap_under_get_rule(raw_ptr as _) })
}
