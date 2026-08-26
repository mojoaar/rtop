use crate::data::snapshot::FanInfo;
use crate::platform::FanStats;
use std::fs;

pub struct LinuxFan;

/// Read fan speeds from the hwmon sysfs interface. `fan*_input` is the fan
/// speed in RPM; `fan*_label` is an optional human-readable name. Missing
/// sensors simply yield an empty list.
impl FanStats for LinuxFan {
    fn read(&self) -> Vec<FanInfo> {
        let mut fans = Vec::new();
        let entries = match fs::read_dir("/sys/class/hwmon") {
            Ok(entries) => entries,
            Err(_) => return fans,
        };

        for entry in entries.flatten() {
            let hwmon = entry.path();
            for i in 1..=10 {
                let input_path = hwmon.join(format!("fan{}_input", i));
                let rpm = match fs::read_to_string(&input_path) {
                    Ok(s) => match s.trim().parse::<u32>() {
                        Ok(v) if v > 0 => v,
                        _ => continue,
                    },
                    Err(_) => continue,
                };
                let label = fs::read_to_string(hwmon.join(format!("fan{}_label", i)))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| format!("Fan {}", i));
                fans.push(FanInfo { label, rpm });
            }
        }

        fans
    }
}
