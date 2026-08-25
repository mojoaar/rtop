use crate::data::format::{format_duration_secs, human_bytes};
use crate::data::snapshot::ProcessInfo;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{
    Block, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortKey {
    Cpu,
    Memory,
    Pid,
    Name,
}

pub fn sort(processes: &mut Vec<ProcessInfo>, key: SortKey) {
    processes.sort_by(|a, b| match key {
        SortKey::Cpu => b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
        SortKey::Memory => b.memory_bytes.cmp(&a.memory_bytes),
        SortKey::Pid => a.pid.cmp(&b.pid),
        SortKey::Name => a.name.cmp(&b.name),
    });
}

pub fn matches_filter(query: &str, p: &ProcessInfo) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    p.name.to_lowercase().contains(&q) || p.pid.to_string().contains(&q)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    processes: &[ProcessInfo],
    selected: Option<usize>,
    scroll: usize,
    theme: &Theme,
) {
    let block = Block::bordered()
        .title(" Processes ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [table_area, scroll_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).areas(inner);

    let widths = [
        Constraint::Length(7),
        Constraint::Length(12),
        Constraint::Min(10),
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(9),
        Constraint::Length(5),
    ];
    let header = Row::new(vec!["PID", "USER", "NAME", "CPU%", "MEM", "TIME", "THR"])
        .style(Style::default().fg(theme.colors.accent));

    let scroll = scroll.min(processes.len().saturating_sub(1));
    let visible = (table_area.height as usize).saturating_sub(1);
    let end = (scroll + visible).min(processes.len());
    let window = &processes[scroll.min(processes.len())..end];

    let rows: Vec<Row> = window
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let abs = scroll + i;
            let threads = p
                .threads
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string());
            let cells = vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.user.clone()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}", p.cpu_usage)),
                Cell::from(human_bytes(p.memory_bytes)),
                Cell::from(format_duration_secs(p.cpu_time)),
                Cell::from(threads),
            ];
            let mut row = Row::new(cells);
            if selected == Some(abs) {
                row = row.style(Style::default().bg(theme.colors.highlight));
            }
            row
        })
        .collect();

    frame.render_widget(Table::new(rows, widths).header(header), table_area);

    let mut state = ScrollbarState::new(processes.len())
        .position(scroll)
        .viewport_content_length(visible);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        scroll_area,
        &mut state,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, name: &str, cpu: f32, mem: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.into(),
            cpu_usage: cpu,
            memory_bytes: mem,
            status: "Running".into(),
            user: "root".into(),
            cpu_time: 0,
            threads: Some(1),
        }
    }

    #[test]
    fn sorts_by_cpu_desc() {
        let mut v = vec![p(1, "a", 10.0, 0), p(2, "b", 50.0, 0), p(3, "c", 30.0, 0)];
        sort(&mut v, SortKey::Cpu);
        assert_eq!(v[0].pid, 2);
        assert_eq!(v[2].pid, 1);
    }

    #[test]
    fn sorts_by_pid_asc() {
        let mut v = vec![p(9, "a", 0.0, 0), p(1, "b", 0.0, 0)];
        sort(&mut v, SortKey::Pid);
        assert_eq!(v[0].pid, 1);
    }

    #[test]
    fn matches_by_name_case_insensitive() {
        let proc = p(123, "Firefox", 0.0, 0);
        assert!(matches_filter("fire", &proc));
        assert!(matches_filter("FOX", &proc));
    }

    #[test]
    fn matches_by_pid() {
        let proc = p(1234, "bash", 0.0, 0);
        assert!(matches_filter("123", &proc));
    }

    #[test]
    fn empty_filter_matches_all() {
        let proc = p(1, "init", 0.0, 0);
        assert!(matches_filter("", &proc));
    }

    #[test]
    fn no_match_returns_false() {
        let proc = p(1, "init", 0.0, 0);
        assert!(!matches_filter("zzz", &proc));
    }
}
