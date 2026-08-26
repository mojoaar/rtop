use crate::data::format::human_bytes;
use crate::data::snapshot::GpuInfo;
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    gpu: Option<&GpuInfo>,
    spark: &[u64],
    show_labels: bool,
    theme: &Theme,
) {
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
        let mut constraints: Vec<Constraint> = Vec::new();
        if show_labels {
            constraints.push(Constraint::Length(1)); // history label
        }
        constraints.push(Constraint::Length(1)); // history sparkline
        constraints.push(Constraint::Min(0));
        constraints.push(Constraint::Length(1)); // n/a
        let areas = Layout::vertical(constraints).split(inner);

        let mut idx = 0;
        if show_labels {
            frame.render_widget(
                Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
                areas[idx],
            );
            idx += 1;
        }
        frame.render_widget(
            Sparkline::default()
                .data(spark)
                .style(Style::default().fg(theme.colors.info)),
            areas[idx],
        );
        idx += 2; // skip sparkline + fill
        frame.render_widget(
            Paragraph::new("n/a")
                .style(Style::default().fg(theme.colors.muted))
                .alignment(Alignment::Right),
            areas[idx],
        );
        return;
    };

    let mut constraints: Vec<Constraint> = Vec::new();
    if show_labels {
        constraints.push(Constraint::Length(1)); // active label
    }
    constraints.push(Constraint::Length(1)); // active gauge
    if show_labels {
        constraints.push(Constraint::Length(1)); // history label
    } else {
        constraints.push(Constraint::Length(1)); // spacer between active/history
    }
    constraints.push(Constraint::Length(1)); // history sparkline
    constraints.push(Constraint::Min(0));
    constraints.push(Constraint::Length(1)); // mem text
    let areas = Layout::vertical(constraints).split(inner);

    let mut idx = 0;
    if show_labels {
        frame.render_widget(
            Paragraph::new("Active").style(Style::default().fg(theme.colors.muted)),
            areas[idx],
        );
        idx += 1;
    }
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.colors.info))
            .ratio((gpu.utilization_percent.clamp(0.0, 100.0) / 100.0) as f64)
            .label(format!("{:.0}%", gpu.utilization_percent)),
        areas[idx],
    );
    idx += 1;
    if show_labels {
        frame.render_widget(
            Paragraph::new("History").style(Style::default().fg(theme.colors.muted)),
            areas[idx],
        );
    }
    idx += 1;
    frame.render_widget(
        Sparkline::default()
            .data(spark)
            .style(Style::default().fg(theme.colors.info)),
        areas[idx],
    );
    idx += 2; // skip sparkline + fill

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
        areas[idx],
    );
}
