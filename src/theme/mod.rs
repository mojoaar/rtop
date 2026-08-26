use ratatui::style::Color;

pub mod catppuccin;
pub mod presets;

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

pub fn all() -> Vec<Theme> {
    let mut themes = catppuccin::all();
    themes.extend(presets::all());
    themes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_eight_themes() {
        assert_eq!(all().len(), 8);
    }
}
