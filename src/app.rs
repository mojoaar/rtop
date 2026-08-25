use crate::config::{Config, GeneralConfig, ThemeConfig};
use crate::data::{self, snapshot::Snapshot, Command};
use crate::event::{poll_action, Action};
use crate::theme;
use crate::ui;
use crate::ui::widgets::processes::{self, SortKey};
use anyhow::Result;

pub struct App {
    snapshot: Snapshot,
    theme: theme::Theme,
    themes: Vec<theme::Theme>,
    theme_index: usize,
    sort_key: SortKey,
    selected: Option<usize>,
    interval_ms: u64,
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
            theme,
            themes,
            theme_index: index,
            sort_key: SortKey::Cpu,
            selected: None,
            interval_ms: config.general.interval_ms,
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.theme_index].clone();
        let cfg = Config {
            theme: ThemeConfig { flavor: self.theme.name.clone() },
            general: GeneralConfig { interval_ms: self.interval_ms },
        };
        let _ = cfg.save();
    }
}

struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

pub fn run(config: &Config) -> Result<()> {
    let mut terminal = ratatui::try_init()?;
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
        terminal.draw(|frame| ui::render(frame, &app.snapshot, &app.theme, app.selected))?;
        match poll_action(std::time::Duration::from_millis(50))? {
            Action::Quit => break,
            Action::NextTheme => app.cycle_theme(),
            Action::SortBy(key) => {
                app.sort_key = key;
                processes::sort(&mut app.snapshot.processes, app.sort_key);
            }
            Action::MoveUp => {
                let len = app.snapshot.processes.len();
                if len == 0 {
                    app.selected = None;
                } else {
                    app.selected = Some(match app.selected {
                        Some(i) if i > 0 => i - 1,
                        _ => 0,
                    });
                }
            }
            Action::MoveDown => {
                let len = app.snapshot.processes.len();
                if len == 0 {
                    app.selected = None;
                } else {
                    app.selected = Some(match app.selected {
                        Some(i) if i + 1 < len => i + 1,
                        None => 0,
                        _ => len - 1,
                    });
                }
            }
            Action::Kill => {
                if let Some(i) = app.selected {
                    if let Some(p) = app.snapshot.processes.get(i) {
                        let _ = cmd_tx.send(Command::Kill(p.pid));
                    }
                }
            }
            Action::Tick => {
                while let Ok(snap) = rx.try_recv() {
                    app.snapshot = snap;
                }
                processes::sort(&mut app.snapshot.processes, app.sort_key);
                if let Some(i) = app.selected {
                    if i >= app.snapshot.processes.len() {
                        app.selected = None;
                    }
                }
            }
            Action::None => {}
        }
    }
    Ok(())
}
