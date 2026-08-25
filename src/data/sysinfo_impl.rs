use crate::data::battery_impl::read_battery;
use crate::data::rate::rate_per_sec;
use crate::data::snapshot::*;
use crate::data::MetricsProvider;
use crate::platform::{FanStats, GpuStats};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System, Users};

/// Refresh slow-changing hardware (users, temps, GPU, fans, battery) and
/// per-process CPU-time syscalls every `SLOW_EVERY` samples. At the default
/// 500 ms interval this is roughly every 2 seconds.
const SLOW_EVERY: u64 = 4;

pub struct SysinfoProvider {
    system: System,
    users: Users,
    last: Instant,
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    prev_disk_read: HashMap<String, u64>,
    prev_disk_write: HashMap<String, u64>,
    gpu: Box<dyn GpuStats + Send>,
    fan: Box<dyn FanStats + Send>,
    tick: u64,
    cached_components: Vec<ComponentInfo>,
    cached_gpu: Option<GpuInfo>,
    cached_fans: Vec<FanInfo>,
    cached_battery: Option<BatteryInfo>,
    proc_stats_cache: HashMap<u32, (u64, Option<usize>)>,
}

impl SysinfoProvider {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            users: Users::new_with_refreshed_list(),
            last: Instant::now(),
            prev_net_rx: Default::default(),
            prev_net_tx: Default::default(),
            prev_disk_read: Default::default(),
            prev_disk_write: Default::default(),
            gpu: crate::platform::build_gpu(),
            fan: crate::platform::build_fan(),
            tick: 0,
            cached_components: Vec::new(),
            cached_gpu: None,
            cached_fans: Vec::new(),
            cached_battery: None,
            proc_stats_cache: HashMap::new(),
        }
    }

    fn diff_map(
        prev: &mut HashMap<String, u64>,
        key: &str,
        current: u64,
        elapsed: std::time::Duration,
    ) -> f64 {
        let rate = prev
            .get(key)
            .map_or(0.0, |p| rate_per_sec(current, *p, elapsed));
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
        let slow = self.tick % SLOW_EVERY == 0;
        self.tick = self.tick.wrapping_add(1);

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        if slow {
            self.users.refresh();
            self.cached_components = Components::new_with_refreshed_list()
                .iter()
                .map(|c| ComponentInfo {
                    label: c.label().to_string(),
                    temperature_c: c.temperature(),
                })
                .collect();
            self.cached_gpu = self.gpu.read();
            self.cached_fans = self.fan.read();
            self.cached_battery = read_battery();
        }

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
                let rx =
                    Self::diff_map(&mut self.prev_net_rx, name, data.total_received(), elapsed);
                let tx = Self::diff_map(
                    &mut self.prev_net_tx,
                    name,
                    data.total_transmitted(),
                    elapsed,
                );
                net_total_received = net_total_received.saturating_add(data.total_received());
                net_total_transmitted =
                    net_total_transmitted.saturating_add(data.total_transmitted());
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
                let r = Self::diff_map(
                    &mut self.prev_disk_read,
                    &key,
                    usage.total_read_bytes,
                    elapsed,
                );
                let w = Self::diff_map(
                    &mut self.prev_disk_write,
                    &key,
                    usage.total_written_bytes,
                    elapsed,
                );
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

        let process_list = self.system.processes();
        let mut processes: Vec<ProcessInfo> = Vec::with_capacity(process_list.len());
        for (pid, p) in process_list.iter() {
            let user = p
                .user_id()
                .and_then(|uid| self.users.get_user_by_id(uid))
                .map(|u| u.name().to_string())
                .unwrap_or_default();
            #[cfg(target_os = "macos")]
            let (cpu_time, threads) = {
                let fallback = || (p.run_time(), p.tasks().map(|t| t.len()));
                if slow {
                    let cached = crate::platform::cpu_time::process_stats(pid.as_u32())
                        .map(|s| (s.cpu_time_secs, (s.threads > 0).then_some(s.threads)));
                    if let Some(c) = cached {
                        self.proc_stats_cache.insert(pid.as_u32(), c);
                    }
                    cached.unwrap_or_else(fallback)
                } else {
                    self.proc_stats_cache
                        .get(&pid.as_u32())
                        .copied()
                        .unwrap_or_else(fallback)
                }
            };
            #[cfg(not(target_os = "macos"))]
            let (cpu_time, threads) = (p.run_time(), p.tasks().map(|t| t.len()));
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage(),
                memory_bytes: p.memory(),
                status: format!("{:?}", p.status()),
                user,
                cpu_time,
                threads,
            });
        }
        if slow {
            let alive: std::collections::HashSet<u32> = processes.iter().map(|p| p.pid).collect();
            self.proc_stats_cache.retain(|pid, _| alive.contains(pid));
        }

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
            components: self.cached_components.clone(),
            battery: self.cached_battery.clone(),
            gpu: self.cached_gpu.clone(),
            fans: self.cached_fans.clone(),
        }
    }
}
