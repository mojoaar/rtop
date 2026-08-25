use crate::theme::{Theme, ThemeColors};
use catppuccin::PALETTE;
use ratatui::style::Color;

fn c(color: catppuccin::Color) -> Color {
    let r = color.rgb.r;
    let g = color.rgb.g;
    let b = color.rgb.b;
    Color::Rgb(r, g, b)
}

pub fn get(flavor: &str) -> Option<Theme> {
    let p = match flavor {
        "latte" => &PALETTE.latte,
        "frappe" => &PALETTE.frappe,
        "macchiato" => &PALETTE.macchiato,
        "mocha" => &PALETTE.mocha,
        _ => return None,
    };
    Some(Theme {
        name: flavor.to_string(),
        colors: ThemeColors {
            bg: c(p.colors.base),
            fg: c(p.colors.text),
            text: c(p.colors.text),
            muted: c(p.colors.overlay0),
            accent: c(p.colors.blue),
            success: c(p.colors.green),
            warning: c(p.colors.yellow),
            danger: c(p.colors.red),
            info: c(p.colors.sapphire),
            surface: c(p.colors.mantle),
            border: c(p.colors.surface0),
            highlight: c(p.colors.surface1),
        },
    })
}

pub fn all() -> Vec<Theme> {
    ["latte", "frappe", "macchiato", "mocha"]
        .iter()
        .filter_map(|f| get(f))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mocha_bg_is_base_hex() {
        let t = get("mocha").unwrap();
        assert_eq!(t.colors.bg, Color::Rgb(30, 30, 46)); // #1e1e2e
        assert_eq!(t.colors.success, Color::Rgb(166, 227, 161)); // #a6e3a1
    }

    #[test]
    fn unknown_flavor_is_none() {
        assert!(get("bogus").is_none());
    }

    #[test]
    fn all_returns_four_flavors() {
        assert_eq!(all().len(), 4);
    }
}
