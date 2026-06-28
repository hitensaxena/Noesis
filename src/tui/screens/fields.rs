//! Fields screen — field registrations and state with detail data.

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
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(area);

    render_field_list(f, app, chunks[0]);
    render_field_detail(f, app, chunks[1]);
}

fn render_field_list(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Fields ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(fields) = app.stats.get("field_names").and_then(|v| v.as_array()) {
        for field in fields {
            let name = field.as_str().unwrap_or("?");
            let (has_data, summary, icon) = field_data(name, app);
            let color = if has_data { colors::GREEN } else { colors::DIM };

            lines.push(Line::from(vec![
                Span::raw(format!("  {} ", icon)),
                Span::styled(format!("{:<20}", name), Style::default().fg(if has_data { colors::TEXT } else { colors::DIM }).add_modifier(if has_data { Modifier::BOLD } else { Modifier::empty() })),
                Span::styled(summary, Style::default().fg(color)),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No fields registered", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn field_data(name: &str, app: &TuiApp) -> (bool, String, &'static str) {
    match name {
        "memory" => {
            let mem = &app.memory_detail;
            let eps = mem.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
            let sem = mem.pointer("/semantic/count").and_then(|c| c.as_u64()).unwrap_or(0);
            (eps > 0, format!("{}eps {}sem", eps, sem), "\u{1F4DA}")
        }
        "identity" => {
            let id = &app.identity_detail;
            let b = id.pointer("/beliefs/count").and_then(|c| c.as_u64()).unwrap_or(0);
            let t = id.pointer("/traits/count").and_then(|c| c.as_u64()).unwrap_or(0);
            (b > 0, format!("{}bel {}traits", b, t), "\u{1F9D1}")
        }
        "agency" => {
            let exec = &app.agency_detail;
            let g = exec.pointer("/goals/active").and_then(|c| c.as_u64())
                .or_else(|| exec.pointer("/goals/items").and_then(|v| v.as_array().map(|a| a.len() as u64)))
                .unwrap_or(0);
            (g > 0, format!("{}goals", g), "\u{1F3AF}")
        }
        "reasoning" => {
            let r = &app.reasoning_detail;
            let i = r.get("insights").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0)
                + r.pointer("/insights/count").and_then(|c| c.as_u64()).unwrap_or(0);
            let d = r.get("decisions").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0)
                + r.pointer("/decisions/count").and_then(|c| c.as_u64()).unwrap_or(0);
            (i + d > 0, format!("{}ins {}dec", i, d), "\u{1F9E0}")
        }
        "awareness" => {
            let aware = &app.awareness_detail;
            let f = aware.pointer("/attention/current_focus").and_then(|s| s.as_str()).unwrap_or("none");
            (f != "none", format!("\"{}\"", f), "\u{1F4A1}")
        }
        "simulation" => {
            let sim = &app.simulation_detail;
            let s = sim.pointer("/scenarios/count").and_then(|c| c.as_u64()).unwrap_or(0);
            (s > 0, format!("{}scen", s), "\u{1F52E}")
        }
        "knowledge_graph" => {
            let mem = &app.memory_detail;
            let e = mem.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);
            let r = mem.pointer("/graph/relations").and_then(|c| c.as_u64()).unwrap_or(0);
            (e > 0, format!("{}ent {}rel", e, r), "\u{1F578}")
        }
        _ => (false, String::new(), "\u{25CF}"),
    }
}

fn render_field_detail(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Field Detail Summary ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let mut lines: Vec<Line> = Vec::new();

    // Memory
    let mem = &app.memory_detail;
    let eps = mem.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let sem = mem.pointer("/semantic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let work = mem.pointer("/working/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let proc = mem.pointer("/procedural/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let cons = mem.pointer("/consolidation/status").and_then(|s| s.as_str()).unwrap_or("—");
    lines.push(Line::from(vec![
        Span::styled(" \u{1F4DA} Memory:   ", Style::default().fg(colors::GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}ep {}sem {}wk {}pr", eps, sem, work, proc), Style::default().fg(if eps > 0 { colors::TEXT } else { colors::DIM })),
        Span::raw(" "),
        Span::styled(cons, Style::default().fg(colors::DIM)),
    ]));

    // Identity
    let id = &app.identity_detail;
    let b = id.pointer("/beliefs/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let t = id.pointer("/traits/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let vals = id.pointer("/values/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let roles = id.pointer("/roles/count").and_then(|c| c.as_u64()).unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{1F9D1} Identity:  ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}bel {}traits {}values {}roles", b, t, vals, roles), Style::default().fg(if b > 0 { colors::TEXT } else { colors::DIM })),
    ]));

    // Agency
    let exec = &app.agency_detail;
    let g = exec.pointer("/goals/active").and_then(|c| c.as_u64())
        .or_else(|| exec.pointer("/goals/items").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let ga = exec.pointer("/goals/active").and_then(|c| c.as_u64()).unwrap_or(g);
    let p = exec.pointer("/active_pursuits/count").and_then(|c| c.as_u64()).unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{1F3AF} Agency: ", Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}goals ({}a) {}pursuits", g, ga, p), Style::default().fg(if g > 0 { colors::TEXT } else { colors::DIM })),
    ]));

    // Awareness
    let aware = &app.awareness_detail;
    let focus = aware.pointer("/attention/current_focus").and_then(|s| s.as_str()).unwrap_or("none");
    let fd = aware.pointer("/attention/focus_stack/depth").and_then(|c| c.as_u64()).unwrap_or(0);
    let h = aware.pointer("/health/status").and_then(|s| s.as_str()).unwrap_or("—");
    lines.push(Line::from(vec![
        Span::styled(" \u{1F4A1} Awareness: ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("\"{}\" d={}", focus, fd), Style::default().fg(if focus != "none" { colors::TEXT } else { colors::DIM })),
        Span::raw(" "),
        Span::styled(h, Style::default().fg(if h == "nominal" { colors::GREEN } else { colors::DIM })),
    ]));

    // Simulation
    let sim = &app.simulation_detail;
    let sc = sim.pointer("/scenarios/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let asm = sim.pointer("/assumptions/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let wm = sim.pointer("/world_models/count").and_then(|c| c.as_u64()).unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{1F52E} Simulation:", Style::default().fg(colors::DIM).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}sc {}as {}wm", sc, asm, wm), Style::default().fg(colors::DIM)),
    ]));

    // Reasoning
    let reas = &app.reasoning_detail;
    let r_ins = reas.get("insights").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0)
        + reas.pointer("/insights/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let r_dec = reas.get("decisions").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0)
        + reas.pointer("/decisions/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let r_hyp = reas.get("hypotheses").and_then(|v| v.as_array().map(|a| a.len() as u64)).unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{1F9E0} Reasoning: ", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}ins {}dec {}hyp", r_ins, r_dec, r_hyp), Style::default().fg(if r_ins > 0 { colors::TEXT } else { colors::DIM })),
    ]));

    // Knowledge Graph
    let graph = &app.graph_detail;
    let g_base = graph.get("graph").or(Some(graph)).and_then(|g| g.as_object()).cloned().unwrap_or_default();
    let g_ent = g_base.get("entity_count").and_then(|c| c.as_u64())
        .or_else(|| g_base.get("entities").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let g_rel = g_base.get("relation_count").and_then(|c| c.as_u64())
        .or_else(|| g_base.get("links").and_then(|v| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{1F578} Knowledge G:", Style::default().fg(colors::GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}ent {}rel", g_ent, g_rel), Style::default().fg(if g_ent > 0 { colors::TEXT } else { colors::DIM })),
    ]));

    // Core (from stats endpoint)
    let f_count = app.stats.get("fields").and_then(|c| c.as_u64()).unwrap_or(0);
    let p_count = app.stats.get("processors").and_then(|c| c.as_u64()).unwrap_or(0);
    let s_count = app.stats.get("signal_types").and_then(|c| c.as_u64()).unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled(" \u{2699} Core:      ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}F {}P {}S", f_count, p_count, s_count), Style::default().fg(colors::TEXT)),
    ]));

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No field data available", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
