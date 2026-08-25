use crate::data::battery_impl::read_battery;
use crate::data::rate::rate_per_sec;
use crate::data::snapshot::*;
use crate::data::MetricsProvider;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System};

pub struct SysinfoProvider {
    system: System,
    last: Instant,
    prev_net_rx: std::collections::HashMap<String, u64>,
    prev_net_tx: std::collections::HashMap<String, u64>,
    prev_disk_read: std::collections::HashMap<String, u64>,
    prev_disk_write: std::collections::HashMap<String, u64>,
}

impl SysinfoProvider {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            last: Instant::now(),
            prev_net_rx: Default::default(),
            prev_net_tx: Default::default(),
            prev_disk_read: Default::default(),
            prev_disk_write: Default::default(),
        }
    }

    fn diff_map(
        prev: &mut std::collections::HashMap<String, u64>,
        key: &str,
        current: u64,
        elapsed: std::time::Duration,
    ) -> f64 {
        let rate = prev.get(key).map_or(0.0, |p| rate_per_sec(current, *p, elapsed));
        prev.insert(key.to_string(), current);
        rate
    }
}

impl MetricsProvider for SysinfoProvider {
    fn sample(&mut self) -> Snapshot {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        self.last = now;

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        let load_avg = sysinfo::System::load_average();
        let cpu = CpuSnapshot {
            global_usage: self.system.global_cpu_usage(),
            per_core: self.system.cpus().iter().map(|c| c.cpu_usage()).collect(),
            load_avg: Some([load_avg.one, load_avg.five, load_avg.fifteen]),
        };

        let memory = MemorySnapshot {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        };

        let network: Vec<NetRate> = networks
            .iter()
            .map(|(name, data)| {
                let rx = Self::diff_map(&mut self.prev_net_rx, name, data.received(), elapsed);
                let tx = Self::diff_map(&mut self.prev_net_tx, name, data.transmitted(), elapsed);
                NetRate {
                    name: name.clone(),
                    rx_bytes_per_sec: rx,
                    tx_bytes_per_sec: tx,
                }
            })
            .collect();

        let disks_usage: Vec<DiskUsage> = disks
            .iter()
            .map(|d| {
                let key = d.mount_point().to_string_lossy().to_string();
                let usage = d.usage();
                let r = Self::diff_map(&mut self.prev_disk_read, &key, usage.total_read_bytes, elapsed);
                let w = Self::diff_map(&mut self.prev_disk_write, &key, usage.total_written_bytes, elapsed);
                DiskUsage {
                    mount_point: key,
                    name: d.name().to_string_lossy().to_string(),
                    total: d.total_space(),
                    available: d.available_space(),
                    read_bytes_per_sec: r,
                    write_bytes_per_sec: w,
                }
            })
            .collect();

        let processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage(),
                memory_bytes: p.memory(),
                status: format!("{:?}", p.status()),
            })
            .collect();

        let components_vec: Vec<ComponentInfo> = components
            .iter()
            .map(|c| ComponentInfo {
                label: c.label().to_string(),
                temperature_c: c.temperature(),
            })
            .collect();

        Snapshot {
            timestamp: Some(now),
            cpu,
            memory,
            network,
            disks: disks_usage,
            processes,
            components: components_vec,
            battery: read_battery(),
            gpu: None,
            fans: vec![],
        }
    }
}
