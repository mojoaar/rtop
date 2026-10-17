pub mod battery_impl;
pub mod format;
pub mod history;
pub mod ip;
pub mod rate;
pub mod snapshot;
pub mod sysinfo_impl;

use crate::platform::signal::SignalChoice;
use snapshot::Snapshot;

pub trait MetricsProvider {
    fn sample(&mut self) -> Snapshot;
    fn kill(&mut self, _pid: u32, _signal: SignalChoice) {}
}

pub enum Command {
    Kill { pid: u32, signal: SignalChoice },
    SetInterval(u64),
    SetPaused(bool),
}

pub fn build_provider() -> Box<dyn MetricsProvider + Send> {
    Box::new(sysinfo_impl::SysinfoProvider::new())
}

use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, TrySendError};
use std::time::Duration;

pub fn spawn_sampler(
    mut provider: Box<dyn MetricsProvider + Send>,
    mut interval: Duration,
    cmd_rx: Receiver<Command>,
) -> Receiver<Snapshot> {
    let (tx, rx) = sync_channel(1);
    std::thread::spawn(move || {
        let mut paused = false;
        loop {
            match cmd_rx.recv_timeout(interval) {
                Ok(Command::Kill { pid, signal }) => {
                    provider.kill(pid, signal);
                    continue;
                }
                Ok(Command::SetInterval(ms)) => {
                    interval = Duration::from_millis(ms);
                    continue;
                }
                Ok(Command::SetPaused(pause)) => {
                    paused = pause;
                    if paused {
                        continue;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            if paused {
                continue;
            }
            // The UI samples at human speed; dropping a snapshot while it is
            // busy is cheaper than queueing them forever.
            match tx.try_send(provider.sample()) {
                Ok(()) | Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => return,
            }
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    struct Fake;

    impl MetricsProvider for Fake {
        fn sample(&mut self) -> Snapshot {
            Snapshot::default()
        }
    }

    #[test]
    fn paused_sampler_stops_producing() {
        let (cmd_tx, cmd_rx) = channel();
        let rx = spawn_sampler(Box::new(Fake), Duration::from_millis(20), cmd_rx);
        cmd_tx.send(Command::SetPaused(true)).unwrap();
        std::thread::sleep(Duration::from_millis(80));
        while rx.try_recv().is_ok() {}
        std::thread::sleep(Duration::from_millis(80));
        assert!(rx.try_recv().is_err());

        cmd_tx.send(Command::SetPaused(false)).unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_ok());
    }

    #[test]
    fn full_queue_keeps_one_sample() {
        let (_cmd_tx, cmd_rx) = channel();
        let rx = spawn_sampler(Box::new(Fake), Duration::from_millis(10), cmd_rx);
        std::thread::sleep(Duration::from_millis(100));
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_err());
    }
}
