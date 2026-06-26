//! Signals screen — signal types, per-type counts, and core event bus info.

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

    render_signal_types(f, app, chunks[0]);
    render_signal_counts_and_bus(f, app, chunks[1]);
}

fn render_signal_types(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Signal Types ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(types) = app.signal_types.get("signal_types").and_then(|v| v.as_array()) {
        for st in types {
            let name = st.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let desc = st.get("description").and_then(|v| v.as_str()).unwrap_or("");

            // Color by domain prefix
            let color = if name.starts_with("memory.") { colors::GREEN }
                else if name.starts_with("belief.") || name.starts_with("identity.") { colors::ACCENT }
                else if name.starts_with("goal.") || name.starts_with("decision.") { colors::YELLOW }
                else if name.starts_with("attention.") || name.starts_with("curiosity.") { colors::PRIMARY }
                else if name.starts_with("narrative.") { colors::DIM }
                else { colors::TEXT };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}", name), Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(desc, Style::default().fg(colors::DIM)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_signal_counts_and_bus(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_signal_counts(f, app, chunks[0]);
    render_event_bus_info(f, app, chunks[1]);
}

fn render_signal_counts(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Signal Counts ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mut lines: Vec<Line> = Vec::new();
    let signals = app.signals.get("signals").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let mut sorted: Vec<_> = signals.iter().filter(|(k, _)| *k != "cascade.dispatch").collect();
    sorted.sort_by(|a, b| b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0)));

    for (name, count) in &sorted {
        let c = count.as_u64().unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>6} ", c), Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD)),
            Span::raw(name.as_str()),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No signals processed yet", colors::DIM)));
    } else {
        // Total
        let total: u64 = sorted.iter().map(|(_, c)| c.as_u64().unwrap_or(0)).sum();
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::styled(format!("  Total: {} signals", total), Style::default().fg(colors::ACCENT).add_modifier(Modifier::BOLD)),
        ]));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_event_bus_info(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Event Bus ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let core = &app.core_detail;
    let subscribers = core.pointer("/event_bus/subscribers").and_then(|c| c.as_u64()).unwrap_or(0);
    let signal_count = core.pointer("/event_bus/signal_count").and_then(|c| c.as_u64()).unwrap_or(0);
    let mode = core.pointer("/event_bus/mode").and_then(|s| s.as_str()).unwrap_or("—");
    let channels = core.pointer("/event_bus/channels").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

    let scheduler = core.pointer("/scheduler/tasks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

    let lines = vec![
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(colors::DIM)),
            Span::styled(mode, Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Subscribers: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", subscribers), Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::styled("Channels: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", channels), Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("Signal count: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", signal_count), Style::default().fg(colors::YELLOW)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Scheduler tasks: ", Style::default().fg(colors::DIM)),
            Span::styled(format!("{}", scheduler), Style::default().fg(colors::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Note: ", Style::default().fg(colors::DIM)),
            Span::styled("Broadcast channels per signal type, fanned into mpsc for sequential cascade", Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}
