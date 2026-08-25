use crate::data::snapshot::{BatteryInfo, ComponentInfo, FanInfo};
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
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
        .title("Battery & Sensors")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(b) = battery {
        lines.push(Line::from(format!("Battery: {:.0}% ({})", b.percentage, b.state)));
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
