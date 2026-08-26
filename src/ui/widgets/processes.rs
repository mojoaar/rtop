use crate::data::format::{format_duration_secs, human_bytes};
use crate::data::snapshot::ProcessInfo;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Cpu,
    Memory,
    Pid,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

pub fn default_dir(key: SortKey) -> SortDir {
    match key {
        SortKey::Cpu | SortKey::Memory => SortDir::Desc,
        SortKey::Pid | SortKey::Name => SortDir::Asc,
    }
}

impl SortKey {
    pub fn as_str(self) -> &'static str {
        match self {
            SortKey::Cpu => "cpu",
            SortKey::Memory => "memory",
            SortKey::Pid => "pid",
            SortKey::Name => "name",
        }
    }
}

impl SortDir {
    pub fn as_str(self) -> &'static str {
        match self {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        }
    }
}

pub fn parse_sort_key(s: &str) -> SortKey {
    match s {
        "memory" => SortKey::Memory,
        "pid" => SortKey::Pid,
        "name" => SortKey::Name,
        _ => SortKey::Cpu,
    }
}

pub fn parse_sort_dir(s: &str) -> SortDir {
    match s {
        "asc" => SortDir::Asc,
        _ => SortDir::Desc,
    }
}

pub struct ProcessView {
    pub selected: Option<usize>,
    pub scroll: usize,
    pub total: usize,
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
}

pub fn cmp(a: &ProcessInfo, b: &ProcessInfo, key: SortKey, dir: SortDir) -> std::cmp::Ordering {
    let ord = match key {
        SortKey::Cpu => a
            .cpu_usage
            .partial_cmp(&b.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortKey::Memory => a.memory_bytes.cmp(&b.memory_bytes),
        SortKey::Pid => a.pid.cmp(&b.pid),
        SortKey::Name => a.name.cmp(&b.name),
    };
    match dir {
        SortDir::Asc => ord,
        SortDir::Desc => ord.reverse(),
    }
}

pub fn matches_filter(query: &str, p: &ProcessInfo) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    contains_ci(&p.name, &q) || contains_ci(&p.user, &q) || p.pid.to_string().contains(&q)
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let hay: Vec<char> = haystack.chars().collect();
    let ndl: Vec<char> = needle.chars().collect();
    if ndl.len() > hay.len() {
        return false;
    }
    hay.windows(ndl.len()).any(|w| {
        w.iter()
            .zip(&ndl)
            .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
    })
}

fn header_label(label: &str, key: Option<SortKey>, active: SortKey, dir: SortDir) -> String {
    if key == Some(active) {
        let arrow = match dir {
            SortDir::Asc => "↑",
            SortDir::Desc => "↓",
        };
        format!("{label}{arrow}")
    } else {
        label.to_string()
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    processes: &[ProcessInfo],
    order: &[usize],
    view: &ProcessView,
    theme: &Theme,
) {
    let ProcessView {
        selected,
        scroll,
        total,
        sort_key,
        sort_dir,
    } = *view;
    let block = Block::bordered()
        .title(format!(" Processes · {} total ", total))
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
    let header = Row::new(vec![
        header_label("PID", Some(SortKey::Pid), sort_key, sort_dir),
        header_label("USER", None, sort_key, sort_dir),
        header_label("NAME", Some(SortKey::Name), sort_key, sort_dir),
        header_label("CPU%", Some(SortKey::Cpu), sort_key, sort_dir),
        header_label("MEM", Some(SortKey::Memory), sort_key, sort_dir),
        header_label("TIME", None, sort_key, sort_dir),
        header_label("THR", None, sort_key, sort_dir),
    ])
    .style(Style::default().fg(theme.colors.accent));

    let len = order.len();
    let scroll = scroll.min(len.saturating_sub(1));
    let visible = (table_area.height as usize).saturating_sub(1);
    let end = (scroll + visible).min(len);
    let window = &order[scroll.min(len)..end];

    let rows: Vec<Row> = window
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let p = &processes[idx];
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

    let mut state = ScrollbarState::new(len)
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
            cmd: String::new(),
            cpu_usage: cpu,
            memory_bytes: mem,
            status: "Running".into(),
            user: "root".into(),
            cpu_time: 0,
            threads: Some(1),
        }
    }

    #[test]
    fn cmp_cpu_desc() {
        let a = p(1, "a", 10.0, 0);
        let b = p(2, "b", 50.0, 0);
        assert_eq!(
            cmp(&a, &b, SortKey::Cpu, SortDir::Desc),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn cmp_cpu_asc() {
        let a = p(1, "a", 10.0, 0);
        let b = p(2, "b", 50.0, 0);
        assert_eq!(
            cmp(&a, &b, SortKey::Cpu, SortDir::Asc),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn cmp_pid_asc() {
        let a = p(9, "a", 0.0, 0);
        let b = p(1, "b", 0.0, 0);
        assert_eq!(
            cmp(&a, &b, SortKey::Pid, SortDir::Asc),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn cmp_name_desc() {
        let a = p(1, "alpha", 0.0, 0);
        let b = p(2, "zebra", 0.0, 0);
        assert_eq!(
            cmp(&a, &b, SortKey::Name, SortDir::Desc),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn contains_ci_substring() {
        assert!(contains_ci("Firefox", "fire"));
        assert!(contains_ci("Firefox", "FOX"));
        assert!(!contains_ci("Firefox", "xyz"));
        assert!(!contains_ci("ab", "abc"));
        assert!(contains_ci("ab", ""));
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
    fn matches_by_user_case_insensitive() {
        let mut proc = p(123, "bash", 0.0, 0);
        proc.user = "mojoaar".into();
        assert!(matches_filter("mojoaar", &proc));
        assert!(matches_filter("MOJO", &proc));
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

    #[test]
    fn parse_sort_key_maps_strings() {
        assert_eq!(parse_sort_key("cpu"), SortKey::Cpu);
        assert_eq!(parse_sort_key("memory"), SortKey::Memory);
        assert_eq!(parse_sort_key("pid"), SortKey::Pid);
        assert_eq!(parse_sort_key("name"), SortKey::Name);
        assert_eq!(parse_sort_key("bogus"), SortKey::Cpu);
    }

    #[test]
    fn parse_sort_dir_maps_strings() {
        assert_eq!(parse_sort_dir("asc"), SortDir::Asc);
        assert_eq!(parse_sort_dir("desc"), SortDir::Desc);
        assert_eq!(parse_sort_dir("bogus"), SortDir::Desc);
    }

    #[test]
    fn sort_key_and_dir_round_trip() {
        for k in [SortKey::Cpu, SortKey::Memory, SortKey::Pid, SortKey::Name] {
            assert_eq!(parse_sort_key(k.as_str()), k);
        }
        assert_eq!(parse_sort_dir(SortDir::Asc.as_str()), SortDir::Asc);
        assert_eq!(parse_sort_dir(SortDir::Desc.as_str()), SortDir::Desc);
    }
}
