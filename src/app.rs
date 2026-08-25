use crate::config::{Config, GeneralConfig, ThemeConfig};
use crate::data::history::{History, ProcessHistory};
use crate::data::snapshot::ProcessInfo;
use crate::data::{self, snapshot::Snapshot, Command};
use crate::event::{poll_action, Action};
use crate::theme;
use crate::ui;
use crate::ui::widgets::processes::{self, SortDir, SortKey};
use anyhow::Result;
use ratatui::layout::Rect;
use std::collections::HashMap;

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
    filter: String,
    filtering: bool,
    display: Vec<ProcessInfo>,
    scroll: usize,
    detail: Option<ProcessInfo>,
    show_help: bool,
    proc_rect: Rect,
    proc_history: HashMap<u32, ProcessHistory>,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let themes = theme::catppuccin::all();
        let index = themes
            .iter()
            .position(|t| t.name == config.theme.flavor)
            .unwrap_or(0);
        let theme = themes.get(index).cloned().unwrap();
        Self {
            snapshot: Snapshot::default(),
            history: History::new(120),
            theme,
            themes,
            theme_index: index,
            sort_key: SortKey::Cpu,
            sort_dir: SortDir::Desc,
            selected: None,
            interval_ms: config.general.interval_ms,
            transparent: config.general.transparent,
            filter: String::new(),
            filtering: false,
            display: Vec::new(),
            scroll: 0,
            detail: None,
            show_help: false,
            proc_rect: Rect::default(),
            proc_history: HashMap::new(),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.theme_index].clone();
        let cfg = Config {
            theme: ThemeConfig { flavor: self.theme.name.clone() },
            general: GeneralConfig {
                interval_ms: self.interval_ms,
                transparent: self.transparent,
            },
        };
        let _ = cfg.save();
    }

    fn refresh_display(&mut self) {
        self.display = self
            .snapshot
            .processes
            .iter()
            .filter(|p| processes::matches_filter(&self.filter, p))
            .cloned()
            .collect();
        processes::sort(&mut self.display, self.sort_key, self.sort_dir);
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
    let interval = std::time::Duration::from_millis(config.general.interval_ms);
    let provider = data::build_provider();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let rx = data::spawn_sampler(provider, interval, cmd_rx);
    let mut app = App::new(config);

    loop {
        terminal.draw(|frame| {
            ui::render(
                frame,
                &app.snapshot,
                &app.theme,
                app.selected,
                &app.history,
                &app.display,
                app.scroll,
                &app.filter,
                app.filtering,
                app.detail.as_ref(),
                app.show_help,
                app.transparent,
                &mut app.proc_rect,
                app.snapshot.processes.len(),
                app.sort_key,
                app.sort_dir,
                &app.proc_history,
            )
        })?;

        let action = poll_action(std::time::Duration::from_millis(50), app.filtering)?;

        if let Action::Tick = action {
            while let Ok(snap) = rx.try_recv() {
                app.snapshot = snap;
                app.history.record(&app.snapshot);
                app.record_proc_history();
            }
            app.refresh_display();
        }

        if app.show_help || app.detail.is_some() {
            match action {
                Action::Quit | Action::OpenDetails | Action::Cancel | Action::ToggleHelp => {
                    app.show_help = false;
                    app.detail = None;
                }
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
            }
            Action::MoveUp => app.move_up(),
            Action::MoveDown => app.move_down(),
            Action::Kill => {
                if let Some(i) = app.selected {
                    if let Some(p) = app.display.get(i) {
                        let _ = cmd_tx.send(Command::Kill(p.pid));
                    }
                }
            }
            Action::OpenDetails => {
                if let Some(i) = app.selected {
                    app.detail = app.display.get(i).cloned();
                }
            }
            Action::ToggleHelp => app.show_help = true,
            Action::FilterToggle => app.filtering = true,
            Action::Click(col, row) => app.click(col, row),
            Action::ScrollUp => app.move_up(),
            Action::ScrollDown => app.move_down(),
            Action::Cancel => {}
            _ => {}
        }
    }
    Ok(())
}
