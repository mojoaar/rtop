use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    #[allow(dead_code)]
    pub timestamp: Option<Instant>,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub network: Vec<NetRate>,
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
    pub name: String,
    pub total: u64,
    pub available: u64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub user: String,
    pub cpu_time: u64,
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
