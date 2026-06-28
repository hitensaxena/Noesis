//! All rendering. Pure functions of `&App` (plus mutable `Sel` state for lists).
//! Matches curlyos-tui's data comprehensiveness — 5 tabs, 20+ sub-views.

use crate::tui::api::*;
use crate::tui::app::{App, Overlay, Tab, FIELD_SUBS, MEMORY_SUBS, SIGNAL_SUBS, SYSTEM_SUBS, TABS};
use crate::tui::colors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

// ── Palette ───────────────────────────────────────────────────────────────────
const GREEN:  ratatui::style::Color = ratatui::style::Color::Rgb(64, 250, 146);
const MINT:   ratatui::style::Color = ratatui::style::Color::Rgb(110, 240, 180);
const CYAN:   ratatui::style::Color = ratatui::style::Color::Rgb(86, 230, 244);
const LIME:   ratatui::style::Color = ratatui::style::Color::Rgb(178, 245, 96);
const CORAL:  ratatui::style::Color = ratatui::style::Color::Rgb(255, 138, 110);
const PERI:   ratatui::style::Color = ratatui::style::Color::Rgb(132, 170, 255);
const PURPLE: ratatui::style::Color = ratatui::style::Color::Rgb(192, 150, 255);
const AMBER:  ratatui::style::Color = ratatui::style::Color::Rgb(255, 198, 92);
const RED:    ratatui::style::Color = ratatui::style::Color::Rgb(255, 104, 104);
const DIM:    ratatui::style::Color = ratatui::style::Color::Rgb(128, 138, 158);
const FAINT:  ratatui::style::Color = ratatui::style::Color::Rgb(78, 86, 104);
const TEXT:   ratatui::style::Color = ratatui::style::Color::Rgb(218, 224, 234);
const BAR: &str = "▌";

// ── entrypoint ───────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    draw_header(f, app, root[0]);
    draw_tabs(f, app, root[1]);
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, root[2]),
        Tab::Memory => draw_memory(f, app, root[2]),
        Tab::Fields => draw_fields(f, app, root[2]),
        Tab::Signals => draw_signals(f, app, root[2]),
        Tab::System => draw_system(f, app, root[2]),
    }
    draw_footer(f, app, root[3]);

    match &app.overlay {
        Overlay::Help => draw_help(f),
        Overlay::Form(_) => draw_form(f, app),
        Overlay::None => {}
    }
}

// ── Header & Footer ──────────────────────────────────────────────────────────

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)]).split(area);
    let titles: Vec<Line> = TABS.iter().enumerate()
        .map(|(i, t)| Line::from(format!(" {} {} ", i + 1, t)))
        .collect();
    let host = app.base.strip_prefix("http://").or_else(|| app.base.strip_prefix("https://")).unwrap_or(&app.base);
    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .divider(Span::styled("·", Style::default().fg(FAINT)))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(FAINT))
            .title(Span::styled(format!(" noesis ⟡ {host} "), Style::default().fg(CORAL).bold())))
        .style(Style::default().fg(DIM))
        .highlight_style(Style::default().fg(colors::BG).bg(CORAL).bold());
    f.render_widget(tabs, cols[0]);
    let status = if let Some(h) = &app.health {
        let col = if h.status == "ok" { GREEN } else { RED };
        format!("{} ● health", if app.loading() { "◍ " } else { "" })
    } else { "connecting…".into() };
    f.render_widget(Paragraph::new(Line::from(Span::styled(status, Style::default().fg(DIM))))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(FAINT))), cols[1]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = TABS.iter().enumerate()
        .map(|(i, t)| Line::from(Span::styled(format!(" {} {} ", i + 1, t),
            if i == app.tab.index() { Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD) } else { Style::default().fg(DIM) })))
        .collect();
    f.render_widget(Tabs::new(titles).highlight_style(Style::default().add_modifier(Modifier::BOLD)), area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    if let Some((msg, is_err)) = &app.status {
        let color = if *is_err { RED } else { MINT };
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled(if *is_err { " ✗ " } else { " ✓ " }, Style::default().fg(color).bold()),
            Span::styled(truncate(msg, 220), Style::default().fg(color)),
        ])), area);
        return;
    }
    let hints = match app.tab {
        Tab::Dashboard => "r refresh · A capture",
        Tab::Memory => "h/l sub-view · ↑↓ scroll · r refresh",
        Tab::Fields => "h/l sub-view · r refresh",
        Tab::Signals => "h/l sub-view",
        Tab::System => "h/l sub-view · r refresh",
    };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" ? ", Style::default().fg(colors::BG).bg(PERI).bold()),
        Span::styled(" help ", Style::default().fg(DIM)),
        Span::styled(hints, Style::default().fg(DIM)),
        Span::styled("  ·  A capture · q quit", Style::default().fg(FAINT)),
    ])), area);
}

