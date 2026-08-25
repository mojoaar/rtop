pub mod battery_impl;
pub mod format;
pub mod history;
pub mod rate;
pub mod snapshot;
pub mod sysinfo_impl;

use snapshot::Snapshot;

pub trait MetricsProvider {
    fn sample(&mut self) -> Snapshot;
}

pub fn build_provider() -> Box<dyn MetricsProvider + Send> {
    Box::new(sysinfo_impl::SysinfoProvider::new())
}

use std::sync::mpsc::Receiver;
use std::time::Duration;

pub fn spawn_sampler(
    mut provider: Box<dyn MetricsProvider + Send>,
    interval: Duration,
) -> Receiver<Snapshot> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        loop {
            if tx.send(provider.sample()).is_err() {
                break;
            }
            std::thread::sleep(interval);
        }
    });
    rx
}
