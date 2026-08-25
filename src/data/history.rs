use super::snapshot::Snapshot;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    head: usize,
    cap: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        Self { buf: Vec::with_capacity(cap), head: 0, cap: cap.max(1) }
    }

    pub fn push(&mut self, value: T) {
        if self.buf.len() < self.cap {
            self.buf.push(value);
        } else {
            self.buf[self.head] = value;
            self.head = (self.head + 1) % self.cap;
        }
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let n = self.buf.len();
        (0..n).map(move |i| &self.buf[(self.head + i) % n])
    }

    pub fn latest(&self) -> Option<&T> {
        if self.buf.is_empty() { return None; }
        let i = (self.head + self.buf.len() - 1) % self.buf.len();
        Some(&self.buf[i])
    }
}

#[derive(Debug, Clone)]
pub struct History {
    cpu: RingBuffer<f32>,
    gpu: RingBuffer<f32>,
    net_rx: RingBuffer<f64>,
    net_tx: RingBuffer<f64>,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            cpu: RingBuffer::new(capacity),
            gpu: RingBuffer::new(capacity),
            net_rx: RingBuffer::new(capacity),
            net_tx: RingBuffer::new(capacity),
        }
    }

    pub fn record(&mut self, snap: &Snapshot) {
        self.cpu.push(snap.cpu.global_usage);
        self.gpu
            .push(snap.gpu.as_ref().map(|g| g.utilization_percent).unwrap_or(0.0));
        let (rx, tx) = snap
            .network
            .iter()
            .fold((0.0f64, 0.0f64), |(r, t), n| (r + n.rx_bytes_per_sec, t + n.tx_bytes_per_sec));
        self.net_rx.push(rx);
        self.net_tx.push(tx);
    }

    pub fn cpu_series(&self) -> Vec<u64> {
        self.cpu.iter().map(|v| *v as u64).collect()
    }

    pub fn gpu_series(&self) -> Vec<u64> {
        self.gpu.iter().map(|v| *v as u64).collect()
    }

    pub fn net_rx_series(&self) -> Vec<u64> {
        self.net_rx.iter().map(|v| *v as u64).collect()
    }

    pub fn net_tx_series(&self) -> Vec<u64> {
        self.net_tx.iter().map(|v| *v as u64).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::snapshot::{CpuSnapshot, NetRate, Snapshot};

    #[test]
    fn wraps_and_keeps_oldest_to_newest_order() {
        let mut rb = RingBuffer::new(3);
        rb.push(1); rb.push(2); rb.push(3); rb.push(4);
        let v: Vec<i32> = rb.iter().copied().collect();
        assert_eq!(v, vec![2, 3, 4]);
        assert_eq!(*rb.latest().unwrap(), 4);
    }

    #[test]
    fn not_full_yields_all_in_order() {
        let mut rb = RingBuffer::new(5);
        rb.push(10); rb.push(20);
        let v: Vec<i32> = rb.iter().copied().collect();
        assert_eq!(v, vec![10, 20]);
        assert_eq!(*rb.latest().unwrap(), 20);
    }

    #[test]
    fn empty_has_no_latest() {
        let rb: RingBuffer<i32> = RingBuffer::new(3);
        assert!(rb.is_empty());
        assert!(rb.latest().is_none());
    }

    fn snap_with_cpu(usage: f32) -> Snapshot {
        let mut s = Snapshot::default();
        s.cpu = CpuSnapshot { global_usage: usage, ..Default::default() };
        s
    }

    #[test]
    fn records_cpu_series_in_order() {
        let mut h = History::new(3);
        h.record(&snap_with_cpu(10.0));
        h.record(&snap_with_cpu(20.0));
        h.record(&snap_with_cpu(30.0));
        assert_eq!(h.cpu_series(), vec![10, 20, 30]);
    }

    #[test]
    fn gpu_none_records_zero() {
        let mut h = History::new(3);
        h.record(&Snapshot::default());
        assert_eq!(h.gpu_series(), vec![0]);
    }

    #[test]
    fn sums_network_across_interfaces() {
        let mut s = Snapshot::default();
        s.network = vec![
            NetRate { name: "en0".into(), rx_bytes_per_sec: 100.0, tx_bytes_per_sec: 50.0 },
            NetRate { name: "en1".into(), rx_bytes_per_sec: 200.0, tx_bytes_per_sec: 150.0 },
        ];
        let mut h = History::new(3);
        h.record(&s);
        assert_eq!(h.net_rx_series(), vec![300]);
        assert_eq!(h.net_tx_series(), vec![200]);
    }

    #[test]
    fn wraps_at_capacity() {
        let mut h = History::new(2);
        h.record(&snap_with_cpu(10.0));
        h.record(&snap_with_cpu(20.0));
        h.record(&snap_with_cpu(30.0));
        assert_eq!(h.cpu_series(), vec![20, 30]);
    }
}