// ── Shared widgets ───────────────────────────────────────────────────────────

fn panel(title: &str) -> Block<'static> {
    Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
        .border_style(Style::default().fg(FAINT))
        .title(Span::styled(format!(" {title} "), Style::default().fg(PERI).bold()))
}

fn sel_style() -> Style { Style::default().bg(ratatui::style::Color::Rgb(30, 40, 52)).add_modifier(Modifier::BOLD) }

fn list_of<'a>(title: String, items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items).block(panel(&title)).highlight_style(sel_style()).highlight_symbol(BAR)
}

fn subtabs(labels: &[&str], active: usize) -> Line<'static> {
    let mut spans = vec![];
    for (i, s) in labels.iter().enumerate() {
        let style = if i == active { Style::default().fg(colors::BG).bg(MINT).bold() } else { Style::default().fg(DIM) };
        spans.push(Span::styled(format!(" {s} "), style));
        spans.push(Span::styled(" ", Style::default()));
    }
    Line::from(spans)
}

fn kv(key: &str, val: &str) -> Line<'static> {
    Line::from(vec![Span::styled(format!("  {key:<16}"), Style::default().fg(DIM)), Span::styled(val.to_string(), Style::default().fg(TEXT))])
}

fn head(t: &str) -> Line<'static> {
    Line::from(Span::styled(format!(" {t}"), Style::default().fg(CORAL).bold()))
}

fn hbar(label: &str, value: i64, max: i64, width: usize, col: ratatui::style::Color) -> Line<'static> {
    let filled = if max > 0 { (value as f64 / max as f64 * width as f64).round() as usize } else { 0 };
    let bar: String = "█".repeat(filled.min(width)) + &"░".repeat(width.saturating_sub(filled));
    Line::from(vec![
        Span::styled(format!(" {bar} "), Style::default().fg(col)),
        Span::styled(format!("{:>5}", value), Style::default().fg(col).bold()),
        Span::styled(format!(" {label:<20}"), Style::default().fg(TEXT)),
    ])
}

fn fmt_int(n: i64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}k", n as f64 / 1_000.0) }
    else { n.to_string() }
}

fn fmt_time(s: Option<&str>) -> String {
    s.and_then(|t| t.split('T').nth(1).map(|t| t[..8].to_string())).unwrap_or_else(|| "--".into())
}

fn fmt_date(s: Option<&str>) -> String {
    s.and_then(|t| t.split('T').next().map(|d| d.to_string())).unwrap_or_else(|| "—".into())
}

fn truncate(s: &str, n: usize) -> String {
    let s = s.replace('\n', " ");
    if s.chars().count() <= n { s } else { format!("{}…", s.chars().take(n).collect::<String>()) }
}

fn spark(values: &[u64]) -> String {
    if values.is_empty() { return "".into(); }
    let max = *values.iter().max().unwrap_or(&1).max(&1);
    values.iter().map(|v| {
        let idx = if max == 0 { 0 } else { (v * 7 / max) as usize };
        ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'][idx.min(7)]
    }).collect()
}

// ============================================================================
// TAB 1: DASHBOARD
// ============================================================================

fn draw_dashboard(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    draw_dash_kpis(f, app, rows[0]);

    let mid = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);
    draw_dash_fields(f, app, mid[0]);
    draw_dash_signals(f, app, mid[1]);

    draw_dash_bottom(f, app, rows[2]);
}

fn draw_dash_kpis(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.stats;
    let uptime = app.health.as_ref().map(|h| h.uptime_seconds).unwrap_or(0.0);
    let cards: [(&str, String, ratatui::style::Color); 6] = [
        ("FIELDS", fmt_int(s.fields as i64), GREEN),
        ("PROCESSORS", fmt_int(s.processors as i64), CYAN),
        ("SIGNAL TYPES", fmt_int(s.signal_types as i64), PERI),
        ("SIGNALS TOTAL", fmt_int(s.signals_total as i64), LIME),
        ("CASCADES", fmt_int(s.cascade_cycles as i64), PURPLE),
        ("UPTIME", format!("{:.0}s", uptime), AMBER),
    ];
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(1, 6); 6]).split(area);
    for (i, (label, val, col)) in cards.iter().enumerate() {
        f.render_widget(Paragraph::new(vec![
            Line::from(Span::styled(*label, Style::default().fg(DIM))),
            Line::from(Span::styled(val.clone(), Style::default().fg(*col).bold())),
        ]).alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(FAINT))), cols[i]);
    }
}

