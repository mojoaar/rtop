pub mod widgets;

use crate::data::format::{format_duration_secs, human_bytes};
use crate::data::history::{History, ProcessHistory};
use crate::data::snapshot::{ProcessInfo, Snapshot};
use crate::theme::Theme;
use crate::ui::widgets::processes::{SortDir, SortKey};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Sparkline};
use ratatui::Frame;
use std::collections::HashMap;

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
    show_settings: bool,
    settings_index: usize,
    interval_ms: u64,
    transparent: bool,
    wan_enabled: bool,
    private_ip: Option<&str>,
    wan_ip: Option<&str>,
    settings_editing: bool,
    wan_url: &str,
    wan_url_edit: &str,
    proc_rect: &mut Rect,
    total: usize,
    sort_key: SortKey,
    sort_dir: SortDir,
    proc_history: &HashMap<u32, ProcessHistory>,
) {
    let area = frame.area();

    if !transparent {
        let bg = Block::default().style(Style::default().bg(theme.colors.bg));
        frame.render_widget(bg, area);
    }

    let core_rows = snapshot.cpu.per_core.len().max(1) as u16;
    let cpu_height = (core_rows + 2).clamp(6, 14);
    let [cpu_area, mem_gpu_area, net_area, disk_sensors_area, proc_area, help_area] =
        Layout::vertical([
            Constraint::Length(cpu_height),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

    let [mem_area, gpu_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(mem_gpu_area);
    let [disk_area, sensors_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(disk_sensors_area);

    *proc_rect = proc_area;

    widgets::cpu::render(frame, cpu_area, &snapshot.cpu, &history.cpu_series(), theme);
    widgets::memory::render(frame, mem_area, &snapshot.memory, &history.mem_series(), theme);
    widgets::gpu::render(frame, gpu_area, snapshot.gpu.as_ref(), &history.gpu_series(), theme);
    widgets::network::render(
        frame,
        net_area,
        &snapshot.network,
        &history.net_rx_series(),
        &history.net_tx_series(),
        snapshot.net_total_received,
        snapshot.net_total_transmitted,
        private_ip,
        wan_ip,
        wan_enabled,
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
    widgets::processes::render(
        frame,
        proc_area,
        processes,
        selected,
        scroll,
        total,
        sort_key,
        sort_dir,
        theme,
    );

    let [keys_area, clock_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(44)]).areas(help_area);

    let (footer, footer_style) = if filtering {
        (format!("filter: {filter}|"), Style::default().fg(theme.colors.warning))
    } else if !filter.is_empty() {
        (
            format!("filter: {filter}  ·  q quit · Enter details · ? help"),
            Style::default().fg(theme.colors.warning),
        )
    } else {
        (
            "q quit · t theme · c/m/p/n sort · ↑↓ select · k kill · f filter · s settings · ? help"
                .to_string(),
            Style::default().fg(theme.colors.muted),
        )
    };
    frame.render_widget(Paragraph::new(footer).style(footer_style), keys_area);

    let now = chrono::Local::now();
    let tz = now.format("%Z").to_string();
    let clock = if tz.is_empty() {
        format!(
            "{} · up {} · {}ms",
            now.format("%H:%M:%S"),
            format_duration_secs(snapshot.uptime),
            interval_ms
        )
    } else {
        format!(
            "{} {} · up {} · {}ms",
            tz,
            now.format("%H:%M:%S"),
            format_duration_secs(snapshot.uptime),
            interval_ms
        )
    };
    frame.render_widget(
        Paragraph::new(clock)
            .style(Style::default().fg(theme.colors.muted))
            .alignment(Alignment::Right),
        clock_area,
    );

    if let Some(p) = detail {
        render_detail(frame, area, p, proc_history.get(&p.pid), theme);
    }
    if show_help {
        render_help(frame, area, theme);
    }
    if show_settings {
        render_settings(
            frame,
            area,
            theme,
            settings_index,
            interval_ms,
            transparent,
            wan_enabled,
            settings_editing,
            wan_url,
            wan_url_edit,
        );
    }
}

fn render_detail(
    frame: &mut Frame,
    area: Rect,
    p: &ProcessInfo,
    history: Option<&ProcessHistory>,
    theme: &Theme,
) {
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
    let info_w = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let width = info_w.max(44);
    let height = lines.len() as u16 + 2 + 2 + 2;
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

    let [info_area, cpu_spark_area, mem_spark_area] = Layout::vertical([
        Constraint::Length(lines.len() as u16),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        info_area,
    );

    let cpu_series = history.map(|h| h.cpu_series()).unwrap_or_default();
    let [cpu_label, cpu_spark] =
        Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).areas(cpu_spark_area);
    frame.render_widget(
        Paragraph::new("CPU history").style(Style::default().fg(theme.colors.muted)),
        cpu_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(cpu_series)
            .max(100)
            .style(Style::default().fg(theme.colors.accent)),
        cpu_spark,
    );

    let mem_series = history.map(|h| h.mem_series()).unwrap_or_default();
    let [mem_label, mem_spark] =
        Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).areas(mem_spark_area);
    frame.render_widget(
        Paragraph::new("Mem history").style(Style::default().fg(theme.colors.muted)),
        mem_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(mem_series)
            .style(Style::default().fg(theme.colors.success)),
        mem_spark,
    );
}

fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let banner = [
        " ██████╗ ████████╗  ██████╗ ██████╗",
        " ██╔══██╗╚══██╔══╝ ██╔═══██╗██╔══██╗",
        " ██████╔╝   ██║    ██║   ██║██████╔╝",
        " ██╔══██╗   ██║    ██║   ██║██╔═══╝",
        " ██║  ██║   ██║    ╚██████╔╝██║",
        " ╚═╝  ╚═╝   ╚═╝     ╚═════╝ ╚═╝",
    ];

    let keys: [(&str, &str); 11] = [
        ("q", "quit"),
        ("t", "cycle theme"),
        ("s", "settings"),
        ("c/m/p/n", "sort cpu/mem/pid/name"),
        ("↑ / ↓", "move selection"),
        ("k", "kill selected process"),
        ("f", "filter by name or pid"),
        ("Enter", "process details"),
        ("?", "this help"),
        ("Esc", "close / cancel"),
        ("mouse", "click = select · scroll = move"),
    ];

    let content_width = keys
        .iter()
        .map(|(_, a)| 10 + a.len())
        .max()
        .unwrap_or(18)
        .max(18)
        .max(banner.iter().map(|b| b.chars().count()).max().unwrap_or(0));

    let mut lines: Vec<Line> = banner
        .iter()
        .map(|b| {
            Line::from(format!("{:^width$}", b, width = content_width)).style(
                Style::default()
                    .fg(theme.colors.accent)
                    .add_modifier(Modifier::BOLD),
            )
        })
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

    lines.push(Line::from(""));
    lines.push(
        Line::from(format!("rtop v{}", env!("CARGO_PKG_VERSION")))
            .style(Style::default().fg(theme.colors.muted)),
    );
    lines.push(
        Line::from("repo: https://github.com/mojoaar/rtop")
            .style(Style::default().fg(theme.colors.muted)),
    );
    lines.push(
        Line::from("author: Morten Johansen — https://johansen.foo")
            .style(Style::default().fg(theme.colors.muted)),
    );

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

fn render_settings(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    index: usize,
    interval_ms: u64,
    transparent: bool,
    wan_enabled: bool,
    settings_editing: bool,
    wan_url: &str,
    wan_url_edit: &str,
) {
    let url_value = if settings_editing {
        format!("{wan_url_edit}|")
    } else {
        wan_url.to_string()
    };
    let rows = [
        ("Refresh", format!("{}ms", interval_ms)),
        ("Theme", theme.name.clone()),
        ("Transparent", if transparent { "on".into() } else { "off".into() }),
        ("WAN IP", if wan_enabled { "on".into() } else { "off".into() }),
        ("WAN URL", url_value),
    ];

    let mut lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let selected = i == index;
            let fg = if selected {
                theme.colors.accent
            } else {
                theme.colors.text
            };
            let value_fg = if selected {
                theme.colors.accent
            } else {
                theme.colors.muted
            };
            let mut line = Line::from(vec![
                ratatui::text::Span::styled(format!("{label:<12}"), Style::default().fg(fg)),
                ratatui::text::Span::styled(format!("  {value}  "), Style::default().fg(value_fg)),
            ]);
            if selected {
                line = line.style(Style::default().bg(theme.colors.highlight));
            }
            line
        })
        .collect();
    lines.push(Line::from(""));
    lines.push(
        Line::from("← / → change  ·  ↑ / ↓ select  ·  Enter edit  ·  Esc close")
            .style(Style::default().fg(theme.colors.muted)),
    );

    let width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let popup = centered_rect(width, height, area);
    let block = Block::bordered()
        .title(" Settings ")
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
        let proc_history = std::collections::HashMap::new();
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
            0,
            1000,
            false,
            false,
            None,
            None,
            false,
            "",
            "",
            &mut proc_rect,
            snap.processes.len(),
            SortKey::Cpu,
            SortDir::Desc,
            &proc_history,
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
