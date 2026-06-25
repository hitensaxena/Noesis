//! Log screen — real-time event stream.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    layout::Rect,
    Frame,
};
use crate::tui::{app::TuiApp, colors};

pub fn render(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Event Log ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::DIM));

    let lines: Vec<Line> = app.log_entries.iter().rev().take(50).map(|entry| {
        Line::from(Span::raw(format!("  {}", entry)))
    }).collect();

    let content = if lines.is_empty() {
        vec![Line::from(Span::styled("  No log entries yet", colors::DIM))]
    } else {
        lines
    };

    f.render_widget(Paragraph::new(content).block(block), area);
}
