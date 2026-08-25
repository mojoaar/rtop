use sysinfo::{Pid, Signal, System};

#[derive(Debug, Clone, Copy)]
pub enum SignalChoice {
    Term,
    #[allow(dead_code)]
    Kill,
    #[allow(dead_code)]
    Interrupt,
}

impl SignalChoice {
    pub fn to_sys(self) -> Signal {
        match self {
            SignalChoice::Term => Signal::Term,
            SignalChoice::Kill => Signal::Kill,
            SignalChoice::Interrupt => Signal::Interrupt,
        }
    }
}

pub fn send_signal(system: &System, pid: u32, signal: SignalChoice) -> bool {
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        #[cfg(unix)]
        {
            return process.kill_with(signal.to_sys()).unwrap_or(false);
        }
        #[cfg(not(unix))]
        {
            let _ = signal;
            return process.kill();
        }
    }
    false
}
