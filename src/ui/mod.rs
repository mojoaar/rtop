mod widgets;

use crate::data::snapshot::Snapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

pub fn render(frame: &mut Frame, snapshot: &Snapshot, theme: &Theme) {
    let area = frame.area();
    let bg = ratatui::widgets::Block::default()
        .style(ratatui::style::Style::default().bg(theme.colors.bg).fg(theme.colors.fg));
    frame.render_widget(bg, area);

    let [cpu_area, mem_area, gpu_area, net_area, disk_area, sensors_area] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(3),
        Constraint::Min(3),
        Constraint::Length(6),
    ])
    .areas(area);

    widgets::cpu::render(frame, cpu_area, &snapshot.cpu, theme);
    widgets::memory::render(frame, mem_area, &snapshot.memory, theme);
    widgets::gpu::render(frame, gpu_area, snapshot.gpu.as_ref(), theme);
    widgets::network::render(frame, net_area, &snapshot.network, theme);
    widgets::disk::render(frame, disk_area, &snapshot.disks, theme);
    widgets::sensors::render(
        frame,
        sensors_area,
        snapshot.battery.as_ref(),
        &snapshot.components,
        &snapshot.fans,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::snapshot::{CpuSnapshot, MemorySnapshot};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn theme() -> Theme {
        crate::theme::catppuccin::get("mocha").unwrap()
    }

    #[test]
    fn renders_without_panicking_on_empty_snapshot() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render(f, &Snapshot::default(), &theme()))
            .unwrap();
    }

    #[test]
    fn cpu_bar_reflects_usage() {
        let mut snap = Snapshot::default();
        snap.cpu = CpuSnapshot {
            global_usage: 50.0,
            per_core: vec![50.0, 25.0],
            load_avg: None,
        };
        snap.memory = MemorySnapshot {
            total: 1024,
            used: 512,
            ..Default::default()
        };
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render(f, &snap, &theme()))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let text: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("CPU"));
        assert!(text.contains("Memory"));
    }
}
