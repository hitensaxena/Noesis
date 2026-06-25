//! Signals screen — signal types and per-type counts.

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
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_signal_types(f, app, chunks[0]);
    render_signal_counts(f, app, chunks[1]);
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
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", name), Style::default().fg(colors::ACCENT)),
                Span::raw(" "),
                Span::styled(desc, Style::default().fg(colors::DIM)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
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
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
