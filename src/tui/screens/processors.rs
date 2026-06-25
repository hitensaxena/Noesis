//! Processors screen — processor list with dispatch stats and latency.

use ratatui::{
    style::{Modifier, Style, Stylize},
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

    render_processor_list(f, app, chunks[0]);
    render_processor_metrics(f, app, chunks[1]);
}

fn render_processor_list(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Processors ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(procs) = app.stats.get("processor_names").and_then(|v| v.as_array()) {
        for proc_name in procs {
            let name = proc_name.as_str().unwrap_or("?");
            let arrow = match name {
                "episode" => "raw text -> EpisodeRecorded",
                "belief" => "Memories -> BeliefChanged",
                "identity" => "Beliefs -> IdentityUpdated",
                "narrative" => "Episodes -> NarrativeGenerated",
                "goal" => "Identity -> GoalCreated/Completed",
                "attention" => "Signals -> AttentionShifted",
                "curiosity" => "Episodes -> CuriosityDetected",
                "extraction" => "Episodes -> TriplesExtracted",
                "consolidation" => "Episodes -> MemoryConsolidated",
                _ => "signals -> signals",
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<15}", name), Style::default().fg(colors::TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(arrow, Style::default().fg(colors::DIM)),
            ]));
        }
    }
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_processor_metrics(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Dispatch Metrics ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::GREEN));

    let mut lines: Vec<Line> = Vec::new();
    if let Some(processors) = app.processor_metrics.as_object() {
        let mut sorted: Vec<_> = processors.iter().collect();
        sorted.sort_by(|a, b| b.1.get("count").and_then(|v| v.as_u64()).unwrap_or(0)
            .cmp(&a.1.get("count").and_then(|v| v.as_u64()).unwrap_or(0)));

        for (name, metrics) in &sorted {
            let count = metrics.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let avg_ms = metrics.get("avg_latency_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", name), Style::default().fg(colors::TEXT)),
                Span::styled(format!("{:>6}x ", count), Style::default().fg(colors::YELLOW)),
                Span::styled(format!("{:>4}ms avg", avg_ms), Style::default().fg(colors::DIM)),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No processor metrics yet", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
