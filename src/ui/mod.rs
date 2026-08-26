pub mod widgets;

use crate::data::format::{format_duration_secs, human_bytes, human_rate};
use crate::data::history::{History, ProcessHistory};
use crate::data::snapshot::{NetRate, ProcessInfo, Snapshot};
use crate::theme::Theme;
use crate::ui::widgets::processes::{SortDir, SortKey};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Sparkline};
use ratatui::Frame;
use std::collections::HashMap;

pub struct RenderContext<'a> {
    pub snapshot: &'a Snapshot,
    pub theme: &'a Theme,
    pub selected: Option<usize>,
    pub history: &'a History,
    pub processes: &'a [ProcessInfo],
    pub order: &'a [usize],
    pub scroll: usize,
    pub filter: &'a str,
    pub filtering: bool,
    pub detail: Option<&'a ProcessInfo>,
    pub show_help: bool,
    pub show_settings: bool,
    pub settings_index: usize,
    pub show_signal: bool,
    pub signal_index: usize,
    pub interval_ms: u64,
    pub transparent: bool,
    pub show_time: bool,
    pub show_uptime: bool,
    pub show_labels: bool,
    pub fullscreen: bool,
    pub net_detail: bool,
    pub frozen: bool,
    pub wan_enabled: bool,
    pub private_ip: Option<&'a str>,
    pub wan_ip: Option<&'a str>,
    pub settings_editing: bool,
    pub wan_url: &'a str,
    pub wan_url_edit: &'a str,
    pub total: usize,
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
    pub proc_history: &'a HashMap<u32, ProcessHistory>,
}

