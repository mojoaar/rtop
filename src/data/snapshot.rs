#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub uptime: u64,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub network: Vec<NetRate>,
    pub net_total_received: u64,
    pub net_total_transmitted: u64,
    pub disks: Vec<DiskUsage>,
    pub processes: Vec<ProcessInfo>,
    pub components: Vec<ComponentInfo>,
    pub battery: Option<BatteryInfo>,
    pub gpu: Option<GpuInfo>,
    pub fans: Vec<FanInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct CpuSnapshot {
    pub global_usage: f32,
    pub per_core: Vec<f32>,
    pub load_avg: Option<[f64; 3]>,
    pub brand: String,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MemorySnapshot {
    pub total: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone, Default)]
pub struct NetRate {
    pub name: String,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DiskUsage {
    pub mount_point: String,
    pub total: u64,
    pub available: u64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub user: String,
    /// Total CPU time; `None` when the OS does not let us read it.
    pub cpu_time: Option<u64>,
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ComponentInfo {
    pub label: String,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub percentage: f32,
    pub state: String,
    pub time_to_full_secs: Option<u64>,
    pub time_to_empty_secs: Option<u64>,
    pub cycle_count: Option<u32>,
    pub health_percent: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub utilization_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FanInfo {
    pub label: String,
    pub rpm: u32,
}

/// Loopback, tunnels, and virtual bridges only add noise; VPN traffic is
/// already counted on the physical link.
pub fn is_relevant_interface(name: &str) -> bool {
    let n = name.to_lowercase();
    ![
        "lo", "gif", "stf", "awdl", "llw", "anpi", "bridge", "br", "utun", "docker", "veth",
        "virbr", "tun", "tap", "wg", "vmnet", "vnic",
    ]
    .iter()
    .any(|p| n.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::is_relevant_interface;

    #[test]
    fn filters_virtual_and_loopback_interfaces() {
        assert!(!is_relevant_interface("lo0"));
        assert!(!is_relevant_interface("lo"));
        assert!(!is_relevant_interface("utun3"));
        assert!(!is_relevant_interface("awdl0"));
        assert!(!is_relevant_interface("llw0"));
        assert!(!is_relevant_interface("anpi1"));
        assert!(!is_relevant_interface("gif0"));
        assert!(!is_relevant_interface("stf0"));
        assert!(!is_relevant_interface("bridge0"));
        assert!(!is_relevant_interface("docker0"));
        assert!(!is_relevant_interface("veth1"));
        assert!(!is_relevant_interface("br-1234"));
        assert!(!is_relevant_interface("tun0"));
        assert!(!is_relevant_interface("tap0"));
        assert!(!is_relevant_interface("wg0"));
        assert!(!is_relevant_interface("vmnet3"));
        assert!(!is_relevant_interface("virbr0"));
    }

    #[test]
    fn keeps_physical_interfaces() {
        assert!(is_relevant_interface("en0"));
        assert!(is_relevant_interface("en1"));
        assert!(is_relevant_interface("eth0"));
        assert!(is_relevant_interface("wlan0"));
        assert!(is_relevant_interface("enp3s0"));
        assert!(is_relevant_interface("wlp2s0"));
    }
}
