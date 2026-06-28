//! Processors screen — processor list with descriptions, dispatch stats, latency.

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

    render_processor_list(f, app, chunks[0]);
    render_processor_detail(f, app, chunks[1]);
}

fn render_processor_list(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Processors ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::ACCENT));

    let mut lines: Vec<Line> = Vec::new();

    // Use stats endpoint for processor names (core detail doesn't have registry paths)
    let proc_names = app.stats.get("processor_names").and_then(|v| v.as_array()).cloned();

    if let Some(procs) = proc_names {
        for proc_val in &procs {
            let name = proc_val.as_str().unwrap_or("?");
            let arrow = match name {
                "episode" => "raw text → EpisodeRecorded",
                "belief" => "MemConsolidated → BeliefChanged",
                "identity" => "BeliefChanged → IdentityUpdated",
                "narrative" => "Episodes(3) → NarrativeGenerated",
                "goal" => "IdentityUpdated → GoalCreated/Completed",
                "attention" => "Episode/Curiosity → AttentionShifted",
                "curiosity" => "Episodes(5) → CuriosityDetected",
                "extraction" => "EpisodeRecorded → TriplesExtracted",
                "consolidation" => "Episodes(3/10) → MemoryConsolidated/PatternDetected",
                "reflection" => "Episodes(5) → IdentityUpdated/BeliefChanged (LLM deep)",
                "resolution" => "TriplesExtracted → EntityCreated/EdgeCreated",
                _ => "signals → signals",
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<15}", name), Style::default().fg(colors::TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(arrow, Style::default().fg(colors::DIM)),
            ]));
        }
    }

    if lines.is_empty() {
        if let Some(procs) = app.stats.get("processor_names").and_then(|v| v.as_array()) {
            for proc_val in procs {
                let name = proc_val.as_str().unwrap_or("?");
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", name), Style::default().fg(colors::TEXT).add_modifier(Modifier::BOLD)),
                ]));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No processors registered", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_processor_detail(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_processor_metrics(f, app, chunks[0]);
    render_processor_pipeline(f, app, chunks[1]);
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
            let color = if count > 0 { colors::GREEN } else { colors::DIM };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", name), Style::default().fg(color)),
                Span::styled(format!("{:>6}x ", count), Style::default().fg(colors::YELLOW)),
                Span::styled(format!("{:>4}ms avg", avg_ms), Style::default().fg(if avg_ms > 0 { colors::ACCENT } else { colors::DIM })),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  No dispatch metrics yet — waiting for signals", colors::DIM)));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_processor_pipeline(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Signal Pipeline ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let lines = vec![
        Line::from(vec![
            Span::styled("  Ingest Request ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ Episode Processor ", Style::default().fg(colors::ACCENT)),
            Span::styled("→ EpisodeRecorded", Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::raw("         ↓               ↓               ↓               ↓"),
        ]),
        Line::from(vec![
            Span::styled("  Consolidation  ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ Belief Processor ", Style::default().fg(colors::ACCENT)),
            Span::styled("→ Identity Proc.  ", Style::default().fg(colors::ACCENT)),
            Span::styled("→ Goal Processor", Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("  Extraction      ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ Resolution      ", Style::default().fg(colors::ACCENT)),
            Span::styled("→ GraphField ", Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::styled("  Attention       ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ AwarenessField ", Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::styled("  Curiosity(5ep)  ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ Attention       ", Style::default().fg(colors::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("  Narrative(3ep)  ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ MemoryField ", Style::default().fg(colors::GREEN)),
        ]),
        Line::from(vec![
            Span::styled("  Reflection(5ep) ", Style::default().fg(colors::YELLOW)),
            Span::styled("→ Identity/Belief ", Style::default().fg(colors::ACCENT)),
            Span::styled("  (LLM Deep)", Style::default().fg(colors::DIM)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("  All signals propagate through recursive cascade", Style::default().fg(colors::DIM)),
            Span::raw(" "),
            Span::styled("until equilibrium (no more emissions)", Style::default().fg(colors::DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}
