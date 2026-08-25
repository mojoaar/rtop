pub mod battery_impl;
pub mod format;
pub mod history;
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
}

pub fn build_provider() -> Box<dyn MetricsProvider + Send> {
    Box::new(sysinfo_impl::SysinfoProvider::new())
}

use std::sync::mpsc::Receiver;
use std::time::Duration;

pub fn spawn_sampler(
    mut provider: Box<dyn MetricsProvider + Send>,
    interval: Duration,
    cmd_rx: Receiver<Command>,
) -> Receiver<Snapshot> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    Command::Kill(pid) => provider.kill(pid),
                }
            }
            if tx.send(provider.sample()).is_err() {
                break;
            }
            std::thread::sleep(interval);
        }
    });
    rx
}
