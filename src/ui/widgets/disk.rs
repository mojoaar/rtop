use crate::data::format::human_bytes;
use crate::data::snapshot::DiskUsage;
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, disks: &[DiskUsage], theme: &Theme) {
    let block = Block::bordered()
        .title("Disk")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = disks
        .iter()
        .map(|d| {
            let used = d.total.saturating_sub(d.available);
            Line::from(format!(
                "{}  {} / {} used",
                d.mount_point,
                human_bytes(used),
                human_bytes(d.total)
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}
