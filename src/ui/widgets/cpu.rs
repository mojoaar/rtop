use crate::data::snapshot::CpuSnapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{BarChart, Block, Gauge, Sparkline};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, cpu: &CpuSnapshot, spark: &[u64], theme: &Theme) {
    let block = Block::bordered()
        .title("CPU")
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [gauge_area, spark_area, cores_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

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

    let labels: Vec<String> = cpu
        .per_core
        .iter()
        .enumerate()
        .map(|(i, _)| format!("c{i}"))
        .collect();
    let data: Vec<(&str, u64)> = labels
        .iter()
        .zip(cpu.per_core.iter())
        .map(|(l, u)| (l.as_str(), *u as u64))
        .collect();
    frame.render_widget(
        BarChart::default()
            .data(&data)
            .bar_width(2)
            .bar_gap(1)
            .bar_style(Style::default().fg(theme.colors.accent)),
        cores_area,
    );
}
