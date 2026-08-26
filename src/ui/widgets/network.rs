use crate::data::format::{human_bytes, human_rate};
use crate::data::snapshot::NetRate;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Paragraph, Sparkline};
use ratatui::Frame;

pub struct NetworkView<'a> {
    pub network: &'a [NetRate],
    pub rx_spark: &'a [u64],
    pub tx_spark: &'a [u64],
    pub total_received: u64,
    pub total_transmitted: u64,
    pub private_ip: Option<&'a str>,
    pub wan_ip: Option<&'a str>,
    pub wan_enabled: bool,
}

pub fn render(frame: &mut Frame, area: Rect, view: &NetworkView<'_>, theme: &Theme) {
    let NetworkView {
        network,
        rx_spark,
        tx_spark,
        total_received,
        total_transmitted,
        private_ip,
        wan_ip,
        wan_enabled,
    } = *view;
    let private = private_ip.unwrap_or("n/a");
    let title = if wan_enabled {
        let wan = wan_ip.unwrap_or("n/a");
        format!(" Network · prv {private} · wan {wan} ")
    } else {
        format!(" Network · prv {private} ")
    };
    let block = Block::bordered()
        .title(title)
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [down_area, up_area, total_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

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

    frame.render_widget(
        Paragraph::new(format!(
            "total ↓ {} · ↑ {}",
            human_bytes(total_received),
            human_bytes(total_transmitted)
        ))
        .style(Style::default().fg(theme.colors.muted))
        .alignment(Alignment::Right),
        total_area,
    );
}
