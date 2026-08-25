use crate::data::format::human_bytes;
use crate::data::snapshot::MemorySnapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, mem: &MemorySnapshot, theme: &Theme) {
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

    let mem_ratio = if mem.total == 0 {
        0.0
    } else {
        mem.used as f64 / mem.total as f64
    };

    if mem.swap_total == 0 {
        let [mem_text, mem_bar] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
        frame.render_widget(
            Paragraph::new(format!(
                "Mem: {} / {} ({:.0}%)",
                human_bytes(mem.used),
                human_bytes(mem.total),
                mem_ratio * 100.0
            ))
            .style(Style::default().fg(theme.colors.text)),
            mem_text,
        );
        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(theme.colors.success))
                .ratio(mem_ratio.clamp(0.0, 1.0)),
            mem_bar,
        );
        return;
    }

    let [mem_text, mem_bar, swap_text, swap_bar] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(format!(
            "Mem: {} / {} ({:.0}%)",
            human_bytes(mem.used),
            human_bytes(mem.total),
            mem_ratio * 100.0
        ))
        .style(Style::default().fg(theme.colors.text)),
        mem_text,
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.success))
            .ratio(mem_ratio.clamp(0.0, 1.0)),
        mem_bar,
    );

    let swap_ratio = mem.swap_used as f64 / mem.swap_total as f64;
    frame.render_widget(
        Paragraph::new(format!(
            "Swap: {} / {}",
            human_bytes(mem.swap_used),
            human_bytes(mem.swap_total)
        ))
        .style(Style::default().fg(theme.colors.text)),
        swap_text,
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.warning))
            .ratio(swap_ratio.clamp(0.0, 1.0)),
        swap_bar,
    );
}
