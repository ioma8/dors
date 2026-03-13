#[cfg(target_os = "macos")]
use std::ffi::c_void;

#[cfg(target_os = "macos")]
use core_foundation::base::{CFRelease, CFRetain, CFTypeRef, TCFType};
#[cfg(target_os = "macos")]
use core_foundation::number::CFBooleanGetValue;
#[cfg(target_os = "macos")]
use core_foundation::string::{CFString, CFStringRef};

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
#[derive(Debug)]
pub struct AxWindowRef(AXUIElementRef);

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunningApplicationInfo {
    pub pid: i32,
    pub name: String,
}

#[cfg(target_os = "macos")]
impl AxWindowRef {
    pub fn as_ptr(&self) -> AXUIElementRef {
        self.0
    }
}

#[cfg(target_os = "macos")]
impl Drop for AxWindowRef {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0.cast());
        }
    }
}

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
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> AXError;
    fn AXValueGetType(value: AXValueRef) -> AXValueType;
    fn AXValueGetValue(value: AXValueRef, value_type: AXValueType, value_ptr: *mut c_void) -> u8;
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
}

#[cfg(target_os = "macos")]
pub fn running_applications() -> Vec<RunningApplicationInfo> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    workspace
        .runningApplications()
        .iter()
        .filter_map(|application| {
            let name = application.localizedName()?.to_string();
            Some(RunningApplicationInfo {
                pid: application.processIdentifier(),
                name,
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
pub fn application_element(pid: i32) -> Option<AXUIElementRef> {
    let application = unsafe { AXUIElementCreateApplication(pid) };
    (!application.is_null()).then_some(application)
}

#[cfg(target_os = "macos")]
pub fn copy_windows(pid: i32) -> Result<Vec<AxWindowRef>, String> {
    let Some(application) = application_element(pid) else {
        return Ok(Vec::new());
    };

    let result = (|| {
        let Some(value) = copy_attribute(application, "AXWindows")? else {
            return Ok(Vec::new());
        };
        let array = value.cast();
        let count = unsafe { CFArrayGetCount(array) };
        let mut windows = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let window = unsafe { CFArrayGetValueAtIndex(array, index) };
            if !window.is_null() {
                let retained = unsafe { CFRetain(window) };
                if !retained.is_null() {
                    windows.push(AxWindowRef(retained.cast()));
                }
            }
        }
        unsafe {
            CFRelease(value.cast());
        }
        Ok(windows)
    })();

    unsafe {
        CFRelease(application.cast());
    }
    result
}

#[cfg(target_os = "macos")]
pub fn copy_string_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<Option<String>, String> {
    let Some(value) = copy_attribute(element, attribute_name)? else {
        return Ok(None);
    };
    let string = unsafe { core_foundation::string::CFString::wrap_under_create_rule(value.cast()) };
    Ok(Some(string.to_string()))
}

#[cfg(target_os = "macos")]
pub fn copy_bool_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<Option<bool>, String> {
    let Some(value) = copy_attribute(element, attribute_name)? else {
        return Ok(None);
    };
    let boolean = unsafe { CFBooleanGetValue(value.cast()) };
    unsafe {
        CFRelease(value.cast());
    }
    Ok(Some(boolean))
}

#[cfg(target_os = "macos")]
pub fn copy_window_frame(element: AXUIElementRef) -> Result<WindowFrame, String> {
    let position_value = copy_attribute(element, "AXPosition")?
        .ok_or_else(|| "AXPosition missing".to_string())?;
    let size_value =
        copy_attribute(element, "AXSize")?.ok_or_else(|| "AXSize missing".to_string())?;

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
pub fn set_bool_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
    value: bool,
) -> Result<(), String> {
    let attribute = CFString::new(attribute_name);
    let boolean = if value {
        unsafe { core_foundation::boolean::kCFBooleanTrue }
    } else {
        unsafe { core_foundation::boolean::kCFBooleanFalse }
    };
    let error = unsafe {
        AXUIElementSetAttributeValue(element, attribute.as_concrete_TypeRef(), boolean.cast())
    };
    if error == AX_ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("failed to set {attribute_name}: error {error}"))
    }
}

#[cfg(target_os = "macos")]
pub fn perform_action(element: AXUIElementRef, action_name: &str) -> Result<(), String> {
    let action = CFString::new(action_name);
    let error = unsafe { AXUIElementPerformAction(element, action.as_concrete_TypeRef()) };
    if error == AX_ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("failed to perform {action_name}: error {error}"))
    }
}

#[cfg(target_os = "macos")]
fn copy_attribute(element: AXUIElementRef, attribute_name: &str) -> Result<Option<CFTypeRef>, String> {
    let attribute = CFString::new(attribute_name);
    let mut value = std::ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };
    if error == AX_ERROR_SUCCESS {
        Ok((!value.is_null()).then_some(value))
    } else {
        Ok(None)
    }
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
