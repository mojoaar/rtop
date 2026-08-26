use crate::ui::widgets::processes::SortKey;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextTheme,
    SortBy(SortKey),
    MoveUp,
    MoveDown,
    OpenSignal,
    SignalUp,
    SignalDown,
    SignalConfirm,
    SignalCancel,
    OpenDetails,
    ToggleHelp,
    ToggleZoom,
    ToggleNetDetail,
    ToggleFreeze,
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
    Signal,
}

pub fn poll_action(timeout: std::time::Duration, mode: Mode) -> anyhow::Result<Action> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => Ok(key_to_action(key.code, mode)),
            Event::Mouse(mouse) => Ok(mouse_to_action(mouse.kind, mouse.column, mouse.row)),
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}

pub fn key_to_action(code: KeyCode, mode: Mode) -> Action {
    match mode {
        Mode::Filtering => match code {
            KeyCode::Esc => Action::FilterCancel,
            KeyCode::Enter => Action::FilterSubmit,
            KeyCode::Backspace => Action::FilterBackspace,
            KeyCode::Char(c) => Action::FilterChar(c),
            _ => Action::None,
        },
        Mode::Settings => match code {
            KeyCode::Esc | KeyCode::Char('q') => Action::Cancel,
            KeyCode::Enter => Action::SettingsActivate,
            KeyCode::Up => Action::SettingsUp,
            KeyCode::Down => Action::SettingsDown,
            KeyCode::Left => Action::SettingsDec,
            KeyCode::Right => Action::SettingsInc,
            _ => Action::None,
        },
        Mode::SettingsEdit => match code {
            KeyCode::Esc => Action::FilterCancel,
            KeyCode::Enter => Action::FilterSubmit,
            KeyCode::Backspace => Action::FilterBackspace,
            KeyCode::Char(c) => Action::FilterChar(c),
            _ => Action::None,
        },
        Mode::Signal => match code {
            KeyCode::Up => Action::SignalUp,
            KeyCode::Down => Action::SignalDown,
            KeyCode::Enter => Action::SignalConfirm,
            KeyCode::Esc => Action::SignalCancel,
            _ => Action::None,
        },
        Mode::Normal => match code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('t') => Action::NextTheme,
            KeyCode::Char('c') => Action::SortBy(SortKey::Cpu),
            KeyCode::Char('m') => Action::SortBy(SortKey::Memory),
            KeyCode::Char('p') => Action::SortBy(SortKey::Pid),
            KeyCode::Char('n') => Action::SortBy(SortKey::Name),
            KeyCode::Char('f') => Action::FilterToggle,
            KeyCode::Char('s') => Action::OpenSettings,
            KeyCode::Char('?') => Action::ToggleHelp,
            KeyCode::Char('z') => Action::ToggleZoom,
            KeyCode::Char('i') => Action::ToggleNetDetail,
            KeyCode::Char(' ') => Action::ToggleFreeze,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') => Action::OpenSignal,
            KeyCode::Enter => Action::OpenDetails,
            KeyCode::Esc => Action::Cancel,
            _ => Action::None,
        },
    }
}

