use sysinfo::{Pid, Signal, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalChoice {
    Term,
    Kill,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_sys_maps_signals() {
        assert_eq!(SignalChoice::Term.to_sys(), Signal::Term);
        assert_eq!(SignalChoice::Kill.to_sys(), Signal::Kill);
        assert_eq!(SignalChoice::Interrupt.to_sys(), Signal::Interrupt);
    }
}
