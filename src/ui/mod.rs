pub mod widgets;

use crate::data::format::{format_duration_secs, human_bytes};
use crate::data::history::History;
use crate::data::snapshot::{ProcessInfo, Snapshot};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Frame;

#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    snapshot: &Snapshot,
    theme: &Theme,
    selected: Option<usize>,
    history: &History,
    processes: &[ProcessInfo],
    scroll: usize,
    filter: &str,
    filtering: bool,
    detail: Option<&ProcessInfo>,
    show_help: bool,
    transparent: bool,
    proc_rect: &mut Rect,
) {
    let area = frame.area();

    if !transparent {
        let bg = Block::default().style(Style::default().bg(theme.colors.bg));
        frame.render_widget(bg, area);
    }

    let [cpu_area, mem_gpu_area, net_area, disk_sensors_area, proc_area, help_area] =
        Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Min(5),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .areas(area);

    let [mem_area, gpu_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(mem_gpu_area);
    let [disk_area, sensors_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(disk_sensors_area);

    *proc_rect = proc_area;

    widgets::cpu::render(frame, cpu_area, &snapshot.cpu, &history.cpu_series(), theme);
    widgets::memory::render(frame, mem_area, &snapshot.memory, theme);
    widgets::gpu::render(frame, gpu_area, snapshot.gpu.as_ref(), &history.gpu_series(), theme);
    widgets::network::render(
        frame,
        net_area,
        &snapshot.network,
        &history.net_rx_series(),
        &history.net_tx_series(),
        theme,
    );
    widgets::disk::render(frame, disk_area, &snapshot.disks, theme);
    widgets::sensors::render(
        frame,
        sensors_area,
        snapshot.battery.as_ref(),
        &snapshot.components,
        &snapshot.fans,
        theme,
    );
    widgets::processes::render(frame, proc_area, processes, selected, scroll, theme);

    let [keys_area, clock_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(24)]).areas(help_area);

    let (footer, footer_style) = if filtering {
        (format!("filter: {filter}|"), Style::default().fg(theme.colors.warning))
    } else if !filter.is_empty() {
        (
            format!("filter: {filter}  ·  q quit · Enter details · ? help"),
            Style::default().fg(theme.colors.warning),
        )
    } else {
        (
            "q quit · t theme · c/m/p/n sort · ↑↓ select · k kill · f filter · Enter details · ? help"
                .to_string(),
            Style::default().fg(theme.colors.muted),
        )
    };
    frame.render_widget(Paragraph::new(footer).style(footer_style), keys_area);

    let clock = format!(
        "{} · up {}",
        chrono::Local::now().format("%H:%M:%S"),
        format_duration_secs(snapshot.uptime)
    );
    frame.render_widget(
        Paragraph::new(clock)
            .style(Style::default().fg(theme.colors.muted))
            .alignment(Alignment::Right),
        clock_area,
    );

    if let Some(p) = detail {
        render_detail(frame, area, p, theme);
    }
    if show_help {
        render_help(frame, area, theme);
    }
}

fn render_detail(frame: &mut Frame, area: Rect, p: &ProcessInfo, theme: &Theme) {
    let lines = vec![
        Line::from(format!("PID: {}", p.pid)),
        Line::from(format!("Name: {}", p.name)),
        Line::from(format!("User: {}", p.user)),
        Line::from(format!("CPU: {:.1}%", p.cpu_usage)),
        Line::from(format!("Memory: {}", human_bytes(p.memory_bytes))),
        Line::from(format!("CPU time: {}", format_duration_secs(p.cpu_time))),
        Line::from(format!(
            "Threads: {}",
            p.threads
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        )),
        Line::from(format!("State: {}", p.status)),
    ];
    let width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let popup = centered_rect(width, height, area);
    let block = Block::bordered()
        .title(" Process ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.accent));
    frame.render_widget(Clear, popup);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}

fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let banner = [
        " ____ _____ ___  ____ ",
        "|  _ \\_   _/ _ \\|  _ \\",
        "| |_) || || | | | |_) |",
        "|  _ < | || |_| |  __/",
        "|_| \\_\\|_| \\___/|_|   ",
    ];

    let keys: [(&str, &str); 10] = [
        ("q", "quit"),
        ("t", "cycle theme"),
        ("c/m/p/n", "sort cpu/mem/pid/name"),
        ("↑ / ↓", "move selection"),
        ("k", "kill selected process"),
        ("f", "filter by name or pid"),
        ("Enter", "process details"),
        ("?", "this help"),
        ("Esc", "close / cancel"),
        ("mouse", "click = select · scroll = move"),
    ];

    let mut lines: Vec<Line> = banner
        .iter()
        .map(|b| Line::from(*b).style(Style::default().fg(theme.colors.accent)))
        .collect();
    lines.push(Line::from(""));

    for (key, action) in keys {
        lines.push(Line::from(vec![
            ratatui::text::Span::styled(
                format!("{key:<10}"),
                Style::default().fg(theme.colors.accent),
            ),
            ratatui::text::Span::styled(action, Style::default().fg(theme.colors.text)),
        ]));
    }

    let width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let popup = centered_rect(width, height, area);
    let block = Block::bordered()
        .title(" Help ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.accent));
    frame.render_widget(Clear, popup);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect {
        x,
        y,
        width: w.min(area.width),
        height: h.min(area.height),
    }
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

    fn history() -> History {
        History::new(1)
    }

    fn draw(frame: &mut Frame, snap: &Snapshot, t: &Theme, history: &History) {
        let mut proc_rect = Rect::default();
        render(
            frame,
            snap,
            t,
            None,
            history,
            &snap.processes,
            0,
            "",
            false,
            None,
            false,
            false,
            &mut proc_rect,
        );
    }

    #[test]
    fn renders_without_panicking_on_empty_snapshot() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| draw(f, &Snapshot::default(), &theme(), &history()))
            .unwrap();
    }

    #[test]
    fn cpu_bar_reflects_usage() {
        let mut snap = Snapshot::default();
        snap.cpu = CpuSnapshot {
            global_usage: 50.0,
            per_core: vec![50.0, 25.0],
            ..Default::default()
        };
        snap.memory = MemorySnapshot {
            total: 1024,
            used: 512,
            ..Default::default()
        };
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| draw(f, &snap, &theme(), &history()))
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