fn draw_dash_fields(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![head("Fields — live status")];
    for name in ["identity", "memory", "agency", "awareness", "reasoning", "simulation", "graph", "core"] {
        let detail = app.field_details.get(name);
        let has_data = detail.is_some_and(|d| !d.is_null() && d.as_object().is_some_and(|o| o.len() > 1));
        let dot = if has_data { "●" } else { "○" };
        let col = if has_data { MINT } else { FAINT };
        lines.push(Line::from(vec![
            Span::styled(format!(" {dot} {name:<12}", dot=dot, name=name), Style::default().fg(col).bold()),
            Span::styled(if has_data { "loaded" } else { "waiting…" }, Style::default().fg(if has_data { TEXT } else { DIM })),
        ]));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Field Status")).wrap(Wrap { trim: true }), area);
}

fn draw_dash_signals(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![head("Signal distribution — top by count")];
    let sigs = app.observability.as_ref().and_then(|o| match &o.signals_processed {
        serde_json::Value::Object(m) => Some(m),
        _ => None,
    });
    if let Some(map) = sigs {
        let mut sorted: Vec<_> = map.iter().collect();
        sorted.sort_by(|a, b| b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0)));
        let max = sorted.first().map(|(_, v)| v.as_u64().unwrap_or(1)).unwrap_or(1).max(1);
        for (k, v) in sorted.iter().take(10) {
            let val = v.as_u64().unwrap_or(0);
            lines.push(hbar(k, val as i64, max as i64, 16, type_color(k)));
        }
    } else {
        lines.push(Line::from(Span::styled("  No signal data yet", Style::default().fg(DIM))));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Signals")).wrap(Wrap { trim: true }), area);
}

fn draw_dash_bottom(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left: signal metrics (if available) + activity
    let mut left = vec![head("Signal metrics")];
    if let Some(sm) = &app.signal_metrics {
        let top: Vec<_> = sm.signals.iter().take(5).collect();
        for (k, v) in &top {
            left.push(kv(k, &fmt_int(**v as i64)));
        }
    }
    left.push(Line::from(Span::raw("")));
    left.push(Line::from(Span::styled(
        " Keys: 1-5 tabs | h/l sub-view | r refresh | A capture | q quit ",
        Style::default().fg(FAINT),
    )));

    // Right: quick stats + health
    let mut right = vec![head("System health")];
    if let Some(h) = &app.health {
        right.push(kv("status", &h.status));
        if let Some(v) = &h.version { right.push(kv("version", v)); }
        if let Some(pg) = &h.postgres { right.push(kv("postgres", pg)); }
        if let Some(rd) = &h.redis { right.push(kv("redis", rd)); }
    }
    right.push(kv("fields", &app.stats.fields.to_string()));
    right.push(kv("processors", &app.stats.processors.to_string()));
    right.push(kv("signal types", &app.stats.signal_types.to_string()));

    f.render_widget(Paragraph::new(left).block(panel("Metrics")).wrap(Wrap { trim: true }), cols[0]);
    f.render_widget(Paragraph::new(right).block(panel("Health")).wrap(Wrap { trim: true }), cols[1]);
}

// ============================================================================
// TAB 2: MEMORY
// ============================================================================

fn draw_memory(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(subtabs(&MEMORY_SUBS, app.sub_idx)), rows[0]);
    match app.sub_idx {
        0 => draw_memory_browse(f, app, rows[1]),
        1 => draw_memory_episodes(f, app, rows[1]),
        _ => draw_memory_history(f, app, rows[1]),
    }
}

fn draw_memory_browse(f: &mut Frame, app: &App, area: Rect) {
    let detail = app.field_details.get("memory");
    let mut lines = vec![];
    if let Some(d) = detail {
        let ep_count = d.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
        let entities = d.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);
        let mem_count = d.pointer("/memories/count").and_then(|c| c.as_u64())
            .or_else(|| d.pointer("/semantic/count").and_then(|c| c.as_u64())).unwrap_or(0);
        lines.push(kv("episodes", &fmt_int(ep_count as i64)));
        lines.push(kv("entities", &fmt_int(entities as i64)));
        lines.push(kv("memories", &fmt_int(mem_count as i64)));
        lines.push(Line::from(Span::raw("")));

        // Show episode items
        if let Some(items) = d.pointer("/episodic/items").and_then(|a| a.as_array()) {
            lines.push(head(&format!("Episodes ({})", items.len())));
            for ep in items.iter().take(8) {
                let content = ep.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let ts = ep.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", fmt_time(Some(ts))), Style::default().fg(FAINT)),
                    Span::styled(truncate(content, 80), Style::default().fg(TEXT)),
                ]));
            }
        } else {
            lines.push(Line::from(Span::styled("  No episodes yet", Style::default().fg(DIM))));
        }
    } else {
        lines.push(Line::from(Span::styled("  Not loaded — press r to refresh", Style::default().fg(DIM))));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Memory Browse")).wrap(Wrap { trim: true }), area);
}

