use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct IpState {
    pub private: Option<String>,
    pub wan: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IpConfig {
    pub enabled: bool,
    pub url: String,
}

pub enum IpCmd {
    Update(IpConfig),
}

pub fn clean_ip(raw: &str) -> Option<String> {
    let s = raw
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn private_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()? {
        std::net::SocketAddr::V4(v4) => Some(v4.ip().to_string()),
        _ => None,
    }
}

pub fn fetch_wan(url: &str) -> Option<String> {
    let out = std::process::Command::new("curl")
        .args(["-s", "--max-time", "5", url])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    clean_ip(std::str::from_utf8(&out.stdout).ok()?)
}

pub fn spawn_ip_monitor(config: IpConfig) -> (Sender<IpCmd>, Receiver<IpState>) {
    let (state_tx, state_rx) = std::sync::mpsc::channel();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut cfg = config;
        let mut private = private_ip();
        let mut wan = if cfg.enabled { fetch_wan(&cfg.url) } else { None };
        let _ = state_tx.send(IpState {
            private: private.clone(),
            wan: wan.clone(),
        });

        loop {
            match cmd_rx.recv_timeout(Duration::from_secs(300)) {
                Ok(IpCmd::Update(new_cfg)) => {
                    cfg = new_cfg;
                    private = private_ip();
                    wan = if cfg.enabled { fetch_wan(&cfg.url) } else { None };
                    let _ = state_tx.send(IpState {
                        private: private.clone(),
                        wan: wan.clone(),
                    });
                }
                Err(RecvTimeoutError::Timeout) => {
                    private = private_ip();
                    wan = if cfg.enabled { fetch_wan(&cfg.url) } else { None };
                    let _ = state_tx.send(IpState {
                        private: private.clone(),
                        wan: wan.clone(),
                    });
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    (cmd_tx, state_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_ip_trims_whitespace_and_quotes() {
        assert_eq!(clean_ip("  1.2.3.4  ").as_deref(), Some("1.2.3.4"));
        assert_eq!(clean_ip("\"1.2.3.4\"").as_deref(), Some("1.2.3.4"));
        assert_eq!(clean_ip("'1.2.3.4'").as_deref(), Some("1.2.3.4"));
        assert_eq!(clean_ip("\"1.2.3.4\n\"").as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn clean_ip_empty_is_none() {
        assert_eq!(clean_ip(""), None);
        assert_eq!(clean_ip("   "), None);
        assert_eq!(clean_ip("\"\""), None);
        assert_eq!(clean_ip("\n"), None);
    }
}
