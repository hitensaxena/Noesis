//! TUI rendering and event handling.

use std::time::Duration;
use anyhow::Result;
use ratatui::{
    Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use super::app::{Screen, TuiApp, DETAIL_NAMES};
use super::screens;
use super::colors;

/// Run the TUI event loop.
pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut TuiApp) -> Result<()> {
    // Spawn a background refresh task
    let refresh_interval = app.refresh_interval;

    loop {
        // Draw the UI
        terminal.draw(|f| render(f, app))?;

        // Handle events with a timeout for periodic refresh
        if event::poll(Duration::from_millis(500)).unwrap_or(false) {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('l') | KeyCode::Right => {
                            if app.screen == Screen::Detail {
                                app.next_detail();
                            } else {
                                app.next_screen();
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if app.screen == Screen::Detail {
                                app.prev_detail();
                            } else {
                                app.prev_screen();
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if app.screen == Screen::Detail {
                                app.next_detail();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if app.screen == Screen::Detail {
                                app.prev_detail();
                            }
                        }
                        KeyCode::Char('r') => app.refresh().await,
                        KeyCode::Char('a') => {
                            app.toggle_auto_refresh();
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            app.set_refresh_interval(app.refresh_interval_secs() + 1);
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            app.set_refresh_interval(app.refresh_interval_secs().saturating_sub(1).max(1));
                        }
                        KeyCode::Enter => {
                            app.refresh().await;
                        }
                        KeyCode::Tab => app.next_screen(),
                        _ => {}
                    }
                }
            }
        }

        // Periodic refresh (only if auto-refresh is enabled)
        if app.auto_refresh && app.last_refresh.elapsed() >= refresh_interval {
            app.refresh().await;
        }
    }

    Ok(())
}

/// Render the full TUI layout.
fn render(f: &mut Frame, app: &TuiApp) {
    let size = f.area();
    if size.width < 60 || size.height < 20 {
        let text = "Terminal too small — resize to at least 60x20";
        f.render_widget(Paragraph::new(text).style(Style::default().fg(colors::RED)), size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(1),  // tabs
            Constraint::Min(5),     // content
            Constraint::Length(1),  // status bar
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_tabs(f, app, chunks[1]);
    render_content(f, app, chunks[2]);
    render_status_bar(f, app, chunks[3]);
}

/// Render the header bar.
fn render_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::PRIMARY));

    let title = format!("Noesis v0.1.0 — {}", app.screen.name());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Noesis ", Style::default().fg(colors::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("|", Style::default().fg(colors::DIM)),
        Span::raw(format!(" {} ", title)),
        Span::styled("|", Style::default().fg(colors::DIM)),
        Span::styled(format!(" Fields: {} ", signal_count(app, "fields")), Style::default().fg(colors::GREEN)),
        Span::styled("|", Style::default().fg(colors::DIM)),
        Span::styled(format!(" Procs: {} ", signal_count(app, "processors")), Style::default().fg(colors::ACCENT)),
        Span::styled("|", Style::default().fg(colors::DIM)),
        Span::styled(format!(" Signals: {} ", signal_count(app, "signal_types")), Style::default().fg(colors::YELLOW)),
    ]))
    .style(Style::default().fg(colors::TEXT));

    f.render_widget(header, inner);
}

/// Render the tab navigation.
fn render_tabs(f: &mut Frame, app: &TuiApp, area: Rect) {
    let titles: Vec<Line> = Screen::all()
        .iter()
        .map(|s| {
            let selected = *s == app.screen;
            let icon = s.icon();
            let name = s.name();
            if selected {
                Line::from(vec![
                    Span::styled(format!(" {} {} ", icon, name), Style::default()
                        .fg(colors::PRIMARY)
                        .add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!(" {} {} ", icon, name), Style::default().fg(colors::DIM)),
                ])
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(tabs, area);
}

/// Render the main content area based on the selected screen.
fn render_content(f: &mut Frame, app: &TuiApp, area: Rect) {
    match app.screen {
        Screen::Dashboard => screens::dashboard::render(f, app, area),
        Screen::Signals => screens::signals::render(f, app, area),
        Screen::Fields => screens::fields::render(f, app, area),
        Screen::Processors => screens::processors::render(f, app, area),
        Screen::Observability => screens::observability::render(f, app, area),
        Screen::Log => screens::log::render(f, app, area),
        Screen::Detail => screens::detail::render(f, app, area),
        Screen::Settings => screens::settings::render(f, app, area),
    }
}

/// Render the status bar at the bottom.
fn render_status_bar(f: &mut Frame, app: &TuiApp, area: Rect) {
    let screen_label = match app.screen {
        Screen::Detail => format!("{}({})", app.screen.name(), DETAIL_NAMES[app.detail_index]),
        Screen::Settings => {
            let auto = if app.auto_refresh { "ON" } else { "OFF" };
            format!("Settings({}s, auto:{})", app.refresh_interval_secs(), auto)
        }
        _ => app.screen.name().to_string(),
    };
    let keys_hint = match app.screen {
        Screen::Detail => " · ↑/↓ Detail · ",
        Screen::Settings => " · +/- Interval · a Refresh · ",
        _ => " · ",
    };
    let status = format!(
        " {} | {} | Keys: ←/→ Tab{}r Refresh · q Quit",
        app.status_message, screen_label, keys_hint,
    );
    let bar = Paragraph::new(Line::from(vec![
        Span::raw(status),
    ]))
    .style(Style::default().fg(colors::DIM).bg(colors::BG));

    f.render_widget(bar, area);
}

fn signal_count(app: &TuiApp, key: &str) -> i64 {
    app.obs.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}
