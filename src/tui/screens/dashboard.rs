//! Dashboard screen — system overview at a glance.

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
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(3),
        ])
        .split(area);

    render_system_card(f, app, chunks[0]);
    render_stats_cards(f, app, chunks[1]);
    render_cascade_flow(f, app, chunks[2]);
}

fn render_system_card(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let uptime = app.obs.get("uptime_seconds").and_then(|v| v.as_i64()).unwrap_or(0);
    let text = vec![
        Line::from(vec![
            Span::styled("Architecture: ", Style::default().fg(colors::DIM)),
            Span::raw("Decentralized Signal Cascade"),
            Span::raw("  |  "),
            Span::styled("Uptime: ", Style::default().fg(colors::DIM)),
            Span::raw(format!("{}s", uptime)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(text).block(block),
        area,
    );
}

fn render_stats_cards(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    for (i, (label, key, color)) in [
        ("Fields", "fields", colors::GREEN),
        ("Processors", "processors", colors::ACCENT),
        ("Signal Types", "signal_types", colors::YELLOW),
    ].iter().enumerate() {
        let val = app.obs.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
        let block = Block::default()
            .title(format!(" {} ", label))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(*color));

        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);
        let p = Paragraph::new(Line::from(vec![
            Span::styled(format!("{}", val), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().fg(colors::TEXT));
        f.render_widget(p, inner);
    }
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

    for (name, count) in sorted.iter().take(10) {
        let c = count.as_u64().unwrap_or(0);
        let color = if c > 10 { colors::GREEN } else if c > 3 { colors::YELLOW } else { colors::DIM };
        let name_str = name.as_str();
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>4}x ", c), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(name_str),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No signals yet — inject an experience", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
