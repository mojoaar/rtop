use crate::data::format::{format_duration_secs, human_bytes, human_rate};
use crate::data::history::ProcessHistory;
use crate::data::snapshot::{NetRate, ProcessInfo};
use crate::theme::Theme;
use crate::ui::widgets;
use crate::ui::widgets::processes::{SortDir, SortKey};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Sparkline};
use ratatui::Frame;

pub(crate) fn wrap(text: &str, width: usize) -> Vec<String> {
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

pub(crate) fn render_detail(
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

pub(crate) fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let banner = [
        " ██████╗ ████████╗  ██████╗ ██████╗",
        " ██╔══██╗╚══██╔══╝ ██╔═══██╗██╔══██╗",
        " ██████╔╝   ██║    ██║   ██║██████╔╝",
        " ██╔══██╗   ██║    ██║   ██║██╔═══╝",
        " ██║  ██║   ██║    ╚██████╔╝██║",
        " ╚═╝  ╚═╝   ╚═╝     ╚═════╝ ╚═╝",
    ];

    let keys: [(&str, &str); 14] = [
        ("↑ / ↓", "move selection"),
        ("Enter", "process details"),
        ("k", "signal menu (term / kill / int)"),
        ("f", "filter by name / user / pid"),
        ("c/m/p/n", "sort cpu/mem/pid/name"),
        ("z", "full-screen processes"),
        ("i", "network interfaces"),
        ("s", "settings"),
        ("?", "this help"),
        ("t", "cycle theme"),
        ("space", "pause / resume"),
        ("mouse", "click = select · scroll = move"),
        ("Esc", "close / cancel"),
        ("q", "quit"),
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

pub(crate) struct SettingsView<'a> {
    pub(crate) index: usize,
    pub(crate) interval_ms: u64,
    pub(crate) transparent: bool,
    pub(crate) show_time: bool,
    pub(crate) show_uptime: bool,
    pub(crate) show_labels: bool,
    pub(crate) sort_key: SortKey,
    pub(crate) sort_dir: SortDir,
    pub(crate) wan_enabled: bool,
    pub(crate) settings_editing: bool,
    pub(crate) wan_url: &'a str,
    pub(crate) wan_url_edit: &'a str,
}

pub(crate) fn render_settings(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    settings: &SettingsView<'_>,
) {
    let SettingsView {
        index,
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
        ("Sort key", sort_key.as_str().to_string()),
        ("Sort dir", sort_dir.as_str().to_string()),
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

pub(crate) fn render_signal(
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

pub(crate) fn render_net_detail(frame: &mut Frame, area: Rect, theme: &Theme, network: &[NetRate]) {
    const NAME_W: usize = 12;
    const RATE_W: usize = 15;
    let mut lines: Vec<Line> = network
        .iter()
        .filter(|n| widgets::network::is_relevant_interface(&n.name))
        .map(|n| {
            Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{:<NAME_W$}", n.name),
                    Style::default().fg(theme.colors.accent),
                ),
                ratatui::text::Span::styled(
                    format!("↓ {:<RATE_W$}", human_rate(n.rx_bytes_per_sec)),
                    Style::default().fg(theme.colors.success),
                ),
                ratatui::text::Span::styled(
                    format!("  ↑ {:<RATE_W$}", human_rate(n.tx_bytes_per_sec)),
                    Style::default().fg(theme.colors.warning),
                ),
            ])
        })
        .collect();
    if lines.is_empty() {
        lines.push(Line::from("no interfaces").style(Style::default().fg(theme.colors.muted)));
    }

    let width = (NAME_W + 2 + RATE_W + 4 + RATE_W) as u16 + 4;
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

pub(crate) fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
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
}
