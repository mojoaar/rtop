use crate::data::format::{human_bytes, human_rate};
use crate::data::snapshot::NetRate;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Paragraph, Sparkline};
use ratatui::Frame;

pub fn active_interface(network: &[NetRate]) -> Option<&str> {
    network
        .iter()
        .filter(|n| !n.name.starts_with("lo"))
        .max_by(|a, b| {
            let ta = a.rx_bytes_per_sec + a.tx_bytes_per_sec;
            let tb = b.rx_bytes_per_sec + b.tx_bytes_per_sec;
            ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|n| n.name.as_str())
}

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
    let iface = active_interface(network).unwrap_or("n/a");
    let title = if wan_enabled {
        let wan = wan_ip.unwrap_or("n/a");
        format!(" Network · {iface} · prv {private} · wan {wan} ")
    } else {
        format!(" Network · {iface} · prv {private} ")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(name: &str, rx: f64, tx: f64) -> NetRate {
        NetRate {
            name: name.to_string(),
            rx_bytes_per_sec: rx,
            tx_bytes_per_sec: tx,
        }
    }

    #[test]
    fn active_interface_empty_is_none() {
        assert_eq!(active_interface(&[]), None);
    }

    #[test]
    fn active_interface_loopback_only_is_none() {
        let rates = [rate("lo0", 100.0, 50.0)];
        assert_eq!(active_interface(&rates), None);
    }

    #[test]
    fn active_interface_picks_busiest() {
        let rates = [rate("en0", 100.0, 50.0), rate("en1", 10.0, 10.0)];
        assert_eq!(active_interface(&rates), Some("en0"));
    }

    #[test]
    fn active_interface_excludes_loopback() {
        let rates = [rate("lo0", 9999.0, 9999.0), rate("en0", 100.0, 50.0)];
        assert_eq!(active_interface(&rates), Some("en0"));
    }
}
