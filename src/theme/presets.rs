use crate::theme::{Theme, ThemeColors};
use ratatui::style::Color;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn dracula() -> Theme {
    Theme {
        name: "dracula".to_string(),
        colors: ThemeColors {
            bg: rgb(40, 42, 54),
            fg: rgb(248, 248, 242),
            text: rgb(248, 248, 242),
            muted: rgb(98, 114, 164),
            accent: rgb(189, 147, 249),
            success: rgb(80, 250, 123),
            warning: rgb(241, 250, 140),
            danger: rgb(255, 85, 85),
            info: rgb(139, 233, 253),
            surface: rgb(68, 71, 90),
            border: rgb(68, 71, 90),
            highlight: rgb(68, 71, 90),
        },
    }
}

pub fn nord() -> Theme {
    Theme {
        name: "nord".to_string(),
        colors: ThemeColors {
            bg: rgb(46, 52, 64),
            fg: rgb(216, 222, 233),
            text: rgb(216, 222, 233),
            muted: rgb(76, 86, 106),
            accent: rgb(136, 192, 208),
            success: rgb(163, 190, 140),
            warning: rgb(235, 203, 139),
            danger: rgb(191, 97, 106),
            info: rgb(129, 161, 193),
            surface: rgb(59, 66, 82),
            border: rgb(67, 76, 94),
            highlight: rgb(59, 66, 82),
        },
    }
}

pub fn github_dark() -> Theme {
    Theme {
        name: "github-dark".to_string(),
        colors: ThemeColors {
            bg: rgb(13, 17, 23),
            fg: rgb(230, 237, 243),
            text: rgb(230, 237, 243),
            muted: rgb(139, 148, 158),
            accent: rgb(88, 166, 255),
            success: rgb(63, 185, 80),
            warning: rgb(210, 153, 34),
            danger: rgb(248, 81, 73),
            info: rgb(57, 197, 207),
            surface: rgb(22, 27, 34),
            border: rgb(48, 54, 61),
            highlight: rgb(22, 27, 34),
        },
    }
}

pub fn all() -> Vec<Theme> {
    vec![dracula(), nord(), github_dark()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_count_is_three() {
        assert_eq!(all().len(), 3);
    }

    #[test]
    fn dracula_bg_is_correct() {
        assert_eq!(dracula().colors.bg, Color::Rgb(40, 42, 54));
    }

    #[test]
    fn nord_accent_is_frost_blue() {
        assert_eq!(nord().colors.accent, Color::Rgb(136, 192, 208));
    }
}
