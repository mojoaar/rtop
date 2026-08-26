use crate::data::format::human_bytes;
use crate::data::snapshot::DiskUsage;
use crate::theme::Theme;
use crate::ui::widgets::{bar_label_split, block_bar, fullness_color};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
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
        let color = fullness_color(pct, theme);
        let [text_area, bar_area] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(*a);
        let label = Line::from(vec![
            Span::styled(
                format!("{}  ", d.mount_point),
                Style::default().fg(theme.colors.muted),
            ),
            Span::styled(human_bytes(used), Style::default().fg(color)),
            Span::styled(" · ", Style::default().fg(theme.colors.muted)),
            Span::styled(
                format!("free {}", human_bytes(d.available)),
                Style::default().fg(color),
            ),
        ]);
        frame.render_widget(Paragraph::new(label), text_area);

        let pct_label = format!("{:.0}%", pct);
        let bar = block_bar(ratio, bar_area.width as usize);
        let bar_line = match bar_label_split(&bar, &pct_label) {
            Some((left, right)) => Line::from(vec![
                Span::styled(left, Style::default().fg(color)),
                Span::styled(pct_label, Style::default().fg(theme.colors.text)),
                Span::styled(right, Style::default().fg(color)),
            ]),
            None => Line::from(Span::styled(
                pct_label,
                Style::default().fg(theme.colors.text),
            )),
        };
        frame.render_widget(Paragraph::new(bar_line), bar_area);
    }
}
