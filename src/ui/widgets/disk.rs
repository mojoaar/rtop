use crate::data::format::human_bytes;
use crate::data::snapshot::DiskUsage;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Gauge};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, disks: &[DiskUsage], theme: &Theme) {
    let block = Block::bordered()
        .title(" Disk ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows: Vec<&DiskUsage> = disks.iter().filter(|d| d.total > 0).collect();
    if rows.is_empty() {
        return;
    }

    let constraints: Vec<Constraint> = rows.iter().map(|_| Constraint::Length(1)).collect();
    let areas = Layout::vertical(constraints).split(inner);

    for (d, a) in rows.iter().zip(areas.iter()) {
        let used = d.total.saturating_sub(d.available);
        let ratio = if d.total == 0 {
            0.0
        } else {
            used as f64 / d.total as f64
        };
        let pct = ratio * 100.0;
        let color: Color = if pct < 60.0 {
            theme.colors.success
        } else if pct < 85.0 {
            theme.colors.warning
        } else {
            theme.colors.danger
        };
        let label = format!(
            "{}  used {} · free {}  ({:.0}%)",
            d.mount_point,
            human_bytes(used),
            human_bytes(d.available),
            pct
        );
        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(color))
                .ratio(ratio.clamp(0.0, 1.0))
                .label(label),
            *a,
        );
    }
}
