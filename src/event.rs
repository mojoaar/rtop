use crate::ui::widgets::processes::SortKey;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    NextTheme,
    SortBy(SortKey),
    MoveUp,
    MoveDown,
    Kill,
    OpenDetails,
    ToggleHelp,
    OpenSettings,
    SettingsUp,
    SettingsDown,
    SettingsDec,
    SettingsInc,
    SettingsActivate,
    FilterToggle,
    FilterChar(char),
    FilterBackspace,
    FilterSubmit,
    FilterCancel,
    Cancel,
    Click(u16, u16),
    ScrollUp,
    ScrollDown,
    Tick,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Filtering,
    Settings,
    SettingsEdit,
}

pub fn poll_action(timeout: std::time::Duration, mode: Mode) -> anyhow::Result<Action> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match mode {
                    Mode::Filtering => match key.code {
                        KeyCode::Esc => Ok(Action::FilterCancel),
                        KeyCode::Enter => Ok(Action::FilterSubmit),
                        KeyCode::Backspace => Ok(Action::FilterBackspace),
                        KeyCode::Char(c) => Ok(Action::FilterChar(c)),
                        _ => Ok(Action::None),
                    },
                    Mode::Settings => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => Ok(Action::Cancel),
                        KeyCode::Enter => Ok(Action::SettingsActivate),
                        KeyCode::Up => Ok(Action::SettingsUp),
                        KeyCode::Down => Ok(Action::SettingsDown),
                        KeyCode::Left => Ok(Action::SettingsDec),
                        KeyCode::Right => Ok(Action::SettingsInc),
                        _ => Ok(Action::None),
                    },
                    Mode::SettingsEdit => match key.code {
                        KeyCode::Esc => Ok(Action::FilterCancel),
                        KeyCode::Enter => Ok(Action::FilterSubmit),
                        KeyCode::Backspace => Ok(Action::FilterBackspace),
                        KeyCode::Char(c) => Ok(Action::FilterChar(c)),
                        _ => Ok(Action::None),
                    },
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => Ok(Action::Quit),
                        KeyCode::Char('t') => Ok(Action::NextTheme),
                        KeyCode::Char('c') => Ok(Action::SortBy(SortKey::Cpu)),
                        KeyCode::Char('m') => Ok(Action::SortBy(SortKey::Memory)),
                        KeyCode::Char('p') => Ok(Action::SortBy(SortKey::Pid)),
                        KeyCode::Char('n') => Ok(Action::SortBy(SortKey::Name)),
                        KeyCode::Char('f') => Ok(Action::FilterToggle),
                        KeyCode::Char('s') => Ok(Action::OpenSettings),
                        KeyCode::Char('?') => Ok(Action::ToggleHelp),
                        KeyCode::Up => Ok(Action::MoveUp),
                        KeyCode::Down => Ok(Action::MoveDown),
                        KeyCode::Char('k') => Ok(Action::Kill),
                        KeyCode::Enter => Ok(Action::OpenDetails),
                        KeyCode::Esc => Ok(Action::Cancel),
                        _ => Ok(Action::None),
                    },
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => Ok(Action::Click(mouse.column, mouse.row)),
                MouseEventKind::ScrollUp => Ok(Action::ScrollUp),
                MouseEventKind::ScrollDown => Ok(Action::ScrollDown),
                _ => Ok(Action::None),
            },
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}
