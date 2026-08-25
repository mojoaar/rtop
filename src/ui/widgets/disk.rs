use crate::data::format::{human_bytes, human_rate};
use crate::data::snapshot::DiskUsage;
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
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

    let lines: Vec<Line> = disks
        .iter()
        .map(|d| {
            let used = d.total.saturating_sub(d.available);
            Line::from(format!(
                "{} [{}]  {} / {} used  R: {}  W: {}",
                d.mount_point,
                d.name,
                human_bytes(used),
                human_bytes(d.total),
                human_rate(d.read_bytes_per_sec),
                human_rate(d.write_bytes_per_sec)
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}
