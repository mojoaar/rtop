use crate::data::snapshot::CpuSnapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, cpu: &CpuSnapshot, spark: &[u64], theme: &Theme) {
    let title = match cpu.load_avg {
        Some(l) => format!(" CPU · load: {:.2} {:.2} {:.2} ", l[0], l[1], l[2]),
        None => " CPU ".to_string(),
    };
    let block = Block::bordered()
        .title(title)
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [left_area, _gap, right_area] = Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Length(2),
        Constraint::Ratio(1, 2),
    ])
    .areas(inner);

    let [gauge_area, spark_area, _spacer, info_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(left_area);

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.accent))
            .ratio((cpu.global_usage.clamp(0.0, 100.0) / 100.0) as f64)
            .label(format!("{:.0}%", cpu.global_usage)),
        gauge_area,
    );

    frame.render_widget(
        Sparkline::default()
            .data(spark)
            .style(Style::default().fg(theme.colors.accent)),
        spark_area,
    );

    let mut info = String::new();
    if !cpu.brand.is_empty() {
        info.push_str(&cpu.brand);
        if cpu.frequency_mhz > 0 {
            info.push_str(&format!(" @ {:.1} GHz", cpu.frequency_mhz as f64 / 1000.0));
        }
    }
    if !info.is_empty() {
        frame.render_widget(
            Paragraph::new(info).style(Style::default().fg(theme.colors.muted)),
            info_area,
        );
    }

    let bar_width = right_area.width.saturating_sub(9) as usize;
    let lines: Vec<Line> = cpu
        .per_core
        .iter()
        .enumerate()
        .map(|(i, u)| Line::from(format!("{:>2} {} {:>4.0}%", i, bar(*u, bar_width), u)))
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(theme.colors.text)),
        right_area,
    );
}

fn bar(pct: f32, width: usize) -> String {
    let width = width.max(1);
    let filled = ((pct.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    format!("{:<width$}", "█".repeat(filled.min(width)), width = width)
}
