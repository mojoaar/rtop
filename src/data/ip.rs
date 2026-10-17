use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

const PRIVATE_EVERY: Duration = Duration::from_secs(30);
const WAN_EVERY: Duration = Duration::from_secs(300);
/// Anything longer is not an IP address; avoids holding a whole error page.
const MAX_WAN_BODY: usize = 256;

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
    raw.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':'))
        .map(|t| t.trim_matches(|c| c == ':' || c == '.'))
        .find_map(|t| t.parse::<std::net::IpAddr>().ok())
        .map(|ip| ip.to_string())
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
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }
    let out = std::process::Command::new("curl")
        .args(["-sf", "--max-time", "5", "--", url])
        .output()
        .ok()?;
    if !out.status.success() || out.stdout.len() > MAX_WAN_BODY {
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
        let mut wan = if cfg.enabled {
            fetch_wan(&cfg.url)
        } else {
            None
        };
        let mut last_wan = Instant::now();
        let _ = state_tx.send(IpState {
            private: private.clone(),
            wan: wan.clone(),
        });

        loop {
            match cmd_rx.recv_timeout(PRIVATE_EVERY) {
                Ok(IpCmd::Update(new_cfg)) => {
                    cfg = new_cfg;
                    private = private_ip();
                    wan = if cfg.enabled {
                        fetch_wan(&cfg.url)
                    } else {
                        None
                    };
                    last_wan = Instant::now();
                }
                Err(RecvTimeoutError::Timeout) => {
                    private = private_ip();
                    if cfg.enabled && last_wan.elapsed() >= WAN_EVERY {
                        wan = fetch_wan(&cfg.url);
                        last_wan = Instant::now();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
            let _ = state_tx.send(IpState {
                private: private.clone(),
                wan: wan.clone(),
            });
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

    #[test]
    fn clean_ip_extracts_from_wrapping_text() {
        assert_eq!(clean_ip("{\"ip\":\"1.2.3.4\"}").as_deref(), Some("1.2.3.4"));
        assert_eq!(
            clean_ip("your ip is 2001:db8::1 today").as_deref(),
            Some("2001:db8::1")
        );
        assert_eq!(clean_ip("1.2.3.4.").as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn clean_ip_rejects_control_chars_and_junk() {
        assert_eq!(clean_ip("\u{1b}]0;pwned\u{7}"), None);
        assert_eq!(clean_ip("<html>no address here</html>"), None);
    }

    #[test]
    fn fetch_wan_rejects_non_http_urls() {
        assert_eq!(fetch_wan("file:///etc/passwd"), None);
        assert_eq!(fetch_wan("-o /etc/passwd"), None);
        assert_eq!(fetch_wan(""), None);
    }
}
