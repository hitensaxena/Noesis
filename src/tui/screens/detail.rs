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
        "Reasoning" => &app.reasoning_detail,
        "Simulation" => &app.simulation_detail,
        "Knowledge Graph" => &app.graph_detail,
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
        "Reasoning" => "Metacognition, decisions, hypotheses, analogies, synthesis, concepts",
        "Simulation" => "Scenarios, world-models, forecasting, risk, and counterfactuals",
        "Knowledge Graph" => "Entities, relations, and knowledge structure",
        "Core"     => "Event bus, registry, config, and runtime info",
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
        "Reasoning" => render_reasoning_content(data, &mut lines),
        "Simulation" => render_simulation_content(data, &mut lines),
        "Knowledge Graph" => render_graph_content(data, &mut lines),
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
    let goals = data.get("goals").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let goals_items = goals.get("items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let goals_active = goals.get("active").and_then(|c| c.as_u64()).unwrap_or(0);
    let goals_completed = goals.get("completed").and_then(|c| c.as_u64()).unwrap_or(0);
    let goals_abandoned = goals.get("abandoned").and_then(|c| c.as_u64()).unwrap_or(0);
    let projects = data.pointer("/projects/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let tasks = data.pointer("/tasks/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let plans = data.pointer("/plans/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let opportunities = data.pointer("/opportunities/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let pursuits = data.pointer("/active_pursuits/count").and_then(|c| c.as_u64()).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Goals", goals_items, goals_items > 0);
    render_key_value(lines, "  Active", &Value::Number(goals_active.into()));
    render_key_value(lines, "  Completed", &Value::Number(goals_completed.into()));
    render_key_value(lines, "  Abandoned", &Value::Number(goals_abandoned.into()));
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

fn render_reasoning_content(data: &Value, lines: &mut Vec<Line>) {
    let insights = data.get("insights").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let decisions = data.get("decisions").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let hypotheses = data.get("hypotheses").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let analogies = data.get("analogies").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let concepts = data.get("concepts").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let models = data.get("models").or(data.get("mental_models")).and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let conclusions = data.get("conclusions").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_section(lines, "Insights", insights, insights > 0);
    render_section(lines, "Decisions", decisions, decisions > 0);
    render_section(lines, "Hypotheses", hypotheses, hypotheses > 0);
    render_section(lines, "Analogies", analogies, analogies > 0);
    render_section(lines, "Concepts", concepts, concepts > 0);
    render_section(lines, "Mental Models", models, models > 0);
    render_section(lines, "Conclusions", conclusions, conclusions > 0);
}

fn render_graph_content(data: &Value, lines: &mut Vec<Line>) {
    let base = data.get("graph").or(Some(data)).and_then(|g| g.as_object()).cloned().unwrap_or_default();
    let entities = base.get("entity_count").and_then(|c| c.as_u64())
        .or_else(|| base.get("entities").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let relations = base.get("relation_count").and_then(|c| c.as_u64())
        .or_else(|| base.get("links").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let nodes = base.get("nodes").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0);
    let edges = base.get("edges").or(base.get("links")).and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0);

    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Total entities", &Value::Number((entities + nodes).into()));
    render_key_value(lines, "Total relations", &Value::Number((relations + edges).into()));
    render_key_value(lines, "Node count", &Value::Number(nodes.into()));
    render_key_value(lines, "Edge count", &Value::Number(edges.into()));
}

fn render_core_content(data: &Value, lines: &mut Vec<Line>) {
    let config = data.get("config").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let rest_enabled = config.get("rest_api_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let default_port = config.get("default_port").and_then(|v| v.as_u64()).unwrap_or(0);
    let cache_interval = config.get("field_cache_interval_secs").and_then(|v| v.as_u64()).unwrap_or(0);
    let features = config.get("features").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();

    let bus_channels = data.pointer("/event_bus/channels").and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
    let meta = data.get("_meta").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let signals = meta.get("signals_processed").and_then(|c| c.as_u64()).unwrap_or(0);
    let arch = meta.get("arch").and_then(|s| s.as_str()).unwrap_or("—");
    let version = meta.get("version").and_then(|s| s.as_str()).unwrap_or("—");

    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "Version", &Value::String(version.to_string()));
    render_key_value(lines, "Architecture", &Value::String(arch.to_string()));
    render_key_value(lines, "Signal Total", &Value::Number(signals.into()));
    render_key_value(lines, "Bus Channels", &Value::Number(bus_channels.into()));
    lines.push(Line::from(Span::raw("")));
    render_key_value(lines, "REST API", &Value::String(if rest_enabled { "Enabled" } else { "Disabled" }.to_string()));
    render_key_value(lines, "Port", &Value::Number(default_port.into()));
    render_key_value(lines, "Cache Interval", &Value::Number(cache_interval.into()));
    render_key_value(lines, "Features", &Value::String(features));
}
