use crate::data::format::human_rate;
use crate::data::snapshot::NetRate;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
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
        .title(" Network ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [down_area, up_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    let total_rx: f64 = network.iter().map(|n| n.rx_bytes_per_sec).sum();
    let total_tx: f64 = network.iter().map(|n| n.tx_bytes_per_sec).sum();

    let [down_label, down_spark] =
        Layout::horizontal([Constraint::Length(16), Constraint::Min(0)]).areas(down_area);
    frame.render_widget(
        Paragraph::new(format!("↓ {}", human_rate(total_rx)))
            .style(Style::default().fg(theme.colors.success)),
        down_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(rx_spark)
            .style(Style::default().fg(theme.colors.success)),
        down_spark,
    );

    let [up_label, up_spark] =
        Layout::horizontal([Constraint::Length(16), Constraint::Min(0)]).areas(up_area);
    frame.render_widget(
        Paragraph::new(format!("↑ {}", human_rate(total_tx)))
            .style(Style::default().fg(theme.colors.warning)),
        up_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(tx_spark)
            .style(Style::default().fg(theme.colors.warning)),
        up_spark,
    );
}
