//! Observability screen — system health, metrics traces.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::tui::{app::TuiApp, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_overview(f, app, chunks[0]);
    render_metrics(f, app, chunks[1]);
}

fn render_overview(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Observability Overview ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::YELLOW));

    let uptime = app.obs.get("uptime_seconds").and_then(|v| v.as_i64()).unwrap_or(0);
    let signals_processed = app.obs.get("signals_processed").and_then(|v| v.as_object()).map(|o| {
        o.values().filter_map(|v| v.as_u64()).sum::<u64>()
    }).unwrap_or(0);

    let lines = vec![
        Line::from(Span::styled(format!("  Service: {}", app.obs.get("service").and_then(|v| v.as_str()).unwrap_or("?")), colors::TEXT)),
        Line::from(Span::styled(format!("  Version: {}", app.obs.get("version").and_then(|v| v.as_str()).unwrap_or("?")), colors::TEXT)),
        Line::from(Span::styled(format!("  Uptime: {}s", uptime), colors::ACCENT)),
        Line::from(Span::styled(format!("  Total signals: {}", signals_processed), colors::GREEN)),
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Fields & Processors:", colors::DIM)),
        Line::from(Span::styled(format!("    {} fields", app.obs.get("fields").and_then(|v| v.as_i64()).unwrap_or(0)), colors::GREEN)),
        Line::from(Span::styled(format!("    {} processors", app.obs.get("processors").and_then(|v| v.as_i64()).unwrap_or(0)), colors::ACCENT)),
        Line::from(Span::styled(format!("    {} signal types", app.obs.get("signal_types").and_then(|v| v.as_i64()).unwrap_or(0)), colors::YELLOW)),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_metrics(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Processor Metrics ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(stats) = app.obs.get("processor_stats").and_then(|v| v.as_object()) {
        let mut sorted: Vec<_> = stats.iter().collect();
        sorted.sort_by(|a, b| b.1.get("count").and_then(|v| v.as_u64()).unwrap_or(0)
            .cmp(&a.1.get("count").and_then(|v| v.as_u64()).unwrap_or(0)));

        for (name, m) in &sorted {
            let count = m.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let avg = m.get("avg_latency_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", name), colors::TEXT),
                Span::styled(format!("{:>6}x ", count), colors::YELLOW),
                Span::styled(format!("{:>5}ms", avg), colors::DIM),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  Waiting for metrics...", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
