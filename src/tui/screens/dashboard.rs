//! Dashboard screen — system overview at a glance.
//!
//! Shows live cognitive state from all 6 field domains using the
//! deep observability detail API endpoints.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Gauge},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::{app::{TuiApp, DETAIL_NAMES}, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // system card
            Constraint::Length(6),   // field summary cards
            Constraint::Min(4),      // cascade flow + detail
        ])
        .split(area);

    render_system_card(f, app, chunks[0]);
    render_field_summaries(f, app, chunks[1]);
    render_cascade_and_detail(f, app, chunks[2]);
}

fn render_system_card(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let uptime = app.obs.get("uptime_seconds").and_then(|v| v.as_i64()).unwrap_or(0);
    let signals_processed = app.obs.get("signals_processed").and_then(|v| v.as_object())
        .map(|o| o.values().filter_map(|v| v.as_u64()).sum::<u64>()).unwrap_or(0);

    let fields = app.obs.get("fields").and_then(|v| v.as_i64()).unwrap_or(0);
    let processors = app.obs.get("processors").and_then(|v| v.as_i64()).unwrap_or(0);
    let signal_types = app.obs.get("signal_types").and_then(|v| v.as_i64()).unwrap_or(0);

    // Build an uptime gauge
    let uptime_pct = ((uptime as f64 / 3600.0).min(1.0) * 100.0) as u16;

    let header = block.inner(area);
    f.render_widget(block, area);

    let gauge_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(header);

    let text = vec![
        Line::from(vec![
            Span::styled("Architecture: ", Style::default().fg(colors::DIM)),
            Span::styled("Decentralized Signal Cascade", Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("Signal Count: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", signals_processed), Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" | State: ", Style::default().fg(colors::DIM)),
            Span::styled("Equilibrium", Style::default().fg(colors::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" | Components: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}F {}P {}S", fields, processors, signal_types), Style::default().fg(colors::TEXT)),
        ]),
    ];

    f.render_widget(Paragraph::new(text), gauge_chunk[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(colors::PRIMARY))
        .label(format!("Uptime: {}s", uptime))
        .percent(uptime_pct);
    f.render_widget(gauge, gauge_chunk[1]);
}

fn render_field_summaries(f: &mut Frame, app: &TuiApp, area: Rect) {
    // Row 1: Identity, Memory, Agency in a 3-row grid (now 8 fields)
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(rows[0]);
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(rows[1]);
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(rows[2]);

    render_identity_summary(f, app, row1[0]);
    render_memory_summary(f, app, row1[1]);
    render_agency_summary(f, app, row1[2]);
    render_awareness_summary(f, app, row2[0]);
    render_reasoning_summary(f, app, row2[1]);
    render_simulation_summary(f, app, row2[2]);
    render_knowledge_graph_summary(f, app, row3[0]);
    render_core_summary(f, app, row3[1]);
}

fn render_identity_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let id = &app.identity_detail;
    let beliefs = id.pointer("/beliefs/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let traits = id.pointer("/traits/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let version = id.pointer("/identity/version").and_then(|c| c.as_u64()).unwrap_or(0);

    let has_data = beliefs > 0 || traits > 0;
    let block = Block::default()
        .title(format!(" {} Identity ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("v{}", version), Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!("{} beliefs", beliefs), Style::default().fg(if beliefs > 0 { colors::GREEN } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} traits", traits), Style::default().fg(if traits > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_memory_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let mem = &app.memory_detail;
    let episodes = mem.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let semantic = mem.pointer("/semantic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let graph_ents = mem.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);
    let consolidation = mem.pointer("/consolidation/status").and_then(|s| s.as_str()).unwrap_or("—");

    let has_data = episodes > 0;
    let block = Block::default()
        .title(format!(" {} Memory ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} episodes", episodes), Style::default().fg(if episodes > 0 { colors::GREEN } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} semantic", semantic), Style::default().fg(if semantic > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} graph", graph_ents), Style::default().fg(if graph_ents > 0 { colors::YELLOW } else { colors::DIM })),
            Span::raw(" "),
            Span::styled(consolidation, Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_agency_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let exec = &app.agency_detail;
    let goals = exec.pointer("/goals/active").and_then(|c| c.as_u64())
        .or_else(|| exec.pointer("/goals/items").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let active = exec.pointer("/goals/active").and_then(|c| c.as_u64()).unwrap_or(goals);
    let pursuits = exec.pointer("/active_pursuits/count").and_then(|c| c.as_u64()).unwrap_or(0);

    let has_data = goals > 0;
    let block = Block::default()
        .title(format!(" {} Agency ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} goals", goals), Style::default().fg(if goals > 0 { colors::GREEN } else { colors::DIM })),
            Span::raw(" "),
            Span::styled(format!("({} active)", active), Style::default().fg(if active > 0 { colors::YELLOW } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} pursuits", pursuits), Style::default().fg(if pursuits > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
        Line::from(Span::raw("")),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_awareness_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let aware = &app.awareness_detail;
    let focus = aware.pointer("/attention/current_focus").and_then(|s| s.as_str()).unwrap_or("none");
    let observer = aware.pointer("/observer/status").and_then(|s| s.as_str()).unwrap_or("—");
    let health = aware.pointer("/health/status").and_then(|s| s.as_str()).unwrap_or("—");

    let has_data = focus != "none";
    let block = Block::default()
        .title(format!(" {} Awareness ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled("observer:", Style::default().fg(colors::DIM)),
            Span::raw(format!(" {}", observer)),
        ]),
        Line::from(vec![
            Span::styled("focus:", Style::default().fg(colors::DIM)),
            Span::raw(format!(" {}", focus)),
        ]),
        Line::from(vec![
            Span::styled("health:", Style::default().fg(colors::DIM)),
            Span::styled(format!(" {}", health), Style::default().fg(if health == "nominal" { colors::GREEN } else { colors::YELLOW })),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_simulation_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let sim = &app.simulation_detail;
    let scenarios = sim.pointer("/scenarios/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let forecasts = sim.pointer("/forecasting/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let risks = sim.pointer("/risk/count").and_then(|c| c.as_u64()).unwrap_or(0);

    let has_data = scenarios > 0;
    let block = Block::default()
        .title(format!(" {} Simulation ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} scenarios", scenarios), Style::default().fg(if scenarios > 0 { colors::GREEN } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} forecasts", forecasts), Style::default().fg(if forecasts > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} risks", risks), Style::default().fg(if risks > 0 { colors::YELLOW } else { colors::DIM })),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_reasoning_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let reas = &app.reasoning_detail;
    let insights = reas.pointer("/insights/count").and_then(|c| c.as_u64())
        .or_else(|| reas.get("insights").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let decisions = reas.pointer("/decisions/count").and_then(|c| c.as_u64())
        .or_else(|| reas.get("decisions").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let mental_models = reas.pointer("/mental_models/count").and_then(|c| c.as_u64())
        .or_else(|| reas.get("models").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);

    let has_data = insights > 0 || decisions > 0;
    let block = Block::default()
        .title(format!(" {} Reasoning ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} insights", insights), Style::default().fg(if insights > 0 { colors::GREEN } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} decisions", decisions), Style::default().fg(if decisions > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} models", mental_models), Style::default().fg(if mental_models > 0 { colors::YELLOW } else { colors::DIM })),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_knowledge_graph_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let graph = &app.graph_detail;
    let base = graph.get("graph").or(Some(graph)).and_then(|g| g.as_object()).cloned().unwrap_or_default();
    let entities = base.get("entity_count").and_then(|c| c.as_u64())
        .or_else(|| base.get("entities").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let relations = base.get("relation_count").and_then(|c| c.as_u64())
        .or_else(|| base.get("relations").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .or_else(|| base.get("links").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);

    // Also show memory-local graph entities
    let mem = &app.memory_detail;
    let mem_ents = mem.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);

    let has_data = entities > 0 || mem_ents > 0;
    let block = Block::default()
        .title(format!(" {} Knowledge Graph ", if has_data { "●"} else { "○" }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if has_data { colors::GREEN } else { colors::DIM }));

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} entities", entities + mem_ents), Style::default().fg(if entities > 0 { colors::GREEN } else { colors::DIM })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} relations", relations), Style::default().fg(if relations > 0 { colors::ACCENT } else { colors::DIM })),
        ]),
        Line::from(Span::raw("")),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_core_summary(f: &mut Frame, app: &TuiApp, area: Rect) {
    let core = &app.core_detail;
    let state = core.get("config").and_then(|c| c.get("rest_api_enabled").and_then(|v| v.as_bool())).unwrap_or(false);
    let kernel = if state { "running" } else { "stopped" };
    let bus_sigs = core.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let runtime = core.pointer("/runtime/tasks_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let permissions = "open";

    let block = Block::default()
        .title(" ♦ Core ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let text = vec![
        Line::from(vec![
            Span::styled("kernel:", Style::default().fg(colors::DIM)),
            Span::styled(format!(" {}", kernel), Style::default().fg(if kernel == "running" { colors::GREEN } else { colors::YELLOW })),
        ]),
        Line::from(vec![
            Span::styled(format!("{} bus sigs", bus_sigs), Style::default().fg(colors::DIM)),
            Span::raw(" "),
            Span::styled(format!("{} tasks", runtime), Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled("perms:", Style::default().fg(colors::DIM)),
            Span::styled(format!(" {}", permissions), Style::default().fg(if permissions == "open" { colors::YELLOW } else { colors::GREEN })),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_cascade_and_detail(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_cascade_flow(f, app, chunks[0]);
    render_dashboard_detail(f, app, chunks[1]);
}

fn render_cascade_flow(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Signal Cascade Flow ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let signals = app.signals.get("signals").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let mut lines: Vec<Line> = Vec::new();
    let mut sorted: Vec<_> = signals.iter()
        .filter(|(k, _)| *k != "cascade.dispatch")
        .collect();
    sorted.sort_by(|a, b| b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0)));

    for (name, count) in sorted.iter().take(8) {
        let c = count.as_u64().unwrap_or(0);
        let color = if c > 10 { colors::GREEN } else if c > 3 { colors::YELLOW } else { colors::DIM };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>4}x ", c), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(name.as_str()),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No signals yet — inject an experience", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_dashboard_detail(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Quick Stats ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let signal_types = app.signal_types.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
    let total_signals = app.obs.get("signals_total").and_then(|c| c.as_u64())
        .or_else(|| app.obs.get("signals_processed").and_then(|v| v.as_object())
            .map(|o| o.values().filter_map(|v| v.as_u64()).sum::<u64>()))
        .unwrap_or(0);

    // Get processor/field info from stats endpoint
    let procs_count = app.stats.get("processors").and_then(|c| c.as_u64()).unwrap_or(0);
    let fields_count = app.stats.get("fields").and_then(|c| c.as_u64()).unwrap_or(0);

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("{} signal types registered", signal_types), Style::default().fg(colors::YELLOW)),
        ]),
        Line::from(vec![
            Span::styled(format!("{} total signals", total_signals), Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled(format!("{} fields", fields_count), Style::default().fg(colors::GREEN)),
            Span::raw("  "),
            Span::styled(format!("{} processors", procs_count), Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Tab: Navigate  ", Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled("Detail: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{} ({})", DETAIL_NAMES[app.detail_index], app.detail_index + 1), Style::default().fg(colors::PRIMARY)),
        ]),
        Line::from(vec![
            Span::styled("Refresh: ", Style::default().fg(colors::DIM)),
            Span::styled(if app.auto_refresh { "auto" } else { "manual" }, Style::default().fg(if app.auto_refresh { colors::GREEN } else { colors::YELLOW })),
            Span::raw(" "),
            Span::styled(format!("({}s)", app.refresh_interval_secs()), Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}
