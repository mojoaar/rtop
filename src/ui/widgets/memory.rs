use crate::data::format::human_bytes;
use crate::data::snapshot::MemorySnapshot;
use crate::theme::Theme;
use crate::ui::widgets::{block_bar, fullness_color};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    mem: &MemorySnapshot,
    spark: &[u64],
    show_labels: bool,
    theme: &Theme,
) {
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
    let mut constraints: Vec<Constraint> = Vec::new();
    if show_labels {
        constraints.push(Constraint::Length(1)); // active label
    }
    constraints.push(Constraint::Length(1)); // active bar
    if show_labels {
        constraints.push(Constraint::Length(1)); // history label
    }
    constraints.push(Constraint::Length(1)); // history sparkline
    constraints.push(Constraint::Min(0));
    constraints.push(Constraint::Length(1)); // stats text
    if has_swap {
        constraints.push(Constraint::Length(1)); // swap
    }
    let areas = Layout::vertical(constraints).split(inner);

    let mut idx = 0;
    if show_labels {
        frame.render_widget(
            Paragraph::new("Active").style(Style::default().fg(theme.colors.muted)),
            areas[idx],
        );
        idx += 1;
    }
    let bar = block_bar(ratio, areas[idx].width as usize);
    frame.render_widget(
        Paragraph::new(bar).style(Style::default().fg(color)),
        areas[idx],
    );
    idx += 1;
    if show_labels {
        frame.render_widget(
            Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
            areas[idx],
        );
        idx += 1;
    }
    frame.render_widget(
        Sparkline::default()
            .data(spark)
            .style(Style::default().fg(color)),
        areas[idx],
    );
    idx += 2; // skip sparkline + fill

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
        areas[idx],
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
            areas[idx + 1],
        );
    }
}
