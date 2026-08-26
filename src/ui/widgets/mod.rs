pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod processes;
pub mod sensors;

use crate::theme::Theme;
use ratatui::style::Color;

pub fn block_bar(ratio: f64, width: usize) -> String {
    let width = width.max(1);
    let filled = ((ratio.clamp(0.0, 1.0)) * width as f64).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

pub fn fullness_bucket(pct: f64) -> u8 {
    if pct < 60.0 {
        0
    } else if pct < 85.0 {
        1
    } else {
        2
    }
}

pub fn fullness_color(pct: f64, theme: &Theme) -> Color {
    match fullness_bucket(pct) {
        0 => theme.colors.success,
        1 => theme.colors.warning,
        _ => theme.colors.danger,
    }
}

pub fn bar_label_split(bar: &str, label: &str) -> Option<(String, String)> {
    let bar_len = bar.chars().count();
    let label_len = label.chars().count();
    if label_len > bar_len {
        return None;
    }
    let start = (bar_len - label_len) / 2;
    let left: String = bar.chars().take(start).collect();
    let right: String = bar.chars().skip(start + label_len).collect();
    Some((left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_bar_full() {
        assert_eq!(block_bar(1.0, 5), "█████");
    }

    #[test]
    fn block_bar_empty() {
        assert_eq!(block_bar(0.0, 5), "░░░░░");
    }

    #[test]
    fn block_bar_half_rounds() {
        assert_eq!(block_bar(0.5, 4), "██░░");
    }

    #[test]
    fn block_bar_clamps_over_one() {
        assert_eq!(block_bar(1.5, 4), "████");
    }

    #[test]
    fn block_bar_min_width() {
        assert_eq!(block_bar(0.5, 0), "█");
    }

    #[test]
    fn fullness_buckets() {
        assert_eq!(fullness_bucket(0.0), 0);
        assert_eq!(fullness_bucket(59.9), 0);
        assert_eq!(fullness_bucket(60.0), 1);
        assert_eq!(fullness_bucket(84.9), 1);
        assert_eq!(fullness_bucket(85.0), 2);
    }

    #[test]
    fn bar_label_split_centers_label() {
        assert_eq!(
            bar_label_split("████████", "42%"),
            Some(("██".to_string(), "███".to_string()))
        );
    }

    #[test]
    fn bar_label_split_exact_fit() {
        assert_eq!(
            bar_label_split("███", "abc"),
            Some(("".to_string(), "".to_string()))
        );
    }

    #[test]
    fn bar_label_split_too_narrow_is_none() {
        assert_eq!(bar_label_split("██", "abc"), None);
    }
}
