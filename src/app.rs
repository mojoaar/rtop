use crate::config::{Config, GeneralConfig, ThemeConfig};
use crate::data::history::{History, ProcessHistory};
use crate::data::ip::{self, IpCmd, IpConfig, IpState};
use crate::data::snapshot::ProcessInfo;
use crate::data::{self, snapshot::Snapshot, Command};
use crate::event::{poll_action, Action, Mode};
use crate::platform::signal::SignalChoice;
use crate::theme;
use crate::ui;
use crate::ui::widgets::processes::{self, SortDir, SortKey};
use crate::ui::RenderContext;
use anyhow::Result;
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

const INTERVAL_PRESETS: [u64; 6] = [100, 250, 500, 1000, 2000, 5000];
const MIN_INTERVAL_MS: u64 = 50;

fn preset_index(ms: u64) -> usize {
    INTERVAL_PRESETS
        .iter()
        .enumerate()
        .min_by_key(|&(_, &v)| (v as i64 - ms as i64).abs())
        .map(|(i, _)| i)
        .unwrap_or(3)
}

fn next_preset(ms: u64) -> u64 {
    let i = preset_index(ms);
    INTERVAL_PRESETS[(i + 1) % INTERVAL_PRESETS.len()]
}

fn prev_preset(ms: u64) -> u64 {
    let i = preset_index(ms);
    INTERVAL_PRESETS[(i + INTERVAL_PRESETS.len() - 1) % INTERVAL_PRESETS.len()]
}

