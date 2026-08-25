use crate::data::snapshot::{FanInfo, GpuInfo};

pub trait GpuStats {
    fn read(&self) -> Option<GpuInfo>;
}

pub trait FanStats {
    fn read(&self) -> Vec<FanInfo>;
}

pub struct NullGpu;
impl GpuStats for NullGpu {
    fn read(&self) -> Option<GpuInfo> {
        None
    }
}

pub struct NullFan;
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
