use crate::data::format::human_rate;
use crate::data::snapshot::NetRate;
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, network: &[NetRate], theme: &Theme) {
    let block = Block::bordered()
        .title("Network")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = network
        .iter()
        .filter(|n| n.rx_bytes_per_sec > 0.0 || n.tx_bytes_per_sec > 0.0)
        .map(|n| {
            Line::from(format!(
                "{}  ↓ {}  ↑ {}",
                n.name,
                human_rate(n.rx_bytes_per_sec),
                human_rate(n.tx_bytes_per_sec)
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}