pub fn render(frame: &mut Frame, ctx: &RenderContext<'_>) -> Rect {
    let RenderContext {
        snapshot,
        theme,
        selected,
        history,
        processes,
        order,
        scroll,
        filter,
        filtering,
        detail,
        show_help,
        show_settings,
        settings_index,
        show_signal,
        signal_index,
        interval_ms,
        transparent,
        show_time,
        show_uptime,
        show_labels,
        fullscreen,
        net_detail,
        frozen,
        wan_enabled,
        private_ip,
        wan_ip,
        settings_editing,
        wan_url,
        wan_url_edit,
        total,
        sort_key,
        sort_dir,
        proc_history,
    } = *ctx;
    let area = frame.area();
    let proc_rect: Rect;

    if !transparent {
        let bg = Block::default().style(Style::default().bg(theme.colors.bg));
        frame.render_widget(bg, area);
    }

    if fullscreen {
        let [proc_area, help_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
        proc_rect = proc_area;
        widgets::processes::render(
            frame,
            proc_area,
            processes,
            order,
            &widgets::processes::ProcessView {
                selected,
                scroll,
                total,
                sort_key,
                sort_dir,
            },
            theme,
        );
        let (footer_text, footer_color) = if frozen {
            (
                "PAUSED · space resume · z back · q quit".to_string(),
                theme.colors.warning,
            )
        } else if filtering {
            (format!("filter: {}|", filter), theme.colors.warning)
        } else if !filter.is_empty() {
            (
                format!(
                    "filter: {} · z back · q quit · Enter details · ? help",
                    filter
                ),
                theme.colors.warning,
            )
        } else {
            (
                "z back · q quit · c/m/p/n sort · ↑↓ select · k kill · f filter · s settings · Enter details · ? help"
                    .to_string(),
                theme.colors.muted,
            )
        };
        frame.render_widget(
            Paragraph::new(footer_text).style(Style::default().fg(footer_color)),
            help_area,
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
                &SettingsView {
                    index: settings_index,
                    interval_ms,
                    transparent,
                    show_time,
                    show_uptime,
                    show_labels,
                    wan_enabled,
                    settings_editing,
                    wan_url,
                    wan_url_edit,
                },
            );
        }
        if show_signal {
            render_signal(
                frame,
                area,
                theme,
                signal_index,
                selected
                    .and_then(|i| order.get(i))
                    .and_then(|&idx| processes.get(idx)),
            );
        }
        if net_detail {
            render_net_detail(frame, area, theme, &snapshot.network);
        }
        return proc_rect;
    }

    let core_rows = snapshot.cpu.per_core.len().max(1) as u16;
    let cpu_height = (core_rows + 2).clamp(6, 14);
    let [cpu_area, mem_gpu_area, net_area, disk_sensors_area, proc_area, help_area] =
        Layout::vertical([
            Constraint::Length(cpu_height),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

    let [mem_area, gpu_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(mem_gpu_area);
    let [disk_area, sensors_area] =
        Layout::horizontal([Constraint::Ratio(1, 1), Constraint::Ratio(1, 1)])
            .areas(disk_sensors_area);

    proc_rect = proc_area;

    widgets::cpu::render(
        frame,
        cpu_area,
        &snapshot.cpu,
        &history.cpu_series(),
        show_labels,
        theme,
    );
    widgets::memory::render(
        frame,
        mem_area,
        &snapshot.memory,
        &history.mem_series(),
        show_labels,
        theme,
    );
    widgets::gpu::render(
        frame,
        gpu_area,
        snapshot.gpu.as_ref(),
        &history.gpu_series(),
        show_labels,
        theme,
    );
    widgets::network::render(
        frame,
        net_area,
        &widgets::network::NetworkView {
            network: &snapshot.network,
            rx_spark: &history.net_rx_series(),
            tx_spark: &history.net_tx_series(),
            total_received: snapshot.net_total_received,
            total_transmitted: snapshot.net_total_transmitted,
            private_ip,
            wan_ip,
            wan_enabled,
        },
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
        order,
        &widgets::processes::ProcessView {
            selected,
            scroll,
            total,
            sort_key,
            sort_dir,
        },
        theme,
    );

    let [keys_area, clock_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(44)]).areas(help_area);

    let (footer, footer_style) = if filtering {
        (
            format!("filter: {filter}|"),
            Style::default().fg(theme.colors.warning),
        )
    } else if !filter.is_empty() {
        (
            format!("filter: {filter}  ·  q quit · Enter details · ? help"),
            Style::default().fg(theme.colors.warning),
        )
    } else {
        (
            "q quit · t theme · z fullscreen · c/m/p/n sort · ↑↓ select · k kill · f filter · s settings · ? help"
                .to_string(),
            Style::default().fg(theme.colors.muted),
        )
    };
    frame.render_widget(Paragraph::new(footer).style(footer_style), keys_area);

    let now = chrono::Local::now();
    let mut parts: Vec<String> = Vec::new();
    if frozen {
        parts.push("PAUSED".to_string());
    }
    if show_time {
        let tz = now.format("%Z").to_string();
        parts.push(if tz.is_empty() {
            now.format("%H:%M:%S").to_string()
        } else {
            format!("{} {}", tz, now.format("%H:%M:%S"))
        });
    }
    if show_uptime {
        parts.push(format!("up {}", format_duration_secs(snapshot.uptime)));
    }
    parts.push(format!("{}ms", interval_ms));
    let clock = parts.join(" · ");
    let clock_style = if frozen {
        Style::default().fg(theme.colors.warning)
    } else {
        Style::default().fg(theme.colors.muted)
    };
    frame.render_widget(
        Paragraph::new(clock)
            .style(clock_style)
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
            &SettingsView {
                index: settings_index,
                interval_ms,
                transparent,
                show_time,
                show_uptime,
                show_labels,
                wan_enabled,
                settings_editing,
                wan_url,
                wan_url_edit,
            },
        );
    }
    if show_signal {
        render_signal(
            frame,
            area,
            theme,
            signal_index,
            selected
                .and_then(|i| order.get(i))
                .and_then(|&idx| processes.get(idx)),
        );
    }
    if net_detail {
        render_net_detail(frame, area, theme, &snapshot.network);
    }
    proc_rect
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let wc = word.chars().count();
        if current.is_empty() {
            if wc <= width {
                current.push_str(word);
            } else {
                let mut rest = word.to_string();
                while !rest.is_empty() {
                    let take: String = rest.chars().take(width).collect();
                    lines.push(take);
                    rest = rest.chars().skip(width).collect();
                }
            }
        } else if current.chars().count() + 1 + wc <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            if wc <= width {
                current.push_str(word);
            } else {
                let mut rest = word.to_string();
                while !rest.is_empty() {
                    let take: String = rest.chars().take(width).collect();
                    lines.push(take);
                    rest = rest.chars().skip(width).collect();
                }
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn render_detail(
    frame: &mut Frame,
    area: Rect,
    p: &ProcessInfo,
    history: Option<&ProcessHistory>,
    theme: &Theme,
) {
    let info_lines = vec![
        Line::from(format!("PID: {}", p.pid)),
        Line::from(format!("Name: {}", p.name)),
        Line::from(format!("User: {}", p.user)),
        Line::from(format!("CPU: {:.1}%", p.cpu_usage)),
        Line::from(format!(
            "Memory: {} ({} KB)",
            human_bytes(p.memory_bytes),
            p.memory_bytes / 1024
        )),
        Line::from(format!("CPU time: {}", format_duration_secs(p.cpu_time))),
        Line::from(format!(
            "Threads: {}",
            p.threads
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        )),
        Line::from(format!("State: {}", p.status)),
    ];
    let info_w = info_lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let width = info_w.max(44).min(area.width.saturating_sub(4).max(20));

    let cmd_text = format!("Cmd: {}", p.cmd);
    let text_width = (width.saturating_sub(2)).max(1) as usize;
    let mut cmd_lines = wrap(&cmd_text, text_width);
    if cmd_lines.len() > 3 {
        cmd_lines.truncate(3);
        if let Some(last) = cmd_lines.last_mut() {
            last.push('…');
        }
    }
    let n_cmd = cmd_lines.len().max(1) as u16;

    let height = info_lines.len() as u16 + n_cmd + 2 + 4;
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

    let [info_area, cmd_area, cpu_spark_area, mem_spark_area] = Layout::vertical([
        Constraint::Length(info_lines.len() as u16),
        Constraint::Length(n_cmd),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(info_lines).style(Style::default().fg(theme.colors.text)),
        info_area,
    );

    let cmd_paragraph_lines: Vec<Line> = cmd_lines.into_iter().map(Line::from).collect();
    frame.render_widget(
        Paragraph::new(cmd_paragraph_lines).style(Style::default().fg(theme.colors.text)),
        cmd_area,
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

    let keys: [(&str, &str); 12] = [
        ("q", "quit"),
        ("t", "cycle theme"),
        ("s", "settings"),
        ("z", "full-screen processes"),
        ("c/m/p/n", "sort cpu/mem/pid/name"),
        ("↑ / ↓", "move selection"),
        ("k", "kill selected process"),
        ("f", "filter by name / user / pid"),
        ("Enter", "process details"),
        ("?", "this help"),
        ("Esc", "close / cancel"),
        ("mouse", "click = select · scroll = move"),
    ];

    let footer = [
        format!("rtop v{}", env!("CARGO_PKG_VERSION")),
        "repo: https://github.com/mojoaar/rtop".to_string(),
        "author: Morten Johansen — https://johansen.foo".to_string(),
    ];

    let content_width = keys
        .iter()
        .map(|(_, a)| 10 + a.len())
        .max()
        .unwrap_or(18)
        .max(18)
        .max(banner.iter().map(|b| b.chars().count()).max().unwrap_or(0))
        .max(footer.iter().map(|l| l.chars().count()).max().unwrap_or(0));

    let banner_max = banner.iter().map(|b| b.chars().count()).max().unwrap_or(0);
    let mut lines: Vec<Line> = banner
        .iter()
        .map(|b| {
            let normalized = format!("{:<bw$}", b, bw = banner_max);
            Line::from(format!("{:^width$}", normalized, width = content_width)).style(
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
    for line in &footer {
        lines.push(Line::from(line.clone()).style(Style::default().fg(theme.colors.muted)));
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

struct SettingsView<'a> {
    index: usize,
    interval_ms: u64,
    transparent: bool,
    show_time: bool,
    show_uptime: bool,
    show_labels: bool,
    wan_enabled: bool,
    settings_editing: bool,
    wan_url: &'a str,
    wan_url_edit: &'a str,
}

fn render_settings(frame: &mut Frame, area: Rect, theme: &Theme, settings: &SettingsView<'_>) {
    let SettingsView {
        index,
        interval_ms,
        transparent,
        show_time,
        show_uptime,
        show_labels,
        wan_enabled,
        settings_editing,
        wan_url,
        wan_url_edit,
    } = *settings;
    let url_value = if settings_editing {
        format!("{wan_url_edit}|")
    } else {
        wan_url.to_string()
    };
    let rows = [
        ("Refresh", format!("{}ms", interval_ms)),
        ("Theme", theme.name.clone()),
        (
            "Transparent",
            if transparent {
                "on".into()
            } else {
                "off".into()
            },
        ),
        ("Time", if show_time { "on".into() } else { "off".into() }),
        (
            "Uptime",
            if show_uptime {
                "on".into()
            } else {
                "off".into()
            },
        ),
        (
            "Labels",
            if show_labels {
                "on".into()
            } else {
                "off".into()
            },
        ),
        (
            "WAN IP",
            if wan_enabled {
                "on".into()
            } else {
                "off".into()
            },
        ),
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

fn render_signal(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    index: usize,
    target: Option<&ProcessInfo>,
) {
    let signals = ["Term (SIGTERM)", "Kill (SIGKILL)", "Interrupt (SIGINT)"];
    let mut lines: Vec<Line> = signals
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let selected = i == index;
            let fg = if selected {
                theme.colors.danger
            } else {
                theme.colors.text
            };
            let mut line = Line::from(label.to_string()).style(Style::default().fg(fg));
            if selected {
                line = line.style(Style::default().bg(theme.colors.highlight));
            }
            line
        })
        .collect();
    if let Some(p) = target {
        lines.insert(0, Line::from(""));
        lines.insert(
            0,
            Line::from(format!("Kill {} (pid {})?", p.name, p.pid))
                .style(Style::default().fg(theme.colors.text)),
        );
    }
    lines.push(Line::from(""));
    lines.push(
        Line::from("↑ / ↓ select  ·  Enter confirm  ·  Esc cancel")
            .style(Style::default().fg(theme.colors.muted)),
    );

    let width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let popup = centered_rect(width, height, area);
    let block = Block::bordered()
        .title(" Signal ")
        .title_style(
            Style::default()
                .fg(theme.colors.danger)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.danger));
    frame.render_widget(Clear, popup);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        inner,
    );
}

fn render_net_detail(frame: &mut Frame, area: Rect, theme: &Theme, network: &[NetRate]) {
    let mut lines: Vec<Line> = network
        .iter()
        .map(|n| {
            Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{:<12}", n.name),
                    Style::default().fg(theme.colors.accent),
                ),
                ratatui::text::Span::styled(
                    format!("↓ {}", human_rate(n.rx_bytes_per_sec)),
                    Style::default().fg(theme.colors.success),
                ),
                ratatui::text::Span::styled(
                    format!("  ↑ {}", human_rate(n.tx_bytes_per_sec)),
                    Style::default().fg(theme.colors.warning),
                ),
            ])
        })
        .collect();
    if lines.is_empty() {
        lines.push(Line::from("no interfaces").style(Style::default().fg(theme.colors.muted)));
    }

    let width = lines.iter().map(|l| l.width()).max().unwrap_or(20) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let popup = centered_rect(width, height, area);
    let block = Block::bordered()
        .title(" Interfaces ")
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
    use crate::data::snapshot::{CpuSnapshot, MemorySnapshot, ProcessInfo};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn theme() -> Theme {
        crate::theme::catppuccin::get("mocha").unwrap()
    }

    fn history() -> History {
        History::new(1)
    }

    fn draw(frame: &mut Frame, snap: &Snapshot, t: &Theme, history: &History) {
        let proc_history = std::collections::HashMap::new();
        let order: Vec<usize> = (0..snap.processes.len()).collect();
        render(
            frame,
            &RenderContext {
                snapshot: snap,
                theme: t,
                selected: None,
                history,
                processes: &snap.processes,
                order: &order,
                scroll: 0,
                filter: "",
                filtering: false,
                detail: None,
                show_help: false,
                show_settings: false,
                settings_index: 0,
                show_signal: false,
                signal_index: 0,
                interval_ms: 1000,
                transparent: false,
                show_time: false,
                show_uptime: false,
                show_labels: false,
                fullscreen: false,
                net_detail: false,
                frozen: false,
                wan_enabled: false,
                private_ip: None,
                wan_ip: None,
                settings_editing: false,
                wan_url: "",
                wan_url_edit: "",
                total: snap.processes.len(),
                sort_key: SortKey::Cpu,
                sort_dir: SortDir::Desc,
                proc_history: &proc_history,
            },
        );
    }

    fn draw_list(
        frame: &mut Frame,
        snap: &Snapshot,
        t: &Theme,
        history: &History,
        order: &[usize],
        fullscreen: bool,
    ) {
        let proc_history = std::collections::HashMap::new();
        render(
            frame,
            &RenderContext {
                snapshot: snap,
                theme: t,
                selected: None,
                history,
                processes: &snap.processes,
                order,
                scroll: 0,
                filter: "",
                filtering: false,
                detail: None,
                show_help: false,
                show_settings: false,
                settings_index: 0,
                show_signal: false,
                signal_index: 0,
                interval_ms: 1000,
                transparent: false,
                show_time: false,
                show_uptime: false,
                show_labels: false,
                fullscreen,
                net_detail: false,
                frozen: false,
                wan_enabled: false,
                private_ip: None,
                wan_ip: None,
                settings_editing: false,
                wan_url: "",
                wan_url_edit: "",
                total: snap.processes.len(),
                sort_key: SortKey::Cpu,
                sort_dir: SortDir::Desc,
                proc_history: &proc_history,
            },
        );
    }

    #[test]
    fn wrap_splits_at_word_boundary() {
        assert_eq!(wrap("abc def ghi", 7), vec!["abc def", "ghi"]);
    }

    #[test]
    fn wrap_hard_breaks_long_word() {
        assert_eq!(wrap("abcdefgh", 4), vec!["abcd", "efgh"]);
    }

    #[test]
    fn wrap_short_fits_single_line() {
        assert_eq!(wrap("abc def", 10), vec!["abc def"]);
    }

    #[test]
    fn wrap_empty_returns_single_empty() {
        assert_eq!(wrap("", 5), vec![""]);
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
        let snap = Snapshot {
            cpu: CpuSnapshot {
                global_usage: 50.0,
                per_core: vec![50.0, 25.0],
                ..Default::default()
            },
            memory: MemorySnapshot {
                total: 1024,
                used: 512,
                ..Default::default()
            },
            ..Default::default()
        };
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| draw(f, &snap, &theme(), &history()))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let text: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("CPU"));
        assert!(text.contains("Memory"));
    }

    #[test]
    fn fullscreen_renders_without_panicking() {
        let snap = Snapshot {
            processes: vec![ProcessInfo {
                pid: 1,
                name: "foo".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| draw_list(f, &snap, &theme(), &history(), &[0usize], true))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let text: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("z back"));
    }

    #[test]
    fn renders_filtered_list_not_full_snapshot() {
        let snap = Snapshot {
            processes: vec![
                ProcessInfo {
                    pid: 1,
                    name: "keepme".into(),
                    ..Default::default()
                },
                ProcessInfo {
                    pid: 2,
                    name: "dropme".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| draw_list(f, &snap, &theme(), &history(), &[0usize], false))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let text: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("keepme"));
        assert!(!text.contains("dropme"));
    }
}
