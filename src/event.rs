use crate::ui::widgets::processes::SortKey;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub enum Action {
    Quit,
    NextTheme,
    SortBy(SortKey),
    MoveUp,
    MoveDown,
    Kill,
    Tick,
    None,
}

pub fn poll_action(timeout: std::time::Duration) -> anyhow::Result<Action> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => Ok(Action::Quit),
                KeyCode::Char('t') => Ok(Action::NextTheme),
                KeyCode::Char('c') => Ok(Action::SortBy(SortKey::Cpu)),
                KeyCode::Char('m') => Ok(Action::SortBy(SortKey::Memory)),
                KeyCode::Char('p') => Ok(Action::SortBy(SortKey::Pid)),
                KeyCode::Char('n') => Ok(Action::SortBy(SortKey::Name)),
                KeyCode::Up => Ok(Action::MoveUp),
                KeyCode::Down => Ok(Action::MoveDown),
                KeyCode::Char('k') => Ok(Action::Kill),
                _ => Ok(Action::None),
            },
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}
