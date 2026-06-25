//! Fields screen — field registrations and state.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    layout::Rect,
    Frame,
};
use crate::tui::{app::TuiApp, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Fields ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(fields) = app.stats.get("field_names").and_then(|v| v.as_array()) {
        for (i, field) in fields.iter().enumerate() {
            let name = field.as_str().unwrap_or("?");
            let icon = match name {
                "memory" => "\u{1F4DA}",
                "identity" => "\u{1F9D1}",
                "executive" => "\u{1F3AF}",
                "awareness" => "\u{1F4A1}",
                "simulation" => "\u{1F52E}",
                "knowledge_graph" => "\u{1F578}",
                _ => "\u{25CF}",
            };
            let active = if i < 3 { "ACTIVE" } else { "ACTIVE" };
            let active_color = colors::GREEN;

            lines.push(Line::from(vec![
                Span::raw(format!("  {} ", icon)),
                Span::styled(format!("{:<20}", name), Style::default().fg(colors::TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(active, Style::default().fg(active_color)),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No fields registered", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
