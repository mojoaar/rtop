use crate::data::snapshot::Snapshot;
use crate::theme::Theme;
use ratatui::Frame;

pub fn render(frame: &mut Frame, snapshot: &Snapshot, theme: &Theme) {
    let area = frame.area();
    let bg = ratatui::widgets::Block::default()
        .style(ratatui::style::Style::default().bg(theme.colors.bg).fg(theme.colors.fg));
    frame.render_widget(bg, area);

    let msg = format!(
        "rtop — {} cores | {} processes | {:.0}% cpu",
        snapshot.cpu.per_core.len(),
        snapshot.processes.len(),
        snapshot.cpu.global_usage
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(msg)
            .style(ratatui::style::Style::default().fg(theme.colors.text)),
        area,
    );
}
