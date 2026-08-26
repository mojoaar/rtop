pub mod popups;
pub mod widgets;

use crate::data::format::format_duration_secs;
use crate::data::history::{History, ProcessHistory};
use crate::data::snapshot::{ProcessInfo, Snapshot};
use crate::theme::Theme;
use crate::ui::popups::SettingsView;
use crate::ui::widgets::processes::{SortDir, SortKey};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};
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
                "z back · q quit · c/m/p/n sort · ↑↓ select · k kill · f filter · i network · space pause · s settings · Enter details · ? help"
                    .to_string(),
                theme.colors.muted,
            )
        };
        frame.render_widget(
            Paragraph::new(footer_text).style(Style::default().fg(footer_color)),
            help_area,
        );
        if let Some(p) = detail {
            popups::render_detail(frame, area, p, proc_history.get(&p.pid), theme);
        }
        if show_help {
            popups::render_help(frame, area, theme);
        }
        if show_settings {
            popups::render_settings(
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
                    sort_key,
                    sort_dir,
                    wan_enabled,
                    settings_editing,
                    wan_url,
                    wan_url_edit,
                },
            );
        }
        if show_signal {
            popups::render_signal(
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
            popups::render_net_detail(frame, area, theme, &snapshot.network);
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
            "q quit · t theme · z fullscreen · c/m/p/n sort · ↑↓ select · k kill · f filter · i network · space pause · s settings · ? help"
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
        popups::render_detail(frame, area, p, proc_history.get(&p.pid), theme);
    }
    if show_help {
        popups::render_help(frame, area, theme);
    }
    if show_settings {
        popups::render_settings(
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
                sort_key,
                sort_dir,
                wan_enabled,
                settings_editing,
                wan_url,
                wan_url_edit,
            },
        );
    }
    if show_signal {
        popups::render_signal(
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
        popups::render_net_detail(frame, area, theme, &snapshot.network);
    }
    proc_rect
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
