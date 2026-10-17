use crate::data::battery_impl::read_battery;
use crate::data::format::sanitize_text;
use crate::data::rate::rate_per_sec;
use crate::data::snapshot::*;
use crate::data::MetricsProvider;
use crate::platform::{FanStats, GpuStats};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System, Users};

/// Refresh slow-changing hardware (users, temps, GPU, fans, battery) and
/// per-process CPU-time syscalls at most this often. The sampler may run
/// anywhere from 50 ms to 5 s apart, so this is time based.
const SLOW_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

pub struct SysinfoProvider {
    system: System,
    users: Users,
    last: Instant,
    last_slow: Option<Instant>,
    networks: Networks,
    disks: Disks,
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    prev_disk_read: HashMap<String, u64>,
    prev_disk_write: HashMap<String, u64>,
    gpu: Box<dyn GpuStats + Send>,
    fan: Box<dyn FanStats + Send>,
    cached_components: Vec<ComponentInfo>,
    cached_gpu: Option<GpuInfo>,
    cached_fans: Vec<FanInfo>,
    cached_battery: Option<BatteryInfo>,
    proc_stats_cache: HashMap<u32, (Option<u64>, Option<usize>)>,
}

impl SysinfoProvider {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            users: Users::new_with_refreshed_list(),
            last: Instant::now(),
            last_slow: None,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            prev_net_rx: Default::default(),
            prev_net_tx: Default::default(),
            prev_disk_read: Default::default(),
            prev_disk_write: Default::default(),
            gpu: crate::platform::build_gpu(),
            fan: crate::platform::build_fan(),
            cached_components: Vec::new(),
            cached_gpu: None,
            cached_fans: Vec::new(),
            cached_battery: None,
            proc_stats_cache: HashMap::new(),
        }
    }
}

/// Compute the per-second rate of a cumulative counter, storing `current`
/// under `key` for the next call. Returns 0.0 on the first sample (no
/// previous value) and when the counter resets (see `rate_per_sec`).
fn diff_value(
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

impl MetricsProvider for SysinfoProvider {
    fn kill(&mut self, pid: u32, signal: crate::platform::signal::SignalChoice) {
        let _ = crate::platform::signal::send_signal(&self.system, pid, signal);
    }

    fn sample(&mut self) -> Snapshot {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        self.last = now;
        let slow = self
            .last_slow
            .is_none_or(|t| now.duration_since(t) >= SLOW_INTERVAL);
        if slow {
            self.last_slow = Some(now);
        }

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::nothing()
                .with_memory()
                .with_cpu()
                .with_disk_usage()
                .with_exe(sysinfo::UpdateKind::OnlyIfNotSet)
                .with_tasks()
                .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet),
        );

        self.networks.refresh(true);
        self.disks.refresh(true);

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
        let network: Vec<NetRate> = self
            .networks
            .iter()
            .filter(|(name, _)| is_relevant_interface(name))
            .map(|(name, data)| {
                let rx = diff_value(&mut self.prev_net_rx, name, data.total_received(), elapsed);
                let tx = diff_value(
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

        let disks_usage: Vec<DiskUsage> = self
            .disks
            .iter()
            .map(|d| {
                let key = d.mount_point().to_string_lossy().to_string();
                let usage = d.usage();
                let r = diff_value(
                    &mut self.prev_disk_read,
                    &key,
                    usage.total_read_bytes,
                    elapsed,
                );
                let w = diff_value(
                    &mut self.prev_disk_write,
                    &key,
                    usage.total_written_bytes,
                    elapsed,
                );
                DiskUsage {
                    mount_point: key,
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
                if slow {
                    match crate::platform::cpu_time::process_stats(pid.as_u32()) {
                        Some(s) => {
                            let stats =
                                (Some(s.cpu_time_secs), (s.threads > 0).then_some(s.threads));
                            self.proc_stats_cache.insert(pid.as_u32(), stats);
                            stats
                        }
                        None => {
                            self.proc_stats_cache.remove(&pid.as_u32());
                            (None, None)
                        }
                    }
                } else {
                    self.proc_stats_cache
                        .get(&pid.as_u32())
                        .copied()
                        .unwrap_or((None, None))
                }
            };
            #[cfg(not(target_os = "macos"))]
            let (cpu_time, threads) = (
                Some(p.accumulated_cpu_time() / 1000),
                p.tasks().map(|t| t.len()),
            );
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: sanitize_text(&p.name().to_string_lossy()),
                cmd: sanitize_text(
                    &p.cmd()
                        .iter()
                        .map(|a| a.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
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

            let ifaces: std::collections::HashSet<&str> =
                self.networks.keys().map(|name| name.as_str()).collect();
            self.prev_net_rx.retain(|k, _| ifaces.contains(k.as_str()));
            self.prev_net_tx.retain(|k, _| ifaces.contains(k.as_str()));

            let mounts: std::collections::HashSet<&str> =
                disks_usage.iter().map(|d| d.mount_point.as_str()).collect();
            self.prev_disk_read
                .retain(|k, _| mounts.contains(k.as_str()));
            self.prev_disk_write
                .retain(|k, _| mounts.contains(k.as_str()));
        }

        Snapshot {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn diff_first_sample_is_zero() {
        let mut prev: HashMap<String, u64> = HashMap::new();
        let rate = diff_value(&mut prev, "eth0", 1000, Duration::from_secs(1));
        assert_eq!(rate, 0.0);
        assert_eq!(prev["eth0"], 1000);
    }

    #[test]
    fn diff_computes_rate() {
        let mut prev: HashMap<String, u64> = HashMap::new();
        diff_value(&mut prev, "eth0", 1000, Duration::from_secs(1));
        let rate = diff_value(&mut prev, "eth0", 3000, Duration::from_secs(2));
        assert_eq!(rate, 1000.0);
    }

    #[test]
    fn diff_counter_reset_is_zero() {
        let mut prev: HashMap<String, u64> = HashMap::new();
        diff_value(&mut prev, "eth0", 3000, Duration::from_secs(1));
        let rate = diff_value(&mut prev, "eth0", 100, Duration::from_secs(1));
        assert_eq!(rate, 0.0);
    }
}