fn draw_memory_episodes(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = app.field_details.get("episodes") {
        let items = d.pointer("/episodes_from_cache/episodes").and_then(|a| a.as_array())
            .or_else(|| d.as_object().and_then(|m| m.get("episodes").and_then(|a| a.as_array())));
        if let Some(eps) = items {
            for ep in eps.iter().take(20) {
                let content = ep.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let ts = ep.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
                let src = ep.get("source").and_then(|s| s.as_str()).unwrap_or("");
                lines.push(Line::from(vec![
                    Span::styled(format!(" {} ", fmt_time(Some(ts))), Style::default().fg(FAINT)),
                    Span::styled(truncate(content, 80), Style::default().fg(TEXT)),
                ]));
                if !src.is_empty() {
                    lines.push(Line::from(Span::styled(format!("    source: {src}"), Style::default().fg(DIM))));
                }
            }
        } else {
            lines.push(Line::from(Span::styled("  No episodes data available", Style::default().fg(DIM))));
        }
    } else {
        lines.push(Line::from(Span::styled("  Press r to load episodes", Style::default().fg(DIM))));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Episodes")).wrap(Wrap { trim: true }), area);
}

fn draw_memory_history(f: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    let items: Vec<ListItem> = app.signal_history.iter().map(|e| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", fmt_time(e.timestamp.as_deref())), Style::default().fg(FAINT)),
            Span::styled(format!("{:<34}", e.signal_type), Style::default().fg(CYAN)),
            Span::styled(truncate(&e.source, 20), Style::default().fg(DIM)),
        ]))
    }).collect();
    if items.is_empty() {
        f.render_widget(Paragraph::new("  No signal history").block(panel("History")).wrap(Wrap { trim: true }), area);
    } else {
        f.render_stateful_widget(list_of(format!("History · {}", app.signal_history.len()), items), cols[0], &mut app.history_sel.state);
        let prev = app.history_sel.selected().and_then(|i| app.signal_history.get(i))
            .map(|e| Text::from(json_lines(&e.data, 0)))
            .unwrap_or_else(|| Text::from("select an event"));
        f.render_widget(Paragraph::new(prev).block(panel("Detail")).wrap(Wrap { trim: true }), cols[1]);
    }
}

// ============================================================================
// TAB 3: FIELDS
// ============================================================================

fn draw_fields(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(subtabs(&FIELD_SUBS, app.sub_idx)), rows[0]);
    let name = FIELD_SUBS[app.sub_idx].to_lowercase();
    let detail = app.field_details.get(&name);

    if app.sub_idx == 0 { draw_fields_overview(f, app, rows[1]); return; }
    match FIELD_SUBS[app.sub_idx] {
        "Identity" => render_identity_field(f, detail, rows[1]),
        "Memory" => render_memory_field(f, detail, rows[1]),
        "Agency" => render_agency_field(f, detail, rows[1]),
        "Awareness" => render_awareness_field(f, detail, rows[1]),
        "Reasoning" => render_reasoning_field(f, detail, rows[1]),
        "Simulation" => render_simulation_field(f, detail, rows[1]),
        "Graph" => render_graph_field(f, detail, rows[1]),
        "Core" => render_core_field(f, detail, rows[1]),
        _ => { f.render_widget(Paragraph::new("Select a field").block(panel("Fields")), rows[1]); }
    }
}

fn count_of(d: &Option<&serde_json::Value>, prefix: &str, counter: &str) -> i64 {
    d.and_then(|d| d.as_object()).and_then(|m| m.get(prefix))
        .and_then(|s| s.get(counter)).and_then(|c| c.as_i64()).unwrap_or(0)
}

fn draw_fields_overview(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![head("All fields — h/l to browse individual field details")];
    lines.push(Line::from(Span::raw("")));
    for name in ["identity", "memory", "agency", "awareness", "reasoning", "simulation", "graph", "core"] {
        let detail = app.field_details.get(name);
        let has_data = detail.is_some_and(|d| !d.is_null());
        let icon = if has_data { "●" } else { "○" }; let col = if has_data { MINT } else { FAINT };
        lines.push(Line::from(vec![
            Span::styled(format!(" {icon} {name:<12}"), Style::default().fg(col).bold()),
            Span::styled(if let Some(d) = detail { summarize_value(d) } else { "—".into() }, Style::default().fg(if has_data { TEXT } else { DIM })),
        ]));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Fields Overview")).wrap(Wrap { trim: true }), area);
}

fn summarize_value(v: &serde_json::Value) -> String {
    match v { serde_json::Value::Object(map) => map.keys().take(4).cloned().collect::<Vec<_>>().join(", "), other => format!("{other}") }
}

fn render_identity_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(_d) = detail {
        lines.push(kv("version", &format!("v{}", count_of(&detail, "identity", "version"))));
        lines.push(kv("beliefs", &fmt_int(count_of(&detail, "beliefs", "count"))));
        lines.push(kv("traits", &fmt_int(count_of(&detail, "traits", "count"))));
        lines.push(kv("principles", &fmt_int(count_of(&detail, "principles", "count"))));
        lines.push(kv("preferences", &fmt_int(count_of(&detail, "preferences", "count"))));
    } else {
        lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM))));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Identity")).wrap(Wrap { trim: true }), area);
}

