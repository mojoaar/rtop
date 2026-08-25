use crate::data::snapshot::GpuInfo;
use crate::platform::GpuStats;
use core_foundation::base::{CFAllocatorRef, CFRelease, CFTypeRef};
use core_foundation::dictionary::{
    CFDictionaryGetValueIfPresent, CFDictionaryRef, CFMutableDictionaryRef,
};
use core_foundation::number::{CFNumberGetValue, CFNumberRef, kCFNumberDoubleType};
use core_foundation::string::{CFStringCreateWithCString, CFStringRef, kCFStringEncodingUTF8};
use std::os::raw::{c_char, c_void};

#[allow(non_camel_case_types)]
type io_object_t = u32;
#[allow(non_camel_case_types)]
type io_registry_entry_t = io_object_t;
#[allow(non_camel_case_types)]
type io_iterator_t = io_object_t;
#[allow(non_camel_case_types)]
type kern_return_t = i32;

const KERN_SUCCESS: kern_return_t = 0;
const IO_OBJECT_NULL: io_object_t = 0;

// kIOMainPortDefault is a preprocessor macro equal to 0 on modern SDKs, so we
// pass the literal `0` as the main port rather than linking against a symbol.
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        main_port: u32,
        matching: CFDictionaryRef,
        existing: *mut io_iterator_t,
    ) -> kern_return_t;
    fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;
    fn IORegistryEntryCreateCFProperty(
        entry: io_registry_entry_t,
        key: CFStringRef,
        allocator: CFAllocatorRef,
        options: u32,
    ) -> CFTypeRef;
    fn IOObjectRelease(object: io_object_t) -> kern_return_t;
}

pub struct MacGpu;

/// Look up a numeric value inside a CFDictionary by string key.
///
/// # Safety
/// `dict` must be a valid, non-null `CFDictionaryRef`.
unsafe fn dict_number(dict: CFDictionaryRef, key: &[u8]) -> Option<f64> {
    let key_ref = CFStringCreateWithCString(
        std::ptr::null(),
        key.as_ptr() as *const c_char,
        kCFStringEncodingUTF8,
    );
    if key_ref.is_null() {
        return None;
    }
    let mut value: *const c_void = std::ptr::null();
    let found = CFDictionaryGetValueIfPresent(dict, key_ref as *const c_void, &mut value);
    CFRelease(key_ref as CFTypeRef);
    if found == 0 || value.is_null() {
        return None;
    }
    let mut out: f64 = 0.0;
    if !CFNumberGetValue(
        value as CFNumberRef,
        kCFNumberDoubleType,
        &mut out as *mut f64 as *mut c_void,
    ) {
        return None;
    }
    Some(out)
}

/// Fetch the "PerformanceStatistics" CFDictionary for the first matching
/// IOKit service. The returned dictionary is owned by the caller (Create rule).
///
/// # Safety
/// Internal FFI helper; callers must `CFRelease` the returned dictionary.
unsafe fn service_property(class_name: &[u8]) -> Option<CFDictionaryRef> {
    let matching = IOServiceMatching(class_name.as_ptr() as *const c_char);
    if matching.is_null() {
        return None;
    }
    let mut iterator: io_iterator_t = 0;
    if IOServiceGetMatchingServices(0, matching, &mut iterator) != KERN_SUCCESS {
        return None;
    }
    let service = IOIteratorNext(iterator);
    IOObjectRelease(iterator);
    if service == IO_OBJECT_NULL {
        return None;
    }

    let key = CFStringCreateWithCString(
        std::ptr::null(),
        b"PerformanceStatistics\0".as_ptr() as *const c_char,
        kCFStringEncodingUTF8,
    );
    if key.is_null() {
        IOObjectRelease(service);
        return None;
    }
    let raw = IORegistryEntryCreateCFProperty(service, key, std::ptr::null(), 0);
    CFRelease(key as CFTypeRef);
    IOObjectRelease(service);
    if raw.is_null() {
        return None;
    }
    Some(raw as CFDictionaryRef)
}

impl GpuStats for MacGpu {
    fn read(&self) -> Option<GpuInfo> {
        unsafe {
            // Apple Silicon exposes the GPU as AGXAccelerator; Intel/AMD use IOAccelerator.
            let props = service_property(b"AGXAccelerator\0")
                .or_else(|| service_property(b"IOAccelerator\0"))?;

            let parsed = (|| {
                let util = dict_number(props, b"Device Utilization %\0")?;
                let mem_used = dict_number(props, b"In use system memory\0")?;
                let mem_total = dict_number(props, b"Alloc system memory\0")?;
                Some((util, mem_used, mem_total))
            })();
            CFRelease(props as CFTypeRef);

            let (util, mem_used, mem_total) = parsed?;
            Some(GpuInfo {
                name: "GPU".into(),
                utilization_percent: util as f32,
                memory_used_bytes: mem_used as u64,
                memory_total_bytes: mem_total as u64,
            })
        }
    }
}
