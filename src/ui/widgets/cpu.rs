use crate::data::snapshot::CpuSnapshot;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{BarChart, Block, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, cpu: &CpuSnapshot, spark: &[u64], theme: &Theme) {
    let block = Block::bordered()
        .title(" CPU ")
        .title_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(theme.colors.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(inner);

    let [gauge_area, spark_area, info_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
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
    if let Some(load) = cpu.load_avg {
        if !info.is_empty() {
            info.push_str("  ·  ");
        }
        info.push_str(&format!("load: {:.2} {:.2} {:.2}", load[0], load[1], load[2]));
    }
    if !info.is_empty() {
        frame.render_widget(
            Paragraph::new(info).style(Style::default().fg(theme.colors.muted)),
            info_area,
        );
    }

    let labels: Vec<String> = cpu
        .per_core
        .iter()
        .map(|u| format!("{u:.0}%"))
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
        right_area,
    );
}
