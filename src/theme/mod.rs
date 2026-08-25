use ratatui::style::Color;

pub mod catppuccin;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    pub bg: Color,
    pub fg: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    pub surface: Color,
    pub border: Color,
    pub highlight: Color,
}