fn render_memory_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = detail {
        let ep_count = count_of(&detail, "episodic", "count");
        let entities = count_of(&detail, "graph", "entities");
        let relations = count_of(&detail, "graph", "relations");
        let mem_count = count_of(&detail, "semantic", "count").max(count_of(&detail, "memories", "count"));
        let consolid = d.pointer("/consolidation/status").and_then(|s| s.as_str()).unwrap_or("-");

        lines.push(kv("episodes", &fmt_int(ep_count)));
        lines.push(kv("entities", &fmt_int(entities)));
        lines.push(kv("relations", &fmt_int(relations)));
        lines.push(kv("memories", &fmt_int(mem_count)));
        lines.push(kv("consolidation", consolid));
        if let Some(items) = d.pointer("/episodic/items").and_then(|a| a.as_array()) {
            if !items.is_empty() {
                lines.push(Line::from(Span::raw(""))); lines.push(head("Recent episodes"));
                for ep in items.iter().take(3) {
                    let content = ep.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let ts = ep.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", fmt_time(Some(ts))), Style::default().fg(FAINT)),
                        Span::styled(truncate(content, 80), Style::default().fg(TEXT)),
                    ]));
                }
            }
        }
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Memory")).wrap(Wrap { trim: true }), area);
}

fn render_agency_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(_d) = detail {
        let active = count_of(&detail, "goals", "active");
        let total = count_of(&detail, "goals", "total").max(active);
        let abandoned = count_of(&detail, "goals", "abandoned");
        let pursuits = count_of(&detail, "active_pursuits", "count");
        let eval_count = count_of(&detail, "evaluation", "count");
        lines.push(kv("goals", &format!("{} active / {} total", active, total.max(active))));
        if abandoned > 0 { lines.push(kv("abandoned", &fmt_int(abandoned))); }
        lines.push(kv("pursuits", &fmt_int(pursuits)));
        lines.push(kv("evaluations", &fmt_int(eval_count)));
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Agency")).wrap(Wrap { trim: true }), area);
}

fn render_awareness_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = detail {
        let sig_count = d.pointer("/_meta/signals_processed").and_then(|c| c.as_u64()).unwrap_or(0);
        let focus_depth = count_of(&detail, "attention", "focus_depth").max(count_of(&detail, "focus_stack", "depth"));
        let transitions = count_of(&detail, "observer", "count").max(count_of(&detail, "transitions", "count"));
        let health = d.pointer("/health/status").and_then(|s| s.as_str()).unwrap_or("--");
        let curiosity = count_of(&detail, "curiosity", "count");
        let mood_samples = count_of(&detail, "mood", "count");
        lines.push(kv("signals tracked", &fmt_int(sig_count as i64)));
        lines.push(kv("transitions", &fmt_int(transitions)));
        lines.push(kv("focus depth", &fmt_int(focus_depth)));
        lines.push(kv("curiosity", &fmt_int(curiosity)));
        lines.push(kv("mood samples", &fmt_int(mood_samples)));
        lines.push(kv("health", health));
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Awareness")).wrap(Wrap { trim: true }), area);
}

fn render_reasoning_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = detail {
        let beliefs = d.get("beliefs").and_then(|b| b.as_array()).map(|a| a.len()).unwrap_or(0);
        let principles = d.get("principles").and_then(|b| b.as_array()).map(|a| a.len()).unwrap_or(0);
        let goals = d.get("goals").and_then(|b| b.as_array()).map(|a| a.len()).unwrap_or(0);
        let assumptions = d.get("assumptions").and_then(|b| b.as_array()).map(|a| a.len()).unwrap_or(0);
        lines.push(kv("beliefs", &fmt_int(beliefs as i64)));
        lines.push(kv("principles", &fmt_int(principles as i64)));
        lines.push(kv("goals", &fmt_int(goals as i64)));
        lines.push(kv("assumptions", &fmt_int(assumptions as i64)));
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Reasoning")).wrap(Wrap { trim: true }), area);
}

fn render_simulation_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(_d) = detail {
        let scenarios = count_of(&detail, "scenarios", "count");
        let forecasts = count_of(&detail, "forecasts", "count");
        let assumptions = count_of(&detail, "assumptions", "count");
        let risks = count_of(&detail, "risk", "count").max(count_of(&detail, "risk_assessments", "count"));
        lines.push(kv("scenarios", &fmt_int(scenarios)));
        lines.push(kv("forecasts", &fmt_int(forecasts)));
        lines.push(kv("assumptions", &fmt_int(assumptions)));
        lines.push(kv("risks", &fmt_int(risks)));
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Simulation")).wrap(Wrap { trim: true }), area);
}

