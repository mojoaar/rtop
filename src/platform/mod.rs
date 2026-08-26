use crate::data::snapshot::{FanInfo, GpuInfo};

pub trait GpuStats {
    fn read(&self) -> Option<GpuInfo>;
}

pub trait FanStats {
    fn read(&self) -> Vec<FanInfo>;
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct NullGpu;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl GpuStats for NullGpu {
    fn read(&self) -> Option<GpuInfo> {
        None
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct NullFan;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl FanStats for NullFan {
    fn read(&self) -> Vec<FanInfo> {
        vec![]
    }
}

#[cfg(target_os = "macos")]
pub mod cpu_time;
#[cfg(target_os = "linux")]
pub mod fan_linux;
#[cfg(target_os = "macos")]
pub mod fan_macos;
#[cfg(target_os = "linux")]
pub mod gpu_linux;
#[cfg(target_os = "macos")]
pub mod gpu_macos;

pub mod signal;

#[cfg(target_os = "macos")]
pub fn build_gpu() -> Box<dyn GpuStats + Send> {
    Box::new(gpu_macos::MacGpu)
}
#[cfg(target_os = "linux")]
pub fn build_gpu() -> Box<dyn GpuStats + Send> {
    Box::new(gpu_linux::LinuxGpu)
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn build_gpu() -> Box<dyn GpuStats + Send> {
    Box::new(NullGpu)
}

#[cfg(target_os = "macos")]
pub fn build_fan() -> Box<dyn FanStats + Send> {
    Box::new(fan_macos::MacFan)
}
#[cfg(target_os = "linux")]
pub fn build_fan() -> Box<dyn FanStats + Send> {
    Box::new(fan_linux::LinuxFan)
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn build_fan() -> Box<dyn FanStats + Send> {
    Box::new(NullFan)
}
