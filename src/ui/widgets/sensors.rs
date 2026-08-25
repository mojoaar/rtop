use crate::data::format::format_duration_secs;
use crate::data::snapshot::{BatteryInfo, ComponentInfo, FanInfo};
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

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
        lines.push(Line::from(line));
    } else {
        lines.push(Line::from("Battery: n/a"));
    }
    for c in components {
        match c.temperature_c {
            Some(t) => lines.push(Line::from(format!("{}: {:.1}°C", c.label, t))),
            None => lines.push(Line::from(format!("{}: n/a", c.label))),
        }
    }
    for f in fans {
        lines.push(Line::from(format!("{}: {} RPM", f.label, f.rpm)));
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}
