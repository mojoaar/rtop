use crate::data::format::human_bytes;
use crate::data::snapshot::GpuInfo;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, gpu: Option<&GpuInfo>, spark: &[u64], theme: &Theme) {
    let title = " GPU ".to_string();
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

    let Some(gpu) = gpu else {
        let [history_label, spark_chart, _spacer, na_area] = Layout::vertical([
            Constraint::Length(1), // history label
            Constraint::Length(1), // history sparkline
            Constraint::Min(0),
            Constraint::Length(1), // n/a
        ])
        .areas(inner);
        frame.render_widget(
            Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
            history_label,
        );
        frame.render_widget(
            Sparkline::default()
                .data(spark)
                .style(Style::default().fg(theme.colors.info)),
            spark_chart,
        );
        frame.render_widget(
            Paragraph::new("n/a")
                .style(Style::default().fg(theme.colors.muted))
                .alignment(Alignment::Right),
            na_area,
        );
        return;
    };

    let [active_label, gauge_chart, history_label, spark_chart, _spacer, mem_area] =
        Layout::vertical([
            Constraint::Length(1), // active label
            Constraint::Length(1), // active gauge
            Constraint::Length(1), // history label
            Constraint::Length(1), // history sparkline
            Constraint::Min(0),
            Constraint::Length(1), // mem text
        ])
        .areas(inner);

    frame.render_widget(
        Paragraph::new("Active").style(Style::default().fg(theme.colors.muted)),
        active_label,
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.info))
            .ratio((gpu.utilization_percent.clamp(0.0, 100.0) / 100.0) as f64)
            .label(format!("{:.0}%", gpu.utilization_percent)),
        gauge_chart,
    );

    frame.render_widget(
        Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
        history_label,
    );
    frame.render_widget(
        Sparkline::default()
            .data(spark)
            .style(Style::default().fg(theme.colors.info)),
        spark_chart,
    );

    let mem_label = if gpu.memory_total_bytes == 0 {
        format!("mem: {}", human_bytes(gpu.memory_used_bytes))
    } else {
        format!(
            "mem: {} / {}",
            human_bytes(gpu.memory_used_bytes),
            human_bytes(gpu.memory_total_bytes)
        )
    };
    frame.render_widget(
        Paragraph::new(mem_label)
            .style(Style::default().fg(theme.colors.muted))
            .alignment(Alignment::Right),
        mem_area,
    );
}
