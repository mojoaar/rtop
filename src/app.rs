use crate::config::Config;
use crate::data::{self, snapshot::Snapshot};
use crate::event::{poll_action, Action};
use crate::theme;
use crate::ui;
use anyhow::Result;

pub struct App {
    snapshot: Snapshot,
    theme: theme::Theme,
    themes: Vec<theme::Theme>,
    theme_index: usize,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let themes = theme::catppuccin::all();
        let index = themes
            .iter()
            .position(|t| t.name == config.theme.flavor)
            .unwrap_or(0);
        let theme = themes.get(index).cloned().unwrap();
        Self { snapshot: Snapshot::default(), theme, themes, theme_index: index }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.theme_index].clone();
    }
}

pub fn run(config: &Config) -> Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = run_inner(&mut terminal, config);
    ratatui::restore();
    result
}

fn run_inner(terminal: &mut ratatui::DefaultTerminal, config: &Config) -> Result<()> {
    let interval = std::time::Duration::from_millis(config.general.interval_ms);
    let provider = data::build_provider();
    let rx = data::spawn_sampler(provider, interval);
    let mut app = App::new(config);

    loop {
        terminal.draw(|frame| ui::render(frame, &app.snapshot, &app.theme))?;
        match poll_action(std::time::Duration::from_millis(50))? {
            Action::Quit => break,
            Action::NextTheme => app.cycle_theme(),
            Action::Tick => {
                while let Ok(snap) = rx.try_recv() {
                    app.snapshot = snap;
                }
            }
            Action::None => {}
        }
    }
    Ok(())
}
