#[cfg(target_os = "macos")]
use std::ffi::c_void;

#[cfg(target_os = "macos")]
use core_foundation::base::{CFRelease, CFRetain, CFTypeRef, TCFType};
#[cfg(target_os = "macos")]
use core_foundation::string::{CFString, CFStringRef};

#[cfg(target_os = "macos")]
type AXError = i32;
#[cfg(target_os = "macos")]
type AXUIElementRef = *const c_void;
#[cfg(target_os = "macos")]
type CFArrayRef = *const c_void;

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct AxWindowRef(AXUIElementRef);

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
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
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
