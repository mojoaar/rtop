use crate::data::format::format_duration_secs;
use crate::data::snapshot::{BatteryInfo, ComponentInfo, FanInfo};
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

const MAX_COMPONENTS: usize = 4;

pub fn format_battery_health(cycle_count: Option<u32>, health_percent: Option<f32>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = cycle_count {
        parts.push(format!("cycles: {}", c));
    }
    if let Some(h) = health_percent {
        parts.push(format!("health: {:.0}%", h));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("  ·  {}", parts.join("  ·  "))
    }
}

pub fn selected_components(components: &[ComponentInfo], max: usize) -> Vec<&ComponentInfo> {
    let is_priority = |c: &ComponentInfo| {
        let l = c.label.to_lowercase();
        l.contains("cpu") || l.contains("gpu") || l.contains("graphic")
    };
    let mut out: Vec<&ComponentInfo> = Vec::new();
    for c in components.iter().filter(|c| is_priority(c)) {
        if out.len() >= max {
            break;
        }
        out.push(c);
    }
    for c in components.iter().filter(|c| !is_priority(c)) {
        if out.len() >= max {
            break;
        }
        out.push(c);
    }
    out
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    battery: Option<&BatteryInfo>,
    components: &[ComponentInfo],
    fans: &[FanInfo],
    theme: &Theme,
) {
    let block = Block::bordered()
        .title(" Battery & Sensors ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(b) = battery {
        let mut line = format!("Battery: {:.0}% ({})", b.percentage, b.state);
        if let Some(t) = b.time_to_empty_secs {
            line.push_str(&format!("  ·  empty in {}", format_duration_secs(t)));
        }
        if let Some(t) = b.time_to_full_secs {
            line.push_str(&format!("  ·  full in {}", format_duration_secs(t)));
        }
        line.push_str(&format_battery_health(b.cycle_count, b.health_percent));
        lines.push(Line::from(line));
    } else {
        lines.push(Line::from("Battery: n/a"));
    }

    let shown = selected_components(components, MAX_COMPONENTS);
    for c in &shown {
        match c.temperature_c {
            Some(t) => lines.push(Line::from(format!("{}: {:.1}°C", c.label, t))),
            None => lines.push(Line::from(format!("{}: n/a", c.label))),
        }
    }
    if components.len() > shown.len() {
        lines.push(Line::from(format!(
            "+{} more",
            components.len() - shown.len()
        )));
    }
    for f in fans {
        lines.push(Line::from(format!("{}: {} RPM", f.label, f.rpm)));
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(label: &str) -> ComponentInfo {
        ComponentInfo {
            label: label.into(),
            temperature_c: Some(50.0),
        }
    }

    #[test]
    fn prefers_cpu_gpu_labels() {
        let comps = vec![c("PMU tdie"), c("CPU Proximity"), c("GPU"), c("Battery")];
        let sel = selected_components(&comps, 2);
        assert_eq!(sel.len(), 2);
        assert_eq!(sel[0].label, "CPU Proximity");
        assert_eq!(sel[1].label, "GPU");
    }

    #[test]
    fn caps_at_max() {
        let comps = vec![c("a"), c("b"), c("c"), c("d"), c("e")];
        let sel = selected_components(&comps, 3);
        assert_eq!(sel.len(), 3);
    }

    #[test]
    fn empty_is_ok() {
        assert!(selected_components(&[], 4).is_empty());
    }

    #[test]
    fn battery_health_omits_missing_values() {
        assert_eq!(format_battery_health(None, None), "");
    }

    #[test]
    fn battery_health_formats_cycles_and_health() {
        assert_eq!(
            format_battery_health(Some(123), Some(95.0)),
            "  ·  cycles: 123  ·  health: 95%"
        );
    }

    #[test]
    fn battery_health_formats_cycles_only() {
        assert_eq!(format_battery_health(Some(10), None), "  ·  cycles: 10");
    }
}
