use crate::data::snapshot::GpuInfo;
use crate::platform::GpuStats;
use std::fs;
use std::path::Path;

pub struct LinuxGpu;

fn read_u64(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

const PCI_IDS_PATHS: &[&str] = &[
    "/usr/share/hwdata/pci.ids",
    "/usr/share/misc/pci.ids",
    "/usr/share/pci.ids",
    "/usr/share/pciutils/pci.ids",
    "/etc/pci.ids",
];

/// Resolve a `(vendor, device)` pair to a device name from `pci.ids` content.
/// Vendor lines are unindented, device lines carry a single leading tab, and
/// subsystem entries two tabs; comments start with `#`. Returns `None` when the
/// pair is absent or the database is missing.
fn parse_pci_ids(content: &str, vendor: u64, device: u64) -> Option<String> {
    let vendor_key = format!("{:04x}", vendor);
    let device_key = format!("{:04x}", device);
    let mut current_vendor = "";

    for raw in content.lines() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('\t') {
            if rest.starts_with('\t') {
                continue;
            }
            if current_vendor == vendor_key
                && rest.get(..4) == Some(device_key.as_str())
                && !rest[4..].trim().is_empty()
            {
                return Some(rest[4..].trim().to_string());
            }
        } else if let Some(space) = line.find(' ') {
            current_vendor = &line[..space];
        }
    }
    None
}

/// Look up a device name in the system `pci.ids` database, trying each
/// candidate location in order. Returns `None` if no database is readable or
/// the pair is unknown.
fn pci_device_name(vendor: u64, device: u64) -> Option<String> {
    for path in PCI_IDS_PATHS {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some(name) = parse_pci_ids(&content, vendor, device) {
                return Some(name);
            }
        }
    }
    None
}

/// Build a display name such as `AMD HawkPoint2`, falling back to the vendor
/// prefix alone when the model could not be resolved.
fn display_name(prefix: &str, resolved: Option<&str>) -> String {
    match resolved {
        Some(n) => format!("{prefix} {n}"),
        None => prefix.to_string(),
    }
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

        let vendor = read_u64(&device.join("vendor"));
        let device_id = read_u64(&device.join("device"));
        let resolved = match (vendor, device_id) {
            (Some(v), Some(d)) => pci_device_name(v, d),
            _ => None,
        };

        if let Some(pct) = read_u64(&device.join("gpu_busy_percent")) {
            return Some(GpuInfo {
                name: display_name("AMD", resolved.as_deref()),
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
                    name: display_name("Intel", resolved.as_deref()),
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# comment line
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t1114  Krackan [Radeon 840M / 860M Graphics]
\t1901  HawkPoint2
\t\t17aa 3988  Z50-75
10de  NVIDIA Corporation
\t2684  RTX 4090
";

    #[test]
    fn resolves_device_name() {
        assert_eq!(
            parse_pci_ids(SAMPLE, 0x1002, 0x1901),
            Some("HawkPoint2".to_string())
        );
    }

    #[test]
    fn preserves_marketing_name_in_brackets() {
        assert_eq!(
            parse_pci_ids(SAMPLE, 0x1002, 0x1114),
            Some("Krackan [Radeon 840M / 860M Graphics]".to_string())
        );
    }

    #[test]
    fn returns_none_for_unknown_device() {
        assert_eq!(parse_pci_ids(SAMPLE, 0x1002, 0x9999), None);
        assert_eq!(parse_pci_ids(SAMPLE, 0x1234, 0x1901), None);
    }

    #[test]
    fn skips_comments_and_subsystem_lines() {
        assert_eq!(
            parse_pci_ids(SAMPLE, 0x10de, 0x2684),
            Some("RTX 4090".to_string())
        );
    }

    #[test]
    fn display_name_composes_prefix_and_model() {
        assert_eq!(
            display_name("AMD", Some("HawkPoint2")),
            "AMD HawkPoint2".to_string()
        );
        assert_eq!(display_name("AMD", None), "AMD".to_string());
        assert_eq!(
            display_name("Intel", Some("Raptor Lake")),
            "Intel Raptor Lake"
        );
    }
}
