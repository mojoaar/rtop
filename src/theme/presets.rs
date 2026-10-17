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

pub fn fjord() -> Theme {
    // Dark variant of the OpenCode "fjord" theme.
    Theme {
        name: "fjord".to_string(),
        colors: ThemeColors {
            bg: rgb(14, 22, 32),
            fg: rgb(198, 212, 224),
            text: rgb(198, 212, 224),
            muted: rgb(125, 144, 163),
            accent: rgb(82, 166, 175),
            success: rgb(162, 203, 182),
            warning: rgb(217, 203, 154),
            danger: rgb(196, 144, 147),
            info: rgb(159, 218, 223),
            surface: rgb(22, 32, 44),
            border: rgb(36, 51, 63),
            highlight: rgb(28, 41, 52),
        },
    }
}

pub fn nordlys() -> Theme {
    // Dark variant of the OpenCode "nordlys" theme.
    Theme {
        name: "nordlys".to_string(),
        colors: ThemeColors {
            bg: rgb(11, 16, 32),
            fg: rgb(204, 216, 238),
            text: rgb(204, 216, 238),
            muted: rgb(124, 138, 176),
            accent: rgb(110, 224, 168),
            success: rgb(158, 235, 196),
            warning: rgb(222, 204, 149),
            danger: rgb(203, 140, 153),
            info: rgb(151, 223, 235),
            surface: rgb(19, 26, 46),
            border: rgb(36, 48, 80),
            highlight: rgb(25, 35, 54),
        },
    }
}

pub fn ravn() -> Theme {
    // Dark variant of the OpenCode "ravn" theme.
    Theme {
        name: "ravn".to_string(),
        colors: ThemeColors {
            bg: rgb(10, 11, 18),
            fg: rgb(201, 209, 226),
            text: rgb(201, 209, 226),
            muted: rgb(118, 126, 146),
            accent: rgb(165, 150, 216),
            success: rgb(157, 198, 175),
            warning: rgb(212, 197, 150),
            danger: rgb(193, 136, 144),
            info: rgb(155, 204, 216),
            surface: rgb(19, 21, 31),
            border: rgb(35, 39, 52),
            highlight: rgb(25, 30, 39),
        },
    }
}

pub fn skumring() -> Theme {
    // Dark variant of the OpenCode "skumring" theme.
    Theme {
        name: "skumring".to_string(),
        colors: ThemeColors {
            bg: rgb(17, 18, 24),
            fg: rgb(204, 208, 221),
            text: rgb(204, 208, 221),
            muted: rgb(126, 132, 150),
            accent: rgb(147, 170, 203),
            success: rgb(183, 207, 191),
            warning: rgb(211, 202, 172),
            danger: rgb(200, 164, 169),
            info: rgb(174, 204, 210),
            surface: rgb(25, 27, 35),
            border: rgb(42, 45, 56),
            highlight: rgb(31, 36, 43),
        },
    }
}

pub fn all() -> Vec<Theme> {
    vec![
        dracula(),
        nord(),
        github_dark(),
        fjord(),
        nordlys(),
        ravn(),
        skumring(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_count_is_seven() {
        assert_eq!(all().len(), 7);
    }

    #[test]
    fn fjord_bg_matches_opencode_theme() {
        assert_eq!(fjord().colors.bg, Color::Rgb(14, 22, 32));
        assert_eq!(fjord().colors.accent, Color::Rgb(82, 166, 175));
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
