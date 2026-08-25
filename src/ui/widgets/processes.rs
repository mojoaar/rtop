use crate::data::snapshot::ProcessInfo;
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Row, Table};
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

pub fn render(
    frame: &mut Frame,
    area: Rect,
    processes: &[ProcessInfo],
    selected: Option<usize>,
    theme: &Theme,
) {
    let block = Block::bordered()
        .title("Processes")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let widths = [
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Min(10),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(8),
    ];
    let header = Row::new(vec!["PID", "NAME", "CPU%", "MEM", "STATE"])
        .style(Style::default().fg(theme.colors.accent));

    let rows: Vec<Row> = processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let cells = vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}", p.cpu_usage)),
                Cell::from(crate::data::format::human_bytes(p.memory_bytes)),
                Cell::from(p.status.clone()),
            ];
            let mut row = Row::new(cells);
            if selected == Some(i) {
                row = row.style(Style::default().bg(theme.colors.highlight));
            }
            row
        })
        .collect();

    frame.render_widget(
        Table::new(rows, widths).header(header),
        inner,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, name: &str, cpu: f32, mem: u64) -> ProcessInfo {
        ProcessInfo { pid, name: name.into(), cpu_usage: cpu, memory_bytes: mem, status: "Running".into() }
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
}
