//! Detail screen — deep field observability.
//!
//! Shows a structured breakdown of the selected field domain.
//! Use Left/Right to cycle through Identity, Memory, Agency,
//! Awareness, Simulation, and Core detail views.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use serde_json::Value;

use crate::tui::{
    app::{TuiApp, DETAIL_NAMES},
    colors,
};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let detail_name = DETAIL_NAMES[app.detail_index];
    let detail_data = detail_for(app, detail_name);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // detail header
            Constraint::Min(3),     // detail content
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_content(f, detail_name, &detail_data, chunks[1]);
}

fn detail_for<'a>(app: &'a TuiApp, name: &str) -> &'a Value {
    match name {
        "Identity" => &app.identity_detail,
        "Memory" => &app.memory_detail,
        "Agency" => &app.agency_detail,
        "Awareness" => &app.awareness_detail,
        "Simulation" => &app.simulation_detail,
        "Core" => &app.core_detail,
        _ => &app.core_detail,
    }
}

fn render_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let current = DETAIL_NAMES[app.detail_index];
    let total = DETAIL_NAMES.len();
    let nav = format!(" [{}/{}]  ← → navigate", app.detail_index + 1, total);

    let block = Block::default()
        .title(format!(" Detail: {} ", current))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let _meta = app.core_detail.get("_meta").or_else(|| {
        // Fallback: try any detail's _meta
        let others = [
            &app.identity_detail, &app.memory_detail,
            &app.agency_detail, &app.awareness_detail,
            &app.simulation_detail,
        ];
        others.iter().find_map(|d| d.get("_meta"))
    });

    let note = match current {
        "Identity" => "Beliefs, traits, identity, values, roles, and self-model",
        "Memory"   => "Episodic, semantic, procedural, graph, and working memory",
        "Agency" => "Goals, strategy, priorities, and what to pursue",
        "Awareness" => "Attention, observer, analytics, health, and curiosity",
        "Simulation" => "Scenarios, world-models, forecasting, risk, and counterfactuals",
        "Core"     => "Event bus, registry, scheduler, metrics, and permissions",
        _ => "",
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Domain: ", Style::default().fg(colors::DIM)),
            Span::styled(current, Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(nav),
        ]),
        Line::from(vec![
            Span::styled(note, Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_content(f: &mut Frame, detail_name: &str, data: &Value, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::DIM));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    match detail_name {
        "Identity" => render_identity_content(data, &mut lines),
        "Memory" => render_memory_content(data, &mut lines),
        "Agency" => render_agency_content(data, &mut lines),
        "Awareness" => render_awareness_content(data, &mut lines),
        "Simulation" => render_simulation_content(data, &mut lines),
        "Core" => render_core_content(data, &mut lines),
        _ => lines.push(Line::from(Span::styled("Unknown detail view", colors::DIM))),
    }

    // Add meta status at bottom
    if let Some(meta) = data.get("_meta") {
        let cached = meta.get("cached").and_then(|c| c.as_bool()).unwrap_or(false);
        let available = meta.get("data_available").and_then(|c| c.as_bool()).unwrap_or(false);
        let signals = meta.get("signals_processed").and_then(|c| c.as_u64()).unwrap_or(0);
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::styled("Cached: ", Style::default().fg(colors::DIM)),
            Span::styled(if cached { "✓" } else { "✗" }, Style::default().fg(if cached { colors::GREEN } else { colors::DIM })),
            Span::styled(" | Has data: ", Style::default().fg(colors::DIM)),
            Span::styled(if available { "✓" } else { "—" }, Style::default().fg(if available { colors::GREEN } else { colors::DIM })),
            Span::styled(" | Signals: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", signals), Style::default().fg(colors::YELLOW)),
        ]));
    }

    // Show note for each detail
    if let Some(note) = data.get("note").and_then(|n| n.as_str()) {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(format!("ℹ {}", note), Style::default().fg(colors::DIM))));
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_section(lines: &mut Vec<Line>, name: &str, count: usize, has_data: bool) {
    let color = if has_data { colors::GREEN } else { colors::DIM };
    let icon = if has_data { "●" } else { "○" };
    let count_str = if count > 0 { format!(" ({})", count) } else { String::new() };
    lines.push(Line::from(vec![
        Span::styled(format!("{} {}", icon, name), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(count_str, Style::default().fg(color)),
    ]));
}

fn render_key_value(lines: &mut Vec<Line>, key: &str, val: &Value) {
    let display = match val {
        Value::Null => "—".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(o) => format!("{{{} keys}}", o.len()),
    };
    lines.push(Line::from(vec![
        Span::styled(format!("  {}: ", key), Style::default().fg(colors::DIM)),
        Span::raw(display),
    ]));
}

fn render_identity_content(data: &Value, lines: &mut Vec<Line>) {
    // Identity info
    if let Some(id) = data.get("identity") {
        let version = id.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
        render_key_value(lines, "Version", &Value::Number(version.into()));
        render_key_value(lines, "Label", &id.get("label").cloned().unwrap_or(Value::Null));
    }

    // Sections with counts
    let beliefs = data.pointer("/beliefs/items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let traits = data.pointer("/traits/items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let values = data.pointer("/values/items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let roles = data.pointer("/roles/items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let principles = data.pointer("/principles/items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Beliefs", beliefs, beliefs > 0);
    render_section(lines, "Traits", traits, traits > 0);
    render_section(lines, "Values", values, values > 0);
    render_section(lines, "Roles", roles, roles > 0);
    render_section(lines, "Principles", principles, principles > 0);
}

fn render_memory_content(data: &Value, lines: &mut Vec<Line>) {
    let episodes = data.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let semantic = data.pointer("/semantic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let working = data.pointer("/working/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let procedural = data.pointer("/procedural/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let graph_entities = data.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);
    let graph_rels = data.pointer("/graph/relations").and_then(|c| c.as_u64()).unwrap_or(0);

    let consolidation = data.pointer("/consolidation/status").and_then(|s| s.as_str()).unwrap_or("inactive");
    let retrieval = data.pointer("/retrieval/mode").and_then(|s| s.as_str()).unwrap_or("none");

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Working Memory", working as usize, working > 0);
    render_section(lines, "Episodic", episodes as usize, episodes > 0);
    render_section(lines, "Semantic", semantic as usize, semantic > 0);
    render_section(lines, "Procedural", procedural as usize, procedural > 0);
    render_section(lines, "Graph Entities", graph_entities as usize, graph_entities > 0);
    render_key_value(lines, "Graph Relations", &Value::Number(graph_rels.into()));
    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Consolidation", &Value::String(consolidation.to_string()));
    render_key_value(lines, "Retrieval Mode", &Value::String(retrieval.to_string()));
}

fn render_agency_content(data: &Value, lines: &mut Vec<Line>) {
    let goals_total = data.pointer("/goals/total").and_then(|c| c.as_u64()).unwrap_or(0);
    let goals_active = data.pointer("/goals/active").and_then(|c| c.as_u64()).unwrap_or(0);
    let goals_completed = data.pointer("/goals/completed").and_then(|c| c.as_u64()).unwrap_or(0);
    let projects = data.pointer("/projects/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let tasks = data.pointer("/tasks/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let plans = data.pointer("/plans/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let opportunities = data.pointer("/opportunities/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let pursuits = data.pointer("/active_pursuits/count").and_then(|c| c.as_u64()).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Goals", goals_total as usize, goals_total > 0);
    render_key_value(lines, "  Active", &Value::Number(goals_active.into()));
    render_key_value(lines, "  Completed", &Value::Number(goals_completed.into()));
    render_section(lines, "Projects", projects as usize, projects > 0);
    render_section(lines, "Tasks", tasks as usize, tasks > 0);
    render_section(lines, "Plans", plans as usize, plans > 0);
    render_section(lines, "Opportunities", opportunities as usize, opportunities > 0);
    render_section(lines, "Active Pursuits", pursuits as usize, pursuits > 0);
}

fn render_awareness_content(data: &Value, lines: &mut Vec<Line>) {
    let observer_status = data.pointer("/observer/status").and_then(|s| s.as_str()).unwrap_or("inactive");
    let current_focus = data.pointer("/attention/current_focus").and_then(|s| s.as_str()).unwrap_or("none");
    let focus_depth = data.pointer("/attention/focus_stack/depth").and_then(|d| d.as_u64()).unwrap_or(0);
    let salience_entries = data.pointer("/attention/salience_map/entries").and_then(|c| c.as_u64()).unwrap_or(0);
    let health_status = data.pointer("/health/status").and_then(|s| s.as_str()).unwrap_or("unknown");
    let curiosity = data.pointer("/curiosity/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let analytics_signals = data.pointer("/analytics/signals_observed").and_then(|c| c.as_u64()).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Observer", &Value::String(observer_status.to_string()));
    render_key_value(lines, "Current Focus", &Value::String(current_focus.to_string()));
    render_key_value(lines, "Focus Stack Depth", &Value::Number(focus_depth.into()));
    render_key_value(lines, "Salience Entries", &Value::Number(salience_entries.into()));
    render_key_value(lines, "Signals Observed", &Value::Number(analytics_signals.into()));
    render_key_value(lines, "Health", &Value::String(health_status.to_string()));
    render_key_value(lines, "Curiosity Items", &Value::Number(curiosity.into()));
}

fn render_simulation_content(data: &Value, lines: &mut Vec<Line>) {
    let scenarios = data.pointer("/scenarios/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let assumptions = data.pointer("/assumptions/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let world_models = data.pointer("/world_models/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let forecasts = data.pointer("/forecasting/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let risks = data.pointer("/risk/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let counterfactuals = data.pointer("/counterfactuals/count").and_then(|c| c.as_u64()).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Scenarios", scenarios as usize, scenarios > 0);
    render_section(lines, "Assumptions", assumptions as usize, assumptions > 0);
    render_section(lines, "World Models", world_models as usize, world_models > 0);
    render_section(lines, "Forecasts", forecasts as usize, forecasts > 0);
    render_section(lines, "Risks", risks as usize, risks > 0);
    render_section(lines, "Counterfactuals", counterfactuals as usize, counterfactuals > 0);
}

fn render_core_content(data: &Value, lines: &mut Vec<Line>) {
    let bus_subs = data.pointer("/event_bus/subscribers").and_then(|c| c.as_u64()).unwrap_or(0);
    let bus_sigs = data.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let scheduler_status = data.pointer("/scheduler/status").and_then(|s| s.as_str()).unwrap_or("—");
    let reg_fields = data.pointer("/registry/fields").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let reg_procs = data.pointer("/registry/processors").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let reg_signals = data.pointer("/registry/signal_types").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let plugins = data.pointer("/plugin_loader/status").and_then(|s| s.as_str()).unwrap_or("—");
    let kernel_status = data.pointer("/kernel/status").and_then(|s| s.as_str()).unwrap_or("—");
    let uptime = data.pointer("/kernel/uptime_secs").and_then(|u| u.as_u64()).unwrap_or(0);
    let runtime_tasks = data.pointer("/runtime/tasks_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let metrics_total = data.pointer("/metrics/signals_processed").and_then(|c| c.as_u64()).unwrap_or(0);
    let permissions = data.pointer("/permissions/mode").and_then(|s| s.as_str()).unwrap_or("—");

    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Event Bus Subscribers", &Value::Number(bus_subs.into()));
    render_key_value(lines, "Event Bus Signal Count", &Value::Number(bus_sigs.into()));
    render_key_value(lines, "Scheduler", &Value::String(scheduler_status.to_string()));
    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Fields", &Value::Number(reg_fields.into()));
    render_key_value(lines, "Processors", &Value::Number(reg_procs.into()));
    render_key_value(lines, "Signal Types", &Value::Number(reg_signals.into()));
    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Plugin Loader", &Value::String(plugins.to_string()));
    render_key_value(lines, "Kernel", &Value::String(kernel_status.to_string()));
    render_key_value(lines, "Uptime (s)", &Value::Number(uptime.into()));
    render_key_value(lines, "Runtime Tasks", &Value::Number(runtime_tasks.into()));
    render_key_value(lines, "Metrics Total", &Value::Number(metrics_total.into()));
    render_key_value(lines, "Permissions", &Value::String(permissions.to_string()));
}
