use crate::data::format::human_bytes;
use crate::data::snapshot::MemorySnapshot;
use crate::theme::Theme;
use crate::ui::widgets::{block_bar, fullness_color};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, mem: &MemorySnapshot, spark: &[u64], theme: &Theme) {
    let block = Block::bordered()
        .title(" Memory ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let ratio = if mem.total == 0 {
        0.0
    } else {
        mem.used as f64 / mem.total as f64
    };
    let pct = ratio * 100.0;
    let color = fullness_color(pct, theme);
    let free = mem.total.saturating_sub(mem.used);

    let has_swap = mem.swap_total > 0;
    let constraints: Vec<Constraint> = if has_swap {
        vec![
            Constraint::Length(1), // active bar
            Constraint::Length(1), // spacing
            Constraint::Length(1), // history sparkline
            Constraint::Min(0),
            Constraint::Length(1), // stats text
            Constraint::Length(1), // swap
        ]
    } else {
        vec![
            Constraint::Length(1), // active bar
            Constraint::Length(1), // spacing
            Constraint::Length(1), // history sparkline
            Constraint::Min(0),
            Constraint::Length(1), // stats text
        ]
    };
    let areas = Layout::vertical(constraints).split(inner);

    let [active_label, active_chart] =
        Layout::horizontal([Constraint::Length(7), Constraint::Min(0)]).areas(areas[0]);
    frame.render_widget(
        Paragraph::new("Active").style(Style::default().fg(theme.colors.muted)),
        active_label,
    );
    let bar = block_bar(ratio, active_chart.width as usize);
    frame.render_widget(
        Paragraph::new(bar).style(Style::default().fg(color)),
        active_chart,
    );

    let [history_label, history_chart] =
        Layout::horizontal([Constraint::Length(7), Constraint::Min(0)]).areas(areas[2]);
    frame.render_widget(
        Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
        history_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(spark)
            .style(Style::default().fg(color)),
        history_chart,
    );

    let values = format!(
        "used {} · free {} · {:.0}%",
        human_bytes(mem.used),
        human_bytes(free),
        pct
    );
    frame.render_widget(
        Paragraph::new(values)
            .style(Style::default().fg(theme.colors.muted))
            .alignment(Alignment::Right),
        areas[4],
    );

    if has_swap {
        let swap_line = Line::from(vec![
            Span::styled("swap ", Style::default().fg(theme.colors.muted)),
            Span::styled(
                format!(
                    "{} / {}",
                    human_bytes(mem.swap_used),
                    human_bytes(mem.swap_total)
                ),
                Style::default().fg(theme.colors.warning),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(swap_line).alignment(Alignment::Right),
            areas[5],
        );
    }
}