fn render_graph_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = detail {
        let entities = d.get("entity_count").and_then(|c| c.as_u64()).or_else(|| d.get("entities").and_then(|a| a.as_array().map(|a| a.len() as u64))).unwrap_or(0);
        let relations = d.get("relation_count").and_then(|c| c.as_u64()).or_else(|| d.get("relations").and_then(|a| a.as_array().map(|a| a.len() as u64))).or_else(|| d.get("links").and_then(|a| a.as_array().map(|a| a.len() as u64))).unwrap_or(0);
        let density = if entities > 0 { relations as f64 / entities as f64 } else { 0.0 };
        lines.push(kv("entities", &fmt_int(entities as i64)));
        lines.push(kv("relations", &fmt_int(relations as i64)));
        lines.push(kv("density", &format!("{density:.2}")));
        if let Some(nodes) = d.get("nodes").and_then(|a| a.as_array()) {
            let top: Vec<&serde_json::Value> = nodes.iter().take(5).collect();
            if !top.is_empty() {
                lines.push(Line::from(Span::raw(""))); lines.push(head("Top entities"));
                for n in top {
                    let name = n.get("name").and_then(|s| s.as_str()).unwrap_or("?");
                    let deg = n.get("degree").and_then(|c| c.as_i64()).unwrap_or(0);
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", fmt_int(deg)), Style::default().fg(CYAN).bold()),
                        Span::styled(truncate(name, 40), Style::default().fg(TEXT)),
                    ]));
                }
            }
        }
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Knowledge Graph")).wrap(Wrap { trim: true }), area);
}

fn render_core_field(f: &mut Frame, detail: Option<&serde_json::Value>, area: Rect) {
    let mut lines = vec![];
    if let Some(d) = detail {
        let kernel = d.pointer("/config/rest_api_enabled").and_then(|b| b.as_bool()).unwrap_or(false);
        let bus_sigs = d.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
        let runtime = d.pointer("/runtime/tasks_count").and_then(|c| c.as_u64()).unwrap_or(0);
        let fields = d.pointer("/config/fields_count").and_then(|c| c.as_u64()).unwrap_or(0);
        let plugins = d.pointer("/config/plugins_count").and_then(|c| c.as_u64()).unwrap_or(0);
        lines.push(kv("kernel", if kernel { "running" } else { "stopped" }));
        lines.push(kv("bus signals", &fmt_int(bus_sigs as i64)));
        lines.push(kv("tasks", &fmt_int(runtime as i64)));
        lines.push(kv("fields", &fmt_int(fields as i64)));
        lines.push(kv("plugins", &fmt_int(plugins as i64)));
    } else { lines.push(Line::from(Span::styled("  Not loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Core")).wrap(Wrap { trim: true }), area);
}

// ============================================================================
// TAB 4: SIGNALS
// ============================================================================

fn draw_signals(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(subtabs(&SIGNAL_SUBS, app.sub_idx)), rows[0]);
    match app.sub_idx {
        0 => draw_signal_types(f, app, rows[1]),
        1 => draw_signal_distribution(f, app, rows[1]),
        _ => draw_proc_table(f, app, rows[1]),
    }
}

