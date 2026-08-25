use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub enum Action {
    Quit,
    NextTheme,
    Tick,
    None,
}

pub fn poll_action(timeout: std::time::Duration) -> anyhow::Result<Action> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => Ok(Action::Quit),
                KeyCode::Char('t') => Ok(Action::NextTheme),
                _ => Ok(Action::None),
            },
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}
