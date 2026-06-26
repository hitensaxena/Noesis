//! Settings screen — configuration and system info.
//!
//! Shows connection status, refresh controls, feature toggles,
//! and system information from the detail endpoints.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Gauge},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::tui::{app::{TuiApp, Screen, DETAIL_NAMES}, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(3),     // content
            Constraint::Length(3),  // keybindings
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_keys(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let text = vec![
        Line::from(vec![
            Span::styled("Noesis TUI v0.1.0", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" — "),
            Span::styled("Decentralized Cognitive Architecture", Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_content(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_connection_panel(f, app, chunks[0]);
    render_system_panel(f, app, chunks[1]);
}

fn render_connection_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Connection & Refresh ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let status = if app.status_message.starts_with("OK") {
        "Connected"
    } else if app.status_message.contains("error") {
        "Error"
    } else {
        "Unknown"
    };
    let status_color = if status == "Connected" { colors::GREEN } else if status == "Error" { colors::RED } else { colors::YELLOW };

    let lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(colors::DIM)),
            Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("API URL: ", Style::default().fg(colors::DIM)),
            Span::raw(&app.api_url),
        ]),
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(colors::DIM)),
            Span::styled(&app.status_message, Style::default().fg(colors::TEXT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Auto-refresh: ", Style::default().fg(colors::DIM)),
            Span::styled(if app.auto_refresh { "ON  [r]" } else { "OFF [r]" }, Style::default().fg(if app.auto_refresh { colors::GREEN } else { colors::RED }).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Interval: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}s  [+/-]", app.refresh_interval_secs()), Style::default().fg(colors::YELLOW)),
        ]),
        Line::from(vec![
            Span::styled("Screen: ", Style::default().fg(colors::DIM)),
            Span::raw(format!("{} / {}", app.screen.name(), Screen::all().len())),
        ]),
        Line::from(vec![
            Span::styled("Detail view: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{} ({}/{})", DETAIL_NAMES[app.detail_index], app.detail_index + 1, DETAIL_NAMES.len()), Style::default().fg(colors::ACCENT)),
        ]),
    ];

    // Refresh interval gauge
    let gauge_title = format!("Refresh Interval: {}s", app.refresh_interval_secs());
    let gauge = Gauge::default()
        .block(Block::default().title(gauge_title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(colors::PRIMARY))
        .percent(((app.refresh_interval_secs() as f64 / 30.0) * 100.0) as u16);

    let inner = block.inner(area);
    f.render_widget(block, area);
    let gauge_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(inner);

    f.render_widget(Paragraph::new(lines), gauge_area[0]);
    f.render_widget(gauge, gauge_area[1]);
}

fn render_system_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_system_info(f, app, chunks[0]);
    render_detail_snapshot(f, app, chunks[1]);
}

fn render_system_info(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" System Info ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let core = &app.core_detail;
    let kernel_status = core.pointer("/kernel/status").and_then(|s| s.as_str()).unwrap_or("—");
    let uptime = core.pointer("/kernel/uptime_secs").and_then(|u| u.as_u64()).unwrap_or(0);
    let bus_subs = core.pointer("/event_bus/subscribers").and_then(|c| c.as_u64()).unwrap_or(0);
    let bus_sigs = core.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let runtime_tasks = core.pointer("/runtime/tasks_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let permissions = core.pointer("/permissions/mode").and_then(|s| s.as_str()).unwrap_or("—");

    let obs = &app.obs;
    let fields = obs.get("fields").and_then(|v| v.as_i64()).unwrap_or(0);
    let processors = obs.get("processors").and_then(|v| v.as_i64()).unwrap_or(0);
    let signal_types = obs.get("signal_types").and_then(|v| v.as_i64()).unwrap_or(0);

    let lines = vec![
        Line::from(vec![
            Span::styled("Service: noesis ", Style::default().fg(colors::TEXT)),
            Span::styled("v0.1.0", Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled("Kernel: ", Style::default().fg(colors::DIM)),
            Span::styled(kernel_status, Style::default().fg(if kernel_status == "running" { colors::GREEN } else { colors::YELLOW })),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}s", uptime), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(format!("{} fields", fields), Style::default().fg(colors::GREEN)),
            Span::raw("  "),
            Span::styled(format!("{} processors", processors), Style::default().fg(colors::ACCENT)),
            Span::raw("  "),
            Span::styled(format!("{} signal types", signal_types), Style::default().fg(colors::YELLOW)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Event bus subscribers: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", bus_subs), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Bus signal count: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", bus_sigs), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Runtime tasks: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", runtime_tasks), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Permissions: ", Style::default().fg(colors::DIM)),
            Span::styled(permissions, Style::default().fg(if permissions == "open" { colors::YELLOW } else { colors::GREEN })),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_detail_snapshot(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Field Data Snapshot ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mem = &app.memory_detail;
    let episodes = mem.pointer("/episodic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let semantic = mem.pointer("/semantic/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let graph_ents = mem.pointer("/graph/entities").and_then(|c| c.as_u64()).unwrap_or(0);

    let id = &app.identity_detail;
    let beliefs = id.pointer("/beliefs/count").and_then(|c| c.as_u64()).unwrap_or(0);
    let traits = id.pointer("/traits/count").and_then(|c| c.as_u64()).unwrap_or(0);

    let exec = &app.agency_detail;
    let goals = exec.pointer("/goals/total").and_then(|c| c.as_u64()).unwrap_or(0);
    let pursuits = exec.pointer("/active_pursuits/count").and_then(|c| c.as_u64()).unwrap_or(0);

    let aware = &app.awareness_detail;
    let focus = aware.pointer("/attention/current_focus").and_then(|s| s.as_str()).unwrap_or("none");

    let lines = vec![
        Line::from(vec![
            Span::styled("Memory", Style::default().fg(colors::GREEN).add_modifier(Modifier::BOLD)),
            Span::raw(format!(": {} episodes, {} semantic, {} graph entities", episodes, semantic, graph_ents)),
        ]),
        Line::from(vec![
            Span::styled("Identity", Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(format!(": {} beliefs, {} traits", beliefs, traits)),
        ]),
        Line::from(vec![
            Span::styled("Agency", Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD)),
            Span::raw(format!(": {} goals, {} pursuits", goals, pursuits)),
        ]),
        Line::from(vec![
            Span::styled("Awareness", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::raw(format!(": focus=\"{}\"", focus)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_keys(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Key Bindings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::DIM));

    let text = vec![
        Line::from(vec![
            Span::styled(" Tab/←→ ", Style::default().fg(colors::PRIMARY)), Span::raw("Navigate screens  "),
            Span::styled(" r ", Style::default().fg(colors::GREEN)), Span::raw("Refresh data  "),
            Span::styled(" q/ESC ", Style::default().fg(colors::RED)), Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled(" ↑/↓ ", Style::default().fg(colors::ACCENT)), Span::raw("Detail cycling  "),
            Span::styled(" + ", Style::default().fg(colors::YELLOW)), Span::raw("Faster refresh  "),
            Span::styled(" - ", Style::default().fg(colors::YELLOW)), Span::raw("Slower refresh  "),
        ]),
        Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(colors::PRIMARY)), Span::raw("Manual refresh  "),
            Span::styled(" a ", Style::default().fg(colors::ACCENT)), Span::raw("Toggle auto-refresh"),
        ]),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}
