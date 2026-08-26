use crate::data::format::{human_bytes, human_rate};
use crate::data::snapshot::DiskUsage;
use crate::theme::Theme;
use crate::ui::widgets::fullness_color;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Cell, Row, Table};
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

    let widths = [
        Constraint::Min(8),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(8),
    ];
    let header = Row::new(vec!["MOUNT", "SIZE", "USED", "FREE", "READ", "WRITE"])
        .style(Style::default().fg(theme.colors.accent));

    let table_rows: Vec<Row> = rows
        .iter()
        .map(|d| {
            let used = d.total.saturating_sub(d.available);
            let pct = if d.total == 0 {
                0.0
            } else {
                used as f64 / d.total as f64 * 100.0
            };
            let used_color = fullness_color(pct, theme);
            Row::new(vec![
                Cell::from(d.mount_point.clone()).style(Style::default().fg(theme.colors.text)),
                Cell::from(human_bytes(d.total)).style(Style::default().fg(theme.colors.muted)),
                Cell::from(human_bytes(used)).style(Style::default().fg(used_color)),
                Cell::from(human_bytes(d.available)).style(Style::default().fg(theme.colors.muted)),
                Cell::from(human_rate(d.read_bytes_per_sec))
                    .style(Style::default().fg(theme.colors.muted)),
                Cell::from(human_rate(d.write_bytes_per_sec))
                    .style(Style::default().fg(theme.colors.muted)),
            ])
        })
        .collect();

    frame.render_widget(Table::new(table_rows, widths).header(header), inner);
}