pub struct App {
    snapshot: Snapshot,
    history: History,
    theme: theme::Theme,
    themes: Vec<theme::Theme>,
    theme_index: usize,
    sort_key: SortKey,
    sort_dir: SortDir,
    selected: Option<usize>,
    interval_ms: u64,
    transparent: bool,
    show_time: bool,
    show_uptime: bool,
    show_labels: bool,
    filter: String,
    filtering: bool,
    show_settings: bool,
    settings_index: usize,
    settings_editing: bool,
    show_signal: bool,
    signal_index: usize,
    wan_enabled: bool,
    wan_url: String,
    wan_url_edit: String,
    private_ip: Option<String>,
    wan_ip: Option<String>,
    ip_tx: Option<Sender<IpCmd>>,
    ip_rx: Receiver<IpState>,
    display: Vec<usize>,
    scroll: usize,
    detail: Option<ProcessInfo>,
    show_help: bool,
    fullscreen: bool,
    net_detail: bool,
    frozen: bool,
    proc_rect: Rect,
    proc_history: HashMap<u32, ProcessHistory>,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let themes = theme::all();
        let index = themes
            .iter()
            .position(|t| t.name == config.theme.flavor)
            .unwrap_or(0);
        let theme = themes
            .get(index)
            .cloned()
            .or_else(|| crate::theme::catppuccin::get("mocha"))
            .unwrap_or_else(|| themes[0].clone());
        let (ip_tx, ip_rx) = ip::spawn_ip_monitor(IpConfig {
            enabled: config.general.wan_enabled,
            url: config.general.wan_url.clone(),
        });
        Self {
            snapshot: Snapshot::default(),
            history: History::new(120),
            theme,
            themes,
            theme_index: index,
            sort_key: processes::parse_sort_key(&config.general.sort_key),
            sort_dir: processes::parse_sort_dir(&config.general.sort_dir),
            selected: None,
            interval_ms: config.general.interval_ms.max(MIN_INTERVAL_MS),
            transparent: config.general.transparent,
            show_time: config.general.show_time,
            show_uptime: config.general.show_uptime,
            show_labels: config.general.show_labels,
            filter: String::new(),
            filtering: false,
            show_settings: false,
            settings_index: 0,
            settings_editing: false,
            show_signal: false,
            signal_index: 0,
            wan_enabled: config.general.wan_enabled,
            wan_url: config.general.wan_url.clone(),
            wan_url_edit: String::new(),
            private_ip: None,
            wan_ip: None,
            ip_tx: Some(ip_tx),
            ip_rx,
            display: Vec::new(),
            scroll: 0,
            detail: None,
            show_help: false,
            fullscreen: false,
            net_detail: false,
            frozen: false,
            proc_rect: Rect::default(),
            proc_history: HashMap::new(),
        }
    }

    fn cycle_theme(&mut self) {
        if self.themes.is_empty() {
            return;
        }
        self.theme_index = (self.theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.theme_index].clone();
        self.save_config();
    }

    fn cycle_theme_back(&mut self) {
        if self.themes.is_empty() {
            return;
        }
        let n = self.themes.len();
        self.theme_index = (self.theme_index + n - 1) % n;
        self.theme = self.themes[self.theme_index].clone();
        self.save_config();
    }

    fn save_config(&self) {
        let cfg = Config {
            theme: ThemeConfig {
                flavor: self.theme.name.clone(),
            },
            general: GeneralConfig {
                interval_ms: self.interval_ms,
                transparent: self.transparent,
                show_time: self.show_time,
                show_uptime: self.show_uptime,
                show_labels: self.show_labels,
                sort_key: self.sort_key.as_str().into(),
                sort_dir: self.sort_dir.as_str().into(),
                wan_enabled: self.wan_enabled,
                wan_url: self.wan_url.clone(),
            },
        };
        if let Err(e) = cfg.save() {
            eprintln!("rtop: failed to save config: {e}");
        }
    }

    fn send_ip_update(&self) {
        if let Some(tx) = &self.ip_tx {
            let _ = tx.send(IpCmd::Update(IpConfig {
                enabled: self.wan_enabled,
                url: self.wan_url.clone(),
            }));
        }
    }

    fn set_interval(&mut self, ms: u64, cmd_tx: &std::sync::mpsc::Sender<Command>) {
        self.interval_ms = ms;
        let _ = cmd_tx.send(Command::SetInterval(ms));
        self.save_config();
    }

    fn settings_up(&mut self) {
        if self.settings_index > 0 {
            self.settings_index -= 1;
        }
    }

    fn settings_down(&mut self) {
        if self.settings_index < 7 {
            self.settings_index += 1;
        }
    }

    fn toggle_setting(&mut self, index: usize) {
        match index {
            2 => self.transparent = !self.transparent,
            3 => self.show_time = !self.show_time,
            4 => self.show_uptime = !self.show_uptime,
            5 => self.show_labels = !self.show_labels,
            6 => {
                self.wan_enabled = !self.wan_enabled;
                self.save_config();
                self.send_ip_update();
                return;
            }
            _ => return,
        }
        self.save_config();
    }

    fn settings_dec(&mut self, cmd_tx: &std::sync::mpsc::Sender<Command>) {
        match self.settings_index {
            0 => {
                let ms = prev_preset(self.interval_ms);
                self.set_interval(ms, cmd_tx);
            }
            1 => self.cycle_theme_back(),
            _ => self.toggle_setting(self.settings_index),
        }
    }

    fn settings_inc(&mut self, cmd_tx: &std::sync::mpsc::Sender<Command>) {
        match self.settings_index {
            0 => {
                let ms = next_preset(self.interval_ms);
                self.set_interval(ms, cmd_tx);
            }
            1 => self.cycle_theme(),
            _ => self.toggle_setting(self.settings_index),
        }
    }

    fn refresh_display(&mut self) {
        let procs = &self.snapshot.processes;
        let mut idx: Vec<usize> = (0..procs.len())
            .filter(|&i| processes::matches_filter(&self.filter, &procs[i]))
            .collect();
        idx.sort_by(|&a, &b| processes::cmp(&procs[a], &procs[b], self.sort_key, self.sort_dir));
        self.display = idx;
        if let Some(i) = self.selected {
            if i >= self.display.len() {
                self.selected = None;
            }
        }
        let max_scroll = self.display.len().saturating_sub(1);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
        self.ensure_visible();
    }

    fn proc_at(&self, i: usize) -> Option<&ProcessInfo> {
        self.display
            .get(i)
            .copied()
            .and_then(|idx| self.snapshot.processes.get(idx))
    }

    fn visible_rows(&self) -> usize {
        (self.proc_rect.height.saturating_sub(3)).max(1) as usize
    }

    fn ensure_visible(&mut self) {
        if let Some(i) = self.selected {
            let vis = self.visible_rows();
            if i < self.scroll {
                self.scroll = i;
            } else if i >= self.scroll.saturating_add(vis) {
                self.scroll = i - vis + 1;
            }
        }
    }

    fn move_up(&mut self) {
        let len = self.display.len();
        if len == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            Some(i) if i > 0 => i - 1,
            _ => 0,
        });
        self.ensure_visible();
    }

    fn move_down(&mut self) {
        let len = self.display.len();
        if len == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            Some(i) if i + 1 < len => i + 1,
            None => 0,
            _ => len - 1,
        });
        self.ensure_visible();
    }

    fn record_proc_history(&mut self) {
        let mut alive = std::collections::HashSet::new();
        for p in &self.snapshot.processes {
            alive.insert(p.pid);
            self.proc_history
                .entry(p.pid)
                .or_insert_with(|| ProcessHistory::new(120))
                .record(p.cpu_usage, p.memory_bytes);
        }
        self.proc_history.retain(|pid, _| alive.contains(pid));
    }

    fn click(&mut self, col: u16, row: u16) {
        let r = self.proc_rect;
        if col < r.x || col >= r.x.saturating_add(r.width) {
            return;
        }
        if row < r.y || row >= r.y.saturating_add(r.height) {
            return;
        }
        let header_row = r.y.saturating_add(2);
        if let Some(rel) = row.checked_sub(header_row) {
            let idx = self.scroll.saturating_add(rel as usize);
            if idx < self.display.len() {
                self.selected = Some(idx);
            }
        }
    }
}

struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        ratatui::restore();
    }
}

pub fn run(config: &Config) -> Result<()> {
    let mut terminal = ratatui::try_init()?;
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    let _guard = TermGuard;
    run_inner(&mut terminal, config)
}

fn run_inner(terminal: &mut ratatui::DefaultTerminal, config: &Config) -> Result<()> {
    let interval =
        std::time::Duration::from_millis(config.general.interval_ms.max(MIN_INTERVAL_MS));
    let provider = data::build_provider();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let rx = data::spawn_sampler(provider, interval, cmd_rx);
    let mut app = App::new(config);
    let mut dirty = true;

    loop {
        if dirty {
            terminal.draw(|frame| {
                app.proc_rect = ui::render(
                    frame,
                    &RenderContext {
                        snapshot: &app.snapshot,
                        theme: &app.theme,
                        selected: app.selected,
                        history: &app.history,
                        processes: &app.snapshot.processes,
                        order: &app.display,
                        scroll: app.scroll,
                        filter: &app.filter,
                        filtering: app.filtering,
                        detail: app.detail.as_ref(),
                        show_help: app.show_help,
                        show_settings: app.show_settings,
                        settings_index: app.settings_index,
                        show_signal: app.show_signal,
                        signal_index: app.signal_index,
                        interval_ms: app.interval_ms,
                        transparent: app.transparent,
                        show_time: app.show_time,
                        show_uptime: app.show_uptime,
                        show_labels: app.show_labels,
                        fullscreen: app.fullscreen,
                        net_detail: app.net_detail,
                        frozen: app.frozen,
                        wan_enabled: app.wan_enabled,
                        private_ip: app.private_ip.as_deref(),
                        wan_ip: app.wan_ip.as_deref(),
                        settings_editing: app.settings_editing,
                        wan_url: &app.wan_url,
                        wan_url_edit: &app.wan_url_edit,
                        total: app.snapshot.processes.len(),
                        sort_key: app.sort_key,
                        sort_dir: app.sort_dir,
                        proc_history: &app.proc_history,
                    },
                )
            })?;
            dirty = false;
        }

        let mode = if app.filtering {
            Mode::Filtering
        } else if app.settings_editing {
            Mode::SettingsEdit
        } else if app.show_settings {
            Mode::Settings
        } else if app.show_signal {
            Mode::Signal
        } else {
            Mode::Normal
        };
        let action = poll_action(std::time::Duration::from_millis(50), mode)?;

        if !matches!(action, Action::Tick | Action::None) {
            dirty = true;
        }

        if let Action::Tick = action {
            if app.frozen {
                while rx.try_recv().is_ok() {}
                while app.ip_rx.try_recv().is_ok() {}
            } else {
                let mut updated = false;
                while let Ok(snap) = rx.try_recv() {
                    app.snapshot = snap;
                    app.history.record(&app.snapshot);
                    app.record_proc_history();
                    updated = true;
                }
                while let Ok(state) = app.ip_rx.try_recv() {
                    app.private_ip = state.private;
                    app.wan_ip = state.wan;
                    updated = true;
                }
                if updated {
                    app.refresh_display();
                    if let Some(d) = &app.detail {
                        if let Some(live) = app
                            .display
                            .iter()
                            .copied()
                            .find(|&i| app.snapshot.processes[i].pid == d.pid)
                            .and_then(|i| app.snapshot.processes.get(i))
                        {
                            app.detail = Some(live.clone());
                        }
                    }
                    dirty = true;
                }
            }
        }

        if app.show_help || app.detail.is_some() || app.net_detail {
            match action {
                Action::Quit | Action::OpenDetails | Action::Cancel | Action::ToggleHelp => {
                    app.show_help = false;
                    app.detail = None;
                    app.net_detail = false;
                }
                Action::ToggleNetDetail => app.net_detail = false,
                _ => {}
            }
            continue;
        }

        if app.show_settings {
            if app.settings_editing {
                match action {
                    Action::FilterChar(c) => app.wan_url_edit.push(c),
                    Action::FilterBackspace => {
                        app.wan_url_edit.pop();
                    }
                    Action::FilterSubmit => {
                        app.wan_url = app.wan_url_edit.clone();
                        app.settings_editing = false;
                        app.save_config();
                        app.send_ip_update();
                    }
                    Action::FilterCancel => {
                        app.settings_editing = false;
                    }
                    Action::Quit => break,
                    _ => {}
                }
                continue;
            }
            match action {
                Action::SettingsUp => app.settings_up(),
                Action::SettingsDown => app.settings_down(),
                Action::SettingsDec => app.settings_dec(&cmd_tx),
                Action::SettingsInc => app.settings_inc(&cmd_tx),
                Action::SettingsActivate => {
                    if app.settings_index == 7 {
                        app.wan_url_edit = app.wan_url.clone();
                        app.settings_editing = true;
                    }
                }
                Action::Cancel | Action::OpenSettings | Action::Quit => app.show_settings = false,
                _ => {}
            }
            continue;
        }

        if app.show_signal {
            match action {
                Action::SignalUp => {
                    if app.signal_index > 0 {
                        app.signal_index -= 1;
                    }
                }
                Action::SignalDown => {
                    if app.signal_index < 2 {
                        app.signal_index += 1;
                    }
                }
                Action::SignalConfirm => {
                    let signal = match app.signal_index {
                        0 => SignalChoice::Term,
                        1 => SignalChoice::Kill,
                        _ => SignalChoice::Interrupt,
                    };
                    if let Some(i) = app.selected {
                        if let Some(p) = app.proc_at(i) {
                            let _ = cmd_tx.send(Command::Kill { pid: p.pid, signal });
                        }
                    }
                    app.show_signal = false;
                }
                Action::SignalCancel | Action::Cancel | Action::Quit => app.show_signal = false,
                _ => {}
            }
            continue;
        }

        if app.filtering {
            match action {
                Action::FilterChar(c) => {
                    app.filter.push(c);
                    app.refresh_display();
                }
                Action::FilterBackspace => {
                    app.filter.pop();
                    app.refresh_display();
                }
                Action::FilterSubmit => app.filtering = false,
                Action::FilterCancel => {
                    app.filtering = false;
                    app.filter.clear();
                    app.refresh_display();
                }
                Action::FilterToggle => app.filtering = false,
                Action::Quit => break,
                _ => {}
            }
            continue;
        }

        match action {
            Action::Quit => break,
            Action::NextTheme => app.cycle_theme(),
            Action::SortBy(key) => {
                if app.sort_key == key {
                    app.sort_dir = match app.sort_dir {
                        SortDir::Asc => SortDir::Desc,
                        SortDir::Desc => SortDir::Asc,
                    };
                } else {
                    app.sort_key = key;
                    app.sort_dir = processes::default_dir(key);
                }
                app.refresh_display();
                app.save_config();
            }
            Action::MoveUp => app.move_up(),
            Action::MoveDown => app.move_down(),
            Action::OpenSignal => {
                if app.selected.is_some() {
                    app.signal_index = 0;
                    app.show_signal = true;
                }
            }
            Action::OpenDetails => {
                if let Some(i) = app.selected {
                    app.detail = app.proc_at(i).cloned();
                }
            }
            Action::ToggleHelp => app.show_help = true,
            Action::ToggleZoom => app.fullscreen = !app.fullscreen,
            Action::ToggleNetDetail => app.net_detail = !app.net_detail,
            Action::ToggleFreeze => app.frozen = !app.frozen,
            Action::OpenSettings => app.show_settings = true,
            Action::FilterToggle => app.filtering = true,
            Action::Click(col, row) => app.click(col, row),
            Action::ScrollUp => app.move_up(),
            Action::ScrollDown => app.move_down(),
            Action::Cancel => app.fullscreen = false,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_index_finds_nearest() {
        assert_eq!(preset_index(1000), 3);
        assert_eq!(preset_index(250), 1);
        assert_eq!(preset_index(100), 0);
        assert_eq!(preset_index(5000), 5);
    }

    #[test]
    fn preset_index_handles_off_grid() {
        assert_eq!(preset_index(750), 2);
        assert_eq!(preset_index(0), 0);
        assert_eq!(preset_index(999_999), 5);
    }

    #[test]
    fn next_prev_preset_wrap() {
        assert_eq!(next_preset(5000), 100);
        assert_eq!(prev_preset(100), 5000);
        assert_eq!(next_preset(1000), 2000);
        assert_eq!(prev_preset(1000), 500);
    }

    #[test]
    fn interval_ms_clamped_above_minimum() {
        let mut config = Config::default();
        config.general.interval_ms = 0;
        let app = App::new(&config);
        assert!(app.interval_ms >= MIN_INTERVAL_MS);
    }

    #[test]
    fn refresh_display_applies_filter_and_sort() {
        let mut app = App::new(&Config::default());
        app.snapshot.processes = vec![
            ProcessInfo {
                pid: 1,
                name: "zsh".into(),
                cpu_usage: 10.0,
                ..Default::default()
            },
            ProcessInfo {
                pid: 2,
                name: "firefox".into(),
                cpu_usage: 50.0,
                ..Default::default()
            },
            ProcessInfo {
                pid: 3,
                name: "chrome".into(),
                cpu_usage: 30.0,
                ..Default::default()
            },
        ];
        app.filter = "fire".into();
        app.sort_key = SortKey::Cpu;
        app.sort_dir = SortDir::Desc;
        app.refresh_display();
        assert_eq!(app.display.len(), 1);
        assert_eq!(app.snapshot.processes[app.display[0]].name, "firefox");
    }

    #[test]
    fn move_up_and_down_clamp_at_bounds() {
        let mut app = App::new(&Config::default());
        app.display = vec![0, 1, 2];
        app.move_down();
        assert_eq!(app.selected, Some(0));
        app.move_up();
        assert_eq!(app.selected, Some(0));
        app.selected = Some(2);
        app.move_down();
        assert_eq!(app.selected, Some(2));
    }

    #[test]
    fn click_maps_to_scrolled_index() {
        let mut app = App::new(&Config::default());
        app.display = vec![0, 1, 2, 3];
        app.proc_rect = Rect::new(0, 0, 80, 20);
        app.scroll = 1;
        app.click(40, 3);
        assert_eq!(app.selected, Some(2));
    }
}
