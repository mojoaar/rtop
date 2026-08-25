use crate::data::format::human_bytes;
use crate::data::snapshot::MemorySnapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Gauge};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, mem: &MemorySnapshot, theme: &Theme) {
    let block = Block::bordered()
        .title("Memory")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [mem_area, swap_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    let mem_ratio = if mem.total == 0 {
        0.0
    } else {
        mem.used as f64 / mem.total as f64
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.success))
            .ratio(mem_ratio.clamp(0.0, 1.0))
            .label(format!(
                "{} / {}",
                human_bytes(mem.used),
                human_bytes(mem.total)
            )),
        mem_area,
    );

    let swap_ratio = if mem.swap_total == 0 {
        0.0
    } else {
        mem.swap_used as f64 / mem.swap_total as f64
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.warning))
            .ratio(swap_ratio.clamp(0.0, 1.0))
            .label(format!(
                "swap {} / {}",
                human_bytes(mem.swap_used),
                human_bytes(mem.swap_total)
            )),
        swap_area,
    );
}