fn draw_signal_types(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    for st in &app.signal_types {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<36}", st.signal_type), Style::default().fg(CYAN).bold()),
            Span::styled(truncate(&st.description, 60), Style::default().fg(DIM)),
        ]));
    }
    if lines.is_empty() { lines.push(Line::from(Span::styled("  No signal types loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel(&format!("Signal Types ({})", app.signal_types.len()))).wrap(Wrap { trim: true }), area);
}

fn draw_signal_distribution(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    if let Some(sm) = &app.signal_metrics {
        let mut sorted: Vec<_> = sm.signals.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let max_val = sorted.first().map(|(_, v)| **v).unwrap_or(1);
        for (k, v) in sorted.iter().take(20) {
            lines.push(hbar(k, **v as i64, max_val as i64, 16, type_color(k)));
        }
        lines.push(Line::from(Span::raw("")));
        lines.push(kv("total signals", &fmt_int(sm.total as i64)));
    } else { lines.push(Line::from(Span::styled("  No signal metrics yet", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Signal Distribution")).wrap(Wrap { trim: true }), area);
}

fn draw_proc_table(f: &mut Frame, app: &App, area: Rect) {
    if app.processor_metrics.is_empty() {
        f.render_widget(Paragraph::new("  No processor metrics yet").block(panel("Processors")), area);
        return;
    }
    let mut lines = vec![Line::from(vec![
        Span::styled("  NAME", Style::default().fg(DIM).bold()),
        Span::styled(format!("{:>8}", "CALLS"), Style::default().fg(DIM).bold()),
        Span::styled(format!("  {:>8}", "AVG LAT"), Style::default().fg(DIM).bold()),
    ])];
    let sorted = { let mut v = app.processor_metrics.clone(); v.sort_by(|a, b| b.count.cmp(&a.count)); v };
    for pm in &sorted {
        let col = if pm.count > 0 { GREEN } else { DIM };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<18}", pm.name), Style::default().fg(col).bold()),
            Span::styled(format!("{:>8}", pm.count), Style::default().fg(LIME).bold()),
            Span::styled(format!("{:>8}ms", pm.avg_latency_ms), Style::default().fg(if pm.avg_latency_ms > 0 { CYAN } else { DIM })),
        ]));
    }
    f.render_widget(Paragraph::new(lines).block(panel(&format!("Processor Dispatch ({})", sorted.len()))).wrap(Wrap { trim: true }), area);
}

// ============================================================================
// TAB 5: SYSTEM
// ============================================================================

fn draw_system(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(subtabs(&SYSTEM_SUBS, app.sub_idx)), rows[0]);
    match app.sub_idx {
        0 => draw_sys_config(f, app, rows[1]),
        1 => draw_sys_plugins(f, app, rows[1]),
        2 => draw_sys_observability(f, app, rows[1]),
        3 => draw_sys_cascade(f, app, rows[1]),
        _ => draw_sys_log(f, app, rows[1]),
    }
}

fn draw_sys_config(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    if let Some(cfg) = &app.config {
        lines.push(kv("rest api", if cfg.rest_api_enabled { "enabled" } else { "disabled" }));
        lines.push(kv("port", &cfg.port.to_string()));
        lines.push(kv("storage", &cfg.storage_backend));
        if !cfg.settings.is_empty() {
            lines.push(Line::from(Span::raw("")));
            lines.push(head("Raw config"));
            for (k, v) in &cfg.settings { lines.push(kv(k, &format!("{v}"))); }
        }
    } else { lines.push(Line::from(Span::styled("  Press r to load config", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Config")).wrap(Wrap { trim: true }), area);
}

fn draw_sys_plugins(f: &mut Frame, app: &App, area: Rect) {
    if app.plugins.is_empty() {
        f.render_widget(Paragraph::new("  No plugins loaded").block(panel("Plugins")), area);
        return;
    }
    let mut lines = vec![];
    for p in &app.plugins {
        lines.push(Line::from(vec![
            Span::styled(format!("  ● {}", p.name), Style::default().fg(GREEN).bold()),
            Span::styled(format!(" v{}", p.version), Style::default().fg(DIM)),
        ]));
        if !p.description.is_empty() { lines.push(Line::from(Span::styled(format!("     {}", truncate(&p.description, 80)), Style::default().fg(TEXT)))); }
        if !p.capabilities.is_empty() { lines.push(Line::from(Span::styled(format!("     caps: {}", p.capabilities.join(", ")), Style::default().fg(FAINT)))); }
        lines.push(Line::from(Span::raw("")));
    }
    f.render_widget(Paragraph::new(lines).block(panel(&format!("Plugins ({})", app.plugins.len()))).wrap(Wrap { trim: true }), area);
}

fn draw_sys_observability(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let mut left = vec![head("Overview")];
    if let Some(o) = &app.observability {
        left.push(kv("fields", &o.fields.map(|f| f.to_string()).unwrap_or_else(|| "N/A".into())));
        left.push(kv("processors", &o.processors.map(|p| p.to_string()).unwrap_or_else(|| "N/A".into())));
        left.push(kv("signal types", &o.signal_types.to_string()));
        left.push(kv("uptime", &format!("{:.0}s", o.uptime_seconds)));
    }
    let mut right = vec![head("Capabilities")];
    for cap in &app.capabilities {
        right.push(Line::from(vec![
            Span::styled(format!("  ● {}", cap.id), Style::default().fg(PERI).bold()),
            Span::styled(format!("  {:.0}%", cap.confidence * 100.0), Style::default().fg(DIM)),
        ]));
    }
    if app.capabilities.is_empty() { right.push(Line::from(Span::styled("  No capabilities loaded", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(left).block(panel("Observability")).wrap(Wrap { trim: true }), cols[0]);
    f.render_widget(Paragraph::new(right).block(panel("Capabilities")).wrap(Wrap { trim: true }), cols[1]);
}

fn draw_sys_cascade(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    if let Some(ct) = &app.cascade_trace {
        lines.push(kv("depth", &ct.depth.to_string()));
        lines.push(kv("duration", &format!("{:.1}ms", ct.duration_ms)));
        if !ct.signals.is_empty() {
            lines.push(Line::from(Span::raw("")));
            lines.push(head("Signal chain"));
            for sig in &ct.signals {
                lines.push(Line::from(Span::styled(format!("  → {sig}"), Style::default().fg(CYAN))));
            }
        }
    } else { lines.push(Line::from(Span::styled("  No cascade trace yet", Style::default().fg(DIM)))); }
    f.render_widget(Paragraph::new(lines).block(panel("Cascade Trace")).wrap(Wrap { trim: true }), area);
}

fn draw_sys_log(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app.log_entries.iter().rev().take(200).rev()
        .map(|e| Line::from(Span::styled(e.clone(), Style::default().fg(TEXT)))).collect();
    let content = if lines.is_empty() { Text::from(Span::styled("  No log entries yet", Style::default().fg(DIM))) } else { Text::from(lines) };
    f.render_widget(Paragraph::new(content).block(panel(&format!("Log ({})", app.log_entries.len()))).wrap(Wrap { trim: false }), area);
}

// ============================================================================
// OVERLAYS
// ============================================================================

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(" Noesis TUI — Help", Style::default().fg(CORAL).bold())),
        Line::from(Span::raw("")),
        Line::from(Span::styled(" Navigation", Style::default().fg(PERI).bold())),
        kv("1-5", "Switch tabs"), kv("h/l or ←/→", "Cycle sub-views"),
        kv("j/k or ↑/↓", "Navigate lists"), kv("r", "Refresh"),
        Line::from(Span::raw("")),
        Line::from(Span::styled(" Actions", Style::default().fg(PERI).bold())),
        kv("A", "Ingest / capture"), kv("?", "Help"), kv("q / Ctrl-C", "Quit"),
        Line::from(Span::raw("")),
        Line::from(Span::styled(" Press any key to close", Style::default().fg(DIM))),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CORAL)).title(" Help ")).alignment(Alignment::Left), area);
}

fn draw_form(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 10, f.area());
    f.render_widget(Clear, area);
    let lines = if let Overlay::Form(form) = &app.overlay {
        let mut lines = vec![];
        for (i, field) in form.fields.iter().enumerate() {
            let active = i == form.active;
            let prefix = if active { " ▸ " } else { "   " };
            let style = if active { Style::default().fg(GREEN).bold() } else { Style::default().fg(DIM) };
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{}: ", field.label), style),
                Span::styled(if field.value.is_empty() && !active { "—".into() } else { field.value.clone() }, if active { Style::default().fg(TEXT) } else { Style::default().fg(DIM) }),
            ]));
        }
        lines.push(Line::from(Span::styled("  Ctrl-S to submit · Enter newline · Esc cancel", Style::default().fg(FAINT))));
        lines
    } else { vec![] };
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MINT)).title(" Form ")), area);
}

