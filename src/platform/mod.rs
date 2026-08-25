use crate::data::snapshot::{FanInfo, GpuInfo};

pub trait GpuStats {
    fn read(&self) -> Option<GpuInfo>;
}

pub trait FanStats {
    fn read(&self) -> Vec<FanInfo>;
}

#[cfg(not(target_os = "macos"))]
pub struct NullGpu;
#[cfg(not(target_os = "macos"))]
impl GpuStats for NullGpu {
    fn read(&self) -> Option<GpuInfo> {
        None
    }
}

#[cfg(not(target_os = "macos"))]
pub struct NullFan;
#[cfg(not(target_os = "macos"))]
impl FanStats for NullFan {
    fn read(&self) -> Vec<FanInfo> {
        vec![]
    }
}

#[cfg(target_os = "macos")]
pub mod gpu_macos;
#[cfg(target_os = "macos")]
pub mod fan_macos;

pub mod signal;

#[cfg(target_os = "macos")]
pub fn build_gpu() -> Box<dyn GpuStats + Send> {
    Box::new(gpu_macos::MacGpu)
}
#[cfg(not(target_os = "macos"))]
pub fn build_gpu() -> Box<dyn GpuStats + Send> {
    Box::new(NullGpu)
}

#[cfg(target_os = "macos")]
pub fn build_fan() -> Box<dyn FanStats + Send> {
    Box::new(fan_macos::MacFan)
}
#[cfg(not(target_os = "macos"))]
pub fn build_fan() -> Box<dyn FanStats + Send> {
    Box::new(NullFan)
}
