use crate::data::format::human_rate;
use crate::data::snapshot::NetRate;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    network: &[NetRate],
    rx_spark: &[u64],
    tx_spark: &[u64],
    theme: &Theme,
) {
    let block = Block::bordered()
        .title("Network")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [spark_area, lines_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    let [rx_area, tx_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(spark_area);

    frame.render_widget(
        Sparkline::default()
            .data(rx_spark)
            .style(Style::default().fg(theme.colors.success)),
        rx_area,
    );
    frame.render_widget(
        Sparkline::default()
            .data(tx_spark)
            .style(Style::default().fg(theme.colors.warning)),
        tx_area,
    );

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
        lines_area,
    );
}
