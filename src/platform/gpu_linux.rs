use crate::data::snapshot::GpuInfo;
use crate::platform::GpuStats;
use std::fs;
use std::path::Path;

pub struct LinuxGpu;

fn read_u64(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Query an NVIDIA GPU through NVML. `Nvml::init()` dynamically loads
/// `libnvidia-ml.so.1`, so this fails gracefully on systems without it.
fn nvidia() -> Option<GpuInfo> {
    let nvml = nvml_wrapper::Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;
    let util = device.utilization_rates().ok()?.gpu;
    let mem = device.memory_info().ok()?;
    let name = device.name().unwrap_or_else(|_| "NVIDIA".to_string());
    Some(GpuInfo {
        name,
        utilization_percent: util as f32,
        memory_used_bytes: mem.used,
        memory_total_bytes: mem.total,
    })
}

/// Query an AMD or Intel GPU via the DRM sysfs interface. AMD exposes a busy
/// percentage; Intel exposes current/max frequency, from which we derive an
/// approximate utilization. Either path returns `None` gracefully if the
/// files are absent.
fn sysfs_drm() -> Option<GpuInfo> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(rest) = name.strip_prefix("card") else {
            continue;
        };
        if !rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }

        let device = entry.path().join("device");
        if !device.is_dir() {
            continue;
        }

        if let Some(pct) = read_u64(&device.join("gpu_busy_percent")) {
            return Some(GpuInfo {
                name: "AMD".to_string(),
                utilization_percent: pct as f32,
                memory_used_bytes: read_u64(&device.join("mem_info_vram_used")).unwrap_or(0),
                memory_total_bytes: read_u64(&device.join("mem_info_vram_total")).unwrap_or(0),
            });
        }

        if let (Some(cur), Some(max)) = (
            read_u64(&device.join("gt_cur_freq_mhz")),
            read_u64(&device.join("gt_max_freq_mhz")),
        ) {
            if max > 0 {
                let pct = (cur as f32 / max as f32 * 100.0).clamp(0.0, 100.0);
                return Some(GpuInfo {
                    name: "Intel".to_string(),
                    utilization_percent: pct,
                    memory_used_bytes: read_u64(&device.join("mem_info_vram_used")).unwrap_or(0),
                    memory_total_bytes: read_u64(&device.join("mem_info_vram_total")).unwrap_or(0),
                });
            }
        }
    }
    None
}

impl GpuStats for LinuxGpu {
    fn read(&self) -> Option<GpuInfo> {
        nvidia().or_else(sysfs_drm)
    }
}
