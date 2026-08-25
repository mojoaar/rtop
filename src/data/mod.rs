pub mod battery_impl;
pub mod format;
pub mod history;
pub mod ip;
pub mod rate;
pub mod snapshot;
pub mod sysinfo_impl;

use snapshot::Snapshot;

pub trait MetricsProvider {
    fn sample(&mut self) -> Snapshot;
    fn kill(&mut self, _pid: u32) {}
}

pub enum Command {
    Kill(u32),
    SetInterval(u64),
}

pub fn build_provider() -> Box<dyn MetricsProvider + Send> {
    Box::new(sysinfo_impl::SysinfoProvider::new())
}

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

pub fn spawn_sampler(
    mut provider: Box<dyn MetricsProvider + Send>,
    mut interval: Duration,
    cmd_rx: Receiver<Command>,
) -> Receiver<Snapshot> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        loop {
            match cmd_rx.recv_timeout(interval) {
                Ok(Command::Kill(pid)) => provider.kill(pid),
                Ok(Command::SetInterval(ms)) => interval = Duration::from_millis(ms),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if tx.send(provider.sample()).is_err() {
            break;
        }
    });
    rx
}
