//! Observability screen — system health, metrics, traces, and core runtime info.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::tui::{app::TuiApp, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(2, 5), Constraint::Ratio(3, 5)])
        .split(area);

    render_overview(f, app, chunks[0]);
    render_metrics_and_runtime(f, app, chunks[1]);
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

    // Core detail data
    let core = &app.core_detail;
    let kernel_status = core.pointer("/kernel/status").and_then(|s| s.as_str()).unwrap_or("?");
    let bus_subs = core.pointer("/event_bus/subscribers").and_then(|c| c.as_u64()).unwrap_or(0);
    let bus_sigs = core.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let runtime_tasks = core.pointer("/runtime/tasks_count").and_then(|c| c.as_u64()).unwrap_or(0);

    let _metrics_total = core.pointer("/metrics/signals_processed").and_then(|c| c.as_u64()).unwrap_or(0);
    let permissions = core.pointer("/permissions/mode").and_then(|s| s.as_str()).unwrap_or("?");

    let lines = vec![
        Line::from(vec![
            Span::styled("Service: ", Style::default().fg(colors::DIM)),
            Span::styled(app.obs.get("service").and_then(|v| v.as_str()).unwrap_or("?"), Style::default().fg(colors::TEXT)),
            Span::raw(" "),
            Span::styled(app.obs.get("version").and_then(|v| v.as_str()).unwrap_or("?"), Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled("Kernel: ", Style::default().fg(colors::DIM)),
            Span::styled(kernel_status, Style::default().fg(if kernel_status == "running" { colors::GREEN } else { colors::YELLOW }).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}s", uptime), Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Signals: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{} processed", signals_processed), Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::styled("Bus: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{} subs, {} sigs", bus_subs, bus_sigs), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Runtime tasks: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", runtime_tasks), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Permissions: ", Style::default().fg(colors::DIM)),
            Span::styled(permissions, Style::default().fg(if permissions == "open" { colors::YELLOW } else { colors::GREEN })),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Fields: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", app.obs.get("fields").and_then(|v| v.as_i64()).unwrap_or(0)), Style::default().fg(colors::GREEN)),
            Span::raw("  "),
            Span::styled("Processors: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", app.obs.get("processors").and_then(|v| v.as_i64()).unwrap_or(0)), Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("Signal types: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", app.obs.get("signal_types").and_then(|v| v.as_i64()).unwrap_or(0)), Style::default().fg(colors::YELLOW)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_metrics_and_runtime(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_processor_metrics(f, app, chunks[0]);
    render_runtime_info(f, app, chunks[1]);
}

fn render_processor_metrics(f: &mut Frame, app: &TuiApp, area: Rect) {
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

fn render_runtime_info(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Runtime & Config ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let core = &app.core_detail;
    let config = core.get("config");
    let rest_enabled = config.and_then(|c| c.get("rest_api_enabled").and_then(|v| v.as_bool())).unwrap_or(false);
    let default_port = config.and_then(|c| c.get("default_port").and_then(|v| v.as_u64())).unwrap_or(0);
    let cache_interval = config.and_then(|c| c.get("field_cache_interval_secs").and_then(|v| v.as_u64())).unwrap_or(0);
    let features = config.and_then(|c| c.get("features").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))).unwrap_or_default();

    let runtime = core.get("runtime");
    let tasks_count = runtime.and_then(|r| r.get("tasks_count").and_then(|v| v.as_u64())).unwrap_or(0);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("REST API: ", Style::default().fg(colors::DIM)),
            Span::styled(if rest_enabled { "Enabled" } else { "Disabled" }, Style::default().fg(if rest_enabled { colors::GREEN } else { colors::DIM })),
            Span::raw(" "),
            Span::styled(format!("(:{})", default_port), Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled("Cache interval: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}s", cache_interval), Style::default().fg(colors::YELLOW)),
        ]),
        Line::from(vec![
            Span::styled("Features: ", Style::default().fg(colors::DIM)),
            Span::styled(features, Style::default().fg(colors::TEXT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Runtime tasks: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{} spawned", tasks_count), Style::default().fg(colors::ACCENT)),
        ]),
    ];

    // Add runtime task names if available
    if let Some(tasks) = core.pointer("/runtime/tasks").and_then(|v| v.as_array()) {
        for task in tasks {
            if let Some(name) = task.as_str() {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(name, Style::default().fg(colors::DIM)),
                ]));
            }
        }
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
