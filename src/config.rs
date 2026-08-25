use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub flavor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub interval_ms: u64,
    pub transparent: bool,
    pub show_time: bool,
    pub show_uptime: bool,
    pub wan_enabled: bool,
    pub wan_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { theme: ThemeConfig::default(), general: GeneralConfig::default() }
    }
}
impl Default for ThemeConfig {
    fn default() -> Self { Self { flavor: "mocha".into() } }
}
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            interval_ms: 500,
            transparent: true,
            show_time: true,
            show_uptime: true,
            wan_enabled: false,
            wan_url: "https://echo.johansen.foo/api/ip".into(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rtop").join("config.toml"))
    }
    pub fn load() -> anyhow::Result<Config> {
        match Self::config_path() {
            Some(p) if p.exists() => {
                let s = std::fs::read_to_string(&p)?;
                Ok(toml::from_str(&s)?)
            }
            _ => Ok(Config::default()),
        }
    }
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(p) = Self::config_path() {
            if let Some(dir) = p.parent() { std::fs::create_dir_all(dir)?; }
            std::fs::write(&p, toml::to_string_pretty(self)?)?;
        }
        Ok(())
    }
    pub fn with_theme(mut self, flavor: Option<String>) -> Self {
        if let Some(f) = flavor { self.theme.flavor = f; }
        self
    }
    pub fn with_interval(mut self, ms: Option<u64>) -> Self {
        if let Some(i) = ms { self.general.interval_ms = i; }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_mocha_500() {
        let c = Config::default();
        assert_eq!(c.theme.flavor, "mocha");
        assert_eq!(c.general.interval_ms, 500);
        assert!(c.general.transparent);
        assert!(c.general.show_time);
        assert!(c.general.show_uptime);
        assert!(!c.general.wan_enabled);
        assert_eq!(c.general.wan_url, "https://echo.johansen.foo/api/ip");
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let c: Config = toml::from_str("[theme]\nflavor = \"latte\"\n").unwrap();
        assert_eq!(c.theme.flavor, "latte");
        assert_eq!(c.general.interval_ms, 500);
        assert!(c.general.transparent);
        assert!(!c.general.wan_enabled);
    }

    #[test]
    fn cli_overrides_apply() {
        let c = Config::default()
            .with_theme(Some("macchiato".into()))
            .with_interval(Some(100));
        assert_eq!(c.theme.flavor, "macchiato");
        assert_eq!(c.general.interval_ms, 100);
    }
}
