pub mod format;
pub mod history;
pub mod rate;
pub mod snapshot;

use snapshot::Snapshot;

pub trait MetricsProvider {
    fn sample(&mut self) -> Snapshot;
}

use std::sync::mpsc::Receiver;
use std::time::Duration;

pub fn spawn_sampler(
    mut provider: impl MetricsProvider + Send + 'static,
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