fn centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    Rect::new(r.x + r.width.saturating_sub(w) / 2, r.y + r.height.saturating_sub(h) / 2, w.min(r.width), h.min(r.height))
}

fn json_lines(v: &serde_json::Value, indent: usize) -> Vec<Line<'static>> {
    let mut lines = vec![];
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                let p = "  ".repeat(indent);
                match val {
                    serde_json::Value::String(s) => lines.push(Line::from(Span::styled(format!("{p}{k}: {s}"), Style::default().fg(TEXT)))),
                    serde_json::Value::Number(n) => lines.push(Line::from(Span::styled(format!("{p}{k}: {n}"), Style::default().fg(LIME)))),
                    serde_json::Value::Bool(b) => lines.push(Line::from(Span::styled(format!("{p}{k}: {b}"), Style::default().fg(CYAN)))),
                    serde_json::Value::Array(arr) => {
                        lines.push(Line::from(Span::styled(format!("{p}{k}: [{}]", arr.len()), Style::default().fg(DIM))));
                        for item in arr.iter().take(5) { lines.extend(json_lines(item, indent + 1)); }
                    }
                    serde_json::Value::Object(_) => { lines.push(Line::from(Span::styled(format!("{p}{k}:"), Style::default().fg(DIM)))); lines.extend(json_lines(val, indent + 1)); }
                    serde_json::Value::Null => {}
                }
            }
        }
        _ => {}
    }
    lines
}

fn type_color(t: &str) -> ratatui::style::Color {
    if t.contains("identity") || t.contains("belief") { PERI }
    else if t.contains("memory") || t.contains("episode") { GREEN }
    else if t.contains("agency") || t.contains("goal") { LIME }
    else if t.contains("awareness") || t.contains("attention") { CYAN }
    else if t.contains("reasoning") { PURPLE }
    else if t.contains("simulation") { AMBER }
    else if t.contains("graph") || t.contains("entity") { CORAL }
    else if t.contains("kernel") || t.contains("beat") { DIM }
    else { MINT }
}