pub fn mouse_to_action(kind: MouseEventKind, column: u16, row: u16) -> Action {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => Action::Click(column, row),
        MouseEventKind::ScrollUp => Action::ScrollUp,
        MouseEventKind::ScrollDown => Action::ScrollDown,
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_mode_key_mapping() {
        assert_eq!(
            key_to_action(KeyCode::Char('q'), Mode::Normal),
            Action::Quit
        );
        assert_eq!(
            key_to_action(KeyCode::Char('t'), Mode::Normal),
            Action::NextTheme
        );
        assert_eq!(
            key_to_action(KeyCode::Char('c'), Mode::Normal),
            Action::SortBy(SortKey::Cpu)
        );
        assert_eq!(
            key_to_action(KeyCode::Char('m'), Mode::Normal),
            Action::SortBy(SortKey::Memory)
        );
        assert_eq!(
            key_to_action(KeyCode::Char('p'), Mode::Normal),
            Action::SortBy(SortKey::Pid)
        );
        assert_eq!(
            key_to_action(KeyCode::Char('n'), Mode::Normal),
            Action::SortBy(SortKey::Name)
        );
        assert_eq!(
            key_to_action(KeyCode::Char('f'), Mode::Normal),
            Action::FilterToggle
        );
        assert_eq!(
            key_to_action(KeyCode::Char('s'), Mode::Normal),
            Action::OpenSettings
        );
        assert_eq!(
            key_to_action(KeyCode::Char('?'), Mode::Normal),
            Action::ToggleHelp
        );
        assert_eq!(
            key_to_action(KeyCode::Char('z'), Mode::Normal),
            Action::ToggleZoom
        );
        assert_eq!(
            key_to_action(KeyCode::Char('i'), Mode::Normal),
            Action::ToggleNetDetail
        );
        assert_eq!(
            key_to_action(KeyCode::Char(' '), Mode::Normal),
            Action::ToggleFreeze
        );
        assert_eq!(key_to_action(KeyCode::Up, Mode::Normal), Action::MoveUp);
        assert_eq!(key_to_action(KeyCode::Down, Mode::Normal), Action::MoveDown);
        assert_eq!(
            key_to_action(KeyCode::Char('k'), Mode::Normal),
            Action::OpenSignal
        );
        assert_eq!(
            key_to_action(KeyCode::Enter, Mode::Normal),
            Action::OpenDetails
        );
        assert_eq!(key_to_action(KeyCode::Esc, Mode::Normal), Action::Cancel);
        assert_eq!(key_to_action(KeyCode::F(1), Mode::Normal), Action::None);
    }

    #[test]
    fn filtering_mode_key_mapping() {
        assert_eq!(
            key_to_action(KeyCode::Esc, Mode::Filtering),
            Action::FilterCancel
        );
        assert_eq!(
            key_to_action(KeyCode::Enter, Mode::Filtering),
            Action::FilterSubmit
        );
        assert_eq!(
            key_to_action(KeyCode::Backspace, Mode::Filtering),
            Action::FilterBackspace
        );
        assert_eq!(
            key_to_action(KeyCode::Char('x'), Mode::Filtering),
            Action::FilterChar('x')
        );
        assert_eq!(key_to_action(KeyCode::Up, Mode::Filtering), Action::None);
    }

    #[test]
    fn settings_mode_key_mapping() {
        assert_eq!(key_to_action(KeyCode::Esc, Mode::Settings), Action::Cancel);
        assert_eq!(
            key_to_action(KeyCode::Char('q'), Mode::Settings),
            Action::Cancel
        );
        assert_eq!(
            key_to_action(KeyCode::Enter, Mode::Settings),
            Action::SettingsActivate
        );
        assert_eq!(
            key_to_action(KeyCode::Up, Mode::Settings),
            Action::SettingsUp
        );
        assert_eq!(
            key_to_action(KeyCode::Down, Mode::Settings),
            Action::SettingsDown
        );
        assert_eq!(
            key_to_action(KeyCode::Left, Mode::Settings),
            Action::SettingsDec
        );
        assert_eq!(
            key_to_action(KeyCode::Right, Mode::Settings),
            Action::SettingsInc
        );
    }

    #[test]
    fn settings_edit_mode_key_mapping() {
        assert_eq!(
            key_to_action(KeyCode::Esc, Mode::SettingsEdit),
            Action::FilterCancel
        );
        assert_eq!(
            key_to_action(KeyCode::Enter, Mode::SettingsEdit),
            Action::FilterSubmit
        );
        assert_eq!(
            key_to_action(KeyCode::Backspace, Mode::SettingsEdit),
            Action::FilterBackspace
        );
        assert_eq!(
            key_to_action(KeyCode::Char('u'), Mode::SettingsEdit),
            Action::FilterChar('u')
        );
    }

    #[test]
    fn signal_mode_key_mapping() {
        assert_eq!(key_to_action(KeyCode::Up, Mode::Signal), Action::SignalUp);
        assert_eq!(
            key_to_action(KeyCode::Down, Mode::Signal),
            Action::SignalDown
        );
        assert_eq!(
            key_to_action(KeyCode::Enter, Mode::Signal),
            Action::SignalConfirm
        );
        assert_eq!(
            key_to_action(KeyCode::Esc, Mode::Signal),
            Action::SignalCancel
        );
        assert_eq!(
            key_to_action(KeyCode::Char('q'), Mode::Signal),
            Action::None
        );
    }

    #[test]
    fn mouse_click_mapping() {
        assert_eq!(
            mouse_to_action(MouseEventKind::Down(MouseButton::Left), 3, 7),
            Action::Click(3, 7)
        );
    }

    #[test]
    fn mouse_scroll_mapping() {
        assert_eq!(
            mouse_to_action(MouseEventKind::ScrollUp, 0, 0),
            Action::ScrollUp
        );
        assert_eq!(
            mouse_to_action(MouseEventKind::ScrollDown, 0, 0),
            Action::ScrollDown
        );
        assert_eq!(mouse_to_action(MouseEventKind::Moved, 0, 0), Action::None);
    }
}
