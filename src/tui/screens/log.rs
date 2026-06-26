//! Log screen — real-time event stream with color-coded entries.

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
        .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
        .split(area);

    render_log_entries(f, app, chunks[0]);
    render_log_status(f, app, chunks[1]);
}

fn render_log_entries(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Event Log ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::DIM));

    let lines: Vec<Line> = app.log_entries.iter().rev().take(100).map(|entry| {
        // Color-code key log types
        if entry.contains("Connected") || entry.contains("OK") {
            Line::from(Span::styled(format!("  {}", entry), Style::default().fg(colors::GREEN)))
        } else if entry.contains("Detail:") {
            Line::from(Span::styled(format!("  {}", entry), Style::default().fg(colors::ACCENT)))
        } else if entry.contains("error") || entry.contains("Error") {
            Line::from(Span::styled(format!("  {}", entry), Style::default().fg(colors::RED)))
        } else if entry.contains("Refresh") || entry.contains("Auto-refresh") {
            Line::from(Span::styled(format!("  {}", entry), Style::default().fg(colors::YELLOW)))
        } else {
            Line::from(Span::raw(format!("  {}", entry)))
        }
    }).collect();

    let content = if lines.is_empty() {
        vec![Line::from(Span::styled("  No log entries yet. Log entries appear as you interact with the TUI.", colors::DIM))]
    } else {
        lines
    };

    f.render_widget(Paragraph::new(content).block(block), area);
}

fn render_log_status(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let entry_count = app.log_entries.len();
    let auto = if app.auto_refresh { "ON" } else { "OFF" };

    let lines = vec![
        Line::from(vec![
            Span::styled("Entries:", Style::default().fg(colors::DIM)),
            Span::raw(format!(" {}", entry_count)),
        ]),
        Line::from(vec![
            Span::styled("Last:", Style::default().fg(colors::DIM)),
            Span::raw(" "),
            Span::styled(app.log_entries.last().map(|s| {
                if s.len() > 50 { format!("{}...", &s[..50]) } else { s.clone() }
            }).unwrap_or_default(), Style::default().fg(colors::DIM)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Refresh: ", Style::default().fg(colors::DIM)),
            Span::styled(auto, Style::default().fg(if app.auto_refresh { colors::GREEN } else { colors::YELLOW })),
            Span::raw(" "),
            Span::styled(format!("({}s)", app.refresh_interval_secs()), Style::default().fg(colors::DIM)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Active screen:", Style::default().fg(colors::DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!(" {}", app.screen.name()), Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}
