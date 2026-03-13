use crate::native_app::system_dock::DockWindowCandidate;

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
use core_graphics2::window::{copy_window_info, kCGNullWindowID, CGWindowListOption, WindowKeys};
#[cfg(target_os = "macos")]
use std::ffi::{CString, c_char, c_void};

#[derive(Clone, Debug, PartialEq)]
pub struct DockVisualTarget {
    pub window_id: u32,
    pub owner_name: String,
    pub layer: i32,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub alpha: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DockSuppressionPlan {
    pub targets: Vec<DockVisualTarget>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockVisualSnapshot {
    pub window_id: u32,
    pub alpha_before: f64,
}

pub fn plan_visual_suppression(windows: &[DockWindowCandidate]) -> DockSuppressionPlan {
    let targets = windows
        .iter()
        .filter(|window| {
            let is_dock_container = window.owner_name == "Dock"
                && window.layer == 20
                && window.y.abs() < 1.0
                && window.width >= 1000.0;
            let is_window_server_strip = window.owner_name == "Window Server"
                && window.layer == 24
                && window.y.abs() < 1.0
                && (30.0..=60.0).contains(&window.height);

            is_dock_container || is_window_server_strip
        })
        .enumerate()
        .map(|(index, window)| DockVisualTarget {
            window_id: index as u32,
            owner_name: window.owner_name.clone(),
            layer: window.layer,
            width: window.width,
            height: window.height,
            x: window.x,
            y: window.y,
            alpha: 1.0,
        })
        .collect();

    DockSuppressionPlan { targets }
}

pub fn build_restore_snapshots(targets: &[DockVisualTarget]) -> Vec<DockVisualSnapshot> {
    targets
        .iter()
        .map(|target| DockVisualSnapshot {
            window_id: target.window_id,
            alpha_before: target.alpha,
        })
        .collect()
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
pub struct DockVisualGuard {
    connection_id: i32,
    snapshots: Vec<DockVisualSnapshot>,
    restart_dock_on_drop: bool,
}

#[cfg(target_os = "macos")]
impl DockVisualGuard {
    pub fn restore(self) -> Result<(), String> {
        restore_visual_state(self.connection_id, &self.snapshots)
    }
}

#[cfg(target_os = "macos")]
impl Drop for DockVisualGuard {
    fn drop(&mut self) {
        let _ = restore_visual_state(self.connection_id, &self.snapshots);
        if self.restart_dock_on_drop {
            let _ = std::process::Command::new("killall").arg("Dock").status();
        }
    }
}

#[cfg(target_os = "macos")]
pub fn query_dock_visual_targets() -> Result<Vec<DockVisualTarget>, String> {
    let window_info = copy_window_info(
        CGWindowListOption::OnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
        kCGNullWindowID,
    )
    .ok_or_else(|| "failed to query on-screen windows".to_string())?;
    let typed_windows: CFArray<CFType> =
        unsafe { TCFType::wrap_under_get_rule(window_info.as_concrete_TypeRef()) };

    let targets = typed_windows
        .iter()
        .filter_map(|entry| dock_visual_target_from_type(&entry))
        .collect::<Vec<_>>();

    Ok(plan_visual_suppression_from_targets(&targets).targets)
}

#[cfg(target_os = "macos")]
pub fn hide_dock_visuals() -> Result<DockVisualGuard, String> {
    let targets = query_dock_visual_targets()?;
    let snapshots = build_restore_snapshots(&targets);
    let connection_id = skylight_main_connection_id()?;
    let mut applied_count = 0usize;
    let mut errors = Vec::new();

    eprintln!(
        "[dock-hide-experiment] applying order-out suppression to {} targets",
        targets.len()
    );
    for target in &targets {
        eprintln!(
            "[dock-hide-experiment] before target id={} owner={} layer={} alpha={} frame=({}, {}, {}, {})",
            target.window_id,
            target.owner_name,
            target.layer,
            target.alpha,
            target.x,
            target.y,
            target.width,
            target.height
        );
    }

    for target in &targets {
        let ordered_in_before = skylight_window_is_ordered_in(connection_id, target.window_id)?;
        eprintln!(
            "[dock-hide-experiment] ordered-in before target id={} value={}",
            target.window_id, ordered_in_before
        );

        let error = skylight_order_window_out(connection_id, target.window_id)?;
        if error != 0 {
            eprintln!(
                "[dock-hide-experiment] order-out target id={} error={}",
                target.window_id, error
            );
            errors.push((target.window_id, error));
            continue;
        }
        let ordered_in_after = skylight_window_is_ordered_in(connection_id, target.window_id)?;
        eprintln!(
            "[dock-hide-experiment] order-out target id={} error={} ordered-in-after={}",
            target.window_id, error, ordered_in_after
        );
        applied_count += 1;
    }

    let after_targets = query_dock_visual_targets()?;
    for target in &after_targets {
        eprintln!(
            "[dock-hide-experiment] after  target id={} owner={} layer={} alpha={} frame=({}, {}, {}, {})",
            target.window_id,
            target.owner_name,
            target.layer,
            target.alpha,
            target.x,
            target.y,
            target.width,
            target.height
        );
    }

    if applied_count == 0 {
        return Err(format!(
            "failed to hide any Dock visual targets via order-out: {:?}",
            errors
        ));
    }

    Ok(DockVisualGuard {
        connection_id,
        snapshots,
        restart_dock_on_drop: true,
    })
}

fn plan_visual_suppression_from_targets(targets: &[DockVisualTarget]) -> DockSuppressionPlan {
    DockSuppressionPlan {
        targets: targets
            .iter()
            .filter(|window| {
                let is_dock_container = window.owner_name == "Dock"
                    && window.layer == 20
                    && window.y.abs() < 1.0
                    && window.width >= 1000.0;
                let is_window_server_strip = window.owner_name == "Window Server"
                    && window.layer == 24
                    && window.y.abs() < 1.0
                    && (30.0..=60.0).contains(&window.height);

                is_dock_container || is_window_server_strip
            })
            .cloned()
            .collect(),
    }
}

#[cfg(target_os = "macos")]
fn restore_visual_state(connection_id: i32, snapshots: &[DockVisualSnapshot]) -> Result<(), String> {
    for snapshot in snapshots {
        let error = skylight_order_window_in(connection_id, snapshot.window_id)?;
        if error != 0 {
            return Err(format!(
                "failed to restore Dock visual target {} with order-in error {}",
                snapshot.window_id, error
            ));
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn dock_visual_target_from_type(entry: &CFType) -> Option<DockVisualTarget> {
    let dictionary = entry.downcast::<CFDictionary>()?;
    let owner_name = dictionary_string_value(&dictionary, WindowKeys::OwnerName)?;
    let layer = dictionary_number_value(&dictionary, WindowKeys::Layer)?;
    let bounds = dictionary_rect_value(&dictionary, WindowKeys::Bounds)?;
    let window_id = dictionary_number_value(&dictionary, WindowKeys::Number)? as u32;
    let alpha = dictionary_float_value(&dictionary, WindowKeys::Alpha).unwrap_or(1.0);

    Some(DockVisualTarget {
        window_id,
        owner_name,
        layer,
        width: bounds.width(),
        height: bounds.height(),
        x: bounds.origin.x,
        y: bounds.origin.y,
        alpha,
    })
}

#[cfg(target_os = "macos")]
fn dictionary_string_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<String> {
    let value = dictionary_value(dictionary, key)?;
    value
        .downcast::<CFString>()
        .map(|string| string.to_string())
}

#[cfg(target_os = "macos")]
fn dictionary_number_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<i32> {
    let value = dictionary_value(dictionary, key)?;
    value.downcast::<CFNumber>()?.to_i32()
}

#[cfg(target_os = "macos")]
fn dictionary_float_value(dictionary: &CFDictionary, key: WindowKeys) -> Option<f64> {
    let value = dictionary_value(dictionary, key)?;
    value.downcast::<CFNumber>()?.to_f64()
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

#[cfg(target_os = "macos")]
type SlsMainConnectionIdFn = unsafe extern "C" fn() -> i32;
#[cfg(target_os = "macos")]
type SlsOrderWindowFn =
    unsafe extern "C" fn(connection_id: i32, window_id: u32, order: i32, relative_window_id: u32) -> i32;
#[cfg(target_os = "macos")]
type SlsWindowIsOrderedInFn =
    unsafe extern "C" fn(connection_id: i32, window_id: u32, value: *mut u8) -> i32;
#[cfg(target_os = "macos")]
type SlsTransactionCreateFn = unsafe extern "C" fn(connection_id: i32) -> *const c_void;
#[cfg(target_os = "macos")]
type SlstTransactionSetWindowSystemAlphaFn =
    unsafe extern "C" fn(transaction: *const c_void, window_id: u32, alpha: f32) -> i32;
#[cfg(target_os = "macos")]
type SlstTransactionCommitFn = unsafe extern "C" fn(transaction: *const c_void, synchronous: i32) -> i32;

#[cfg(target_os = "macos")]
fn skylight_main_connection_id() -> Result<i32, String> {
    let symbol = resolve_skylight_symbol::<SlsMainConnectionIdFn>("SLSMainConnectionID")?;
    Ok(unsafe { symbol() })
}

fn skylight_order_window_out(connection_id: i32, window_id: u32) -> Result<i32, String> {
    let symbol = resolve_skylight_symbol::<SlsOrderWindowFn>("SLSOrderWindow")?;
    Ok(unsafe { symbol(connection_id, window_id, 0, 0) })
}

#[cfg(target_os = "macos")]
fn skylight_order_window_in(connection_id: i32, window_id: u32) -> Result<i32, String> {
    let symbol = resolve_skylight_symbol::<SlsOrderWindowFn>("SLSOrderWindow")?;
    Ok(unsafe { symbol(connection_id, window_id, 1, 0) })
}

#[cfg(target_os = "macos")]
fn skylight_window_is_ordered_in(connection_id: i32, window_id: u32) -> Result<bool, String> {
    let symbol = resolve_skylight_symbol::<SlsWindowIsOrderedInFn>("SLSWindowIsOrderedIn")?;
    let mut value = 0u8;
    let error = unsafe { symbol(connection_id, window_id, &mut value) };
    if error != 0 {
        return Err(format!(
            "failed to query ordered-in state for window {} with error {}",
            window_id, error
        ));
    }

    Ok(value != 0)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn skylight_set_window_system_alpha(
    connection_id: i32,
    window_id: u32,
    alpha: f32,
) -> Result<i32, String> {
    let create = resolve_skylight_symbol::<SlsTransactionCreateFn>("SLSTransactionCreate")?;
    let set_alpha = resolve_skylight_symbol::<SlstTransactionSetWindowSystemAlphaFn>(
        "SLSTransactionSetWindowSystemAlpha",
    )?;
    let commit = resolve_skylight_symbol::<SlstTransactionCommitFn>("SLSTransactionCommit")?;

    let transaction = unsafe { create(connection_id) };
    if transaction.is_null() {
        return Err("failed to create SkyLight transaction".to_string());
    }

    let set_result = unsafe { set_alpha(transaction, window_id, alpha) };
    if set_result != 0 {
        return Ok(set_result);
    }

    Ok(unsafe { commit(transaction, 1) })
}

#[cfg(target_os = "macos")]
fn resolve_skylight_symbol<T>(name: &str) -> Result<T, String>
where
    T: Copy,
{
    let path = CString::new("/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight")
        .map_err(|error| error.to_string())?;
    let symbol_name = CString::new(name).map_err(|error| error.to_string())?;

    let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        return Err(format!("failed to dlopen SkyLight for symbol {name}"));
    }

    let symbol = unsafe { dlsym(handle, symbol_name.as_ptr()) };
    if symbol.is_null() {
        return Err(format!("failed to resolve SkyLight symbol {name}"));
    }

    Ok(unsafe { std::mem::transmute_copy::<*mut c_void, T>(&symbol) })
}

#[cfg(target_os = "macos")]
const RTLD_NOW: i32 = 2;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn dlopen(path: *const c_char, mode: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
