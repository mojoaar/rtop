use crate::data::battery_impl::read_battery;
use crate::data::rate::rate_per_sec;
use crate::data::snapshot::*;
use crate::data::MetricsProvider;
use crate::platform::{FanStats, GpuStats};
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System, Users};

pub struct SysinfoProvider {
    system: System,
    last: Instant,
    prev_net_rx: std::collections::HashMap<String, u64>,
    prev_net_tx: std::collections::HashMap<String, u64>,
    prev_disk_read: std::collections::HashMap<String, u64>,
    prev_disk_write: std::collections::HashMap<String, u64>,
    gpu: Box<dyn GpuStats + Send>,
    fan: Box<dyn FanStats + Send>,
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
            gpu: crate::platform::build_gpu(),
            fan: crate::platform::build_fan(),
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
    fn kill(&mut self, pid: u32) {
        let _ = crate::platform::signal::send_signal(
            &self.system,
            pid,
            crate::platform::signal::SignalChoice::Term,
        );
    }

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
        let cpus = self.system.cpus();
        let (brand, freq) = cpus
            .first()
            .map(|c| (c.brand().to_string(), c.frequency()))
            .unwrap_or_default();
        let cpu = CpuSnapshot {
            global_usage: self.system.global_cpu_usage(),
            per_core: cpus.iter().map(|c| c.cpu_usage()).collect(),
            load_avg: Some([load_avg.one, load_avg.five, load_avg.fifteen]),
            brand,
            frequency_mhz: freq,
        };

        let memory = MemorySnapshot {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        };

        let mut net_total_received: u64 = 0;
        let mut net_total_transmitted: u64 = 0;
        let network: Vec<NetRate> = networks
            .iter()
            .map(|(name, data)| {
                let rx = Self::diff_map(&mut self.prev_net_rx, name, data.total_received(), elapsed);
                let tx = Self::diff_map(&mut self.prev_net_tx, name, data.total_transmitted(), elapsed);
                net_total_received = net_total_received.saturating_add(data.total_received());
                net_total_transmitted = net_total_transmitted.saturating_add(data.total_transmitted());
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

        let users = Users::new_with_refreshed_list();
        let processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| {
                let user = p
                    .user_id()
                    .and_then(|uid| users.get_user_by_id(uid))
                    .map(|u| u.name().to_string())
                    .unwrap_or_default();
                #[cfg(target_os = "macos")]
                let (cpu_time, threads) =
                    crate::platform::cpu_time::process_stats(pid.as_u32())
                        .map(|s| (s.cpu_time_secs, (s.threads > 0).then_some(s.threads)))
                        .unwrap_or_else(|| (p.run_time(), p.tasks().map(|t| t.len())));
                #[cfg(not(target_os = "macos"))]
                let (cpu_time, threads) = (p.run_time(), p.tasks().map(|t| t.len()));
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().to_string(),
                    cpu_usage: p.cpu_usage(),
                    memory_bytes: p.memory(),
                    status: format!("{:?}", p.status()),
                    user,
                    cpu_time,
                    threads,
                }
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
            uptime: sysinfo::System::uptime(),
            cpu,
            memory,
            network,
            net_total_received,
            net_total_transmitted,
            disks: disks_usage,
            processes,
            components: components_vec,
            battery: read_battery(),
            gpu: self.gpu.read(),
            fans: self.fan.read(),
        }
    }
}
