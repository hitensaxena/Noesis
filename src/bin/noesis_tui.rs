//! Noesis TUI — terminal dashboard for the Noesis cognitive OS.
//!
//! Architecture: the UI thread owns terminal + state and stays responsive by
//! delegating every network call to a single background worker thread (worker.rs)
//! over channels. The worker drains requests FIFO, so responses arrive in request
//! order — no staleness races.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use noesis::tui::api::Client;
use noesis::tui::app::App;
use noesis::tui::ui;
use noesis::tui::worker;

fn main() -> Result<()> {
    let base = std::env::var("NOESIS_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8647".to_string());

    // Channels: UI → worker (Req), worker → UI (Resp)
    let (req_tx, req_rx) = mpsc::channel();
    let (resp_tx, resp_rx) = mpsc::channel();
    worker::spawn(Client::new(base.clone()), req_rx, resp_tx);

    let mut app = App::new(req_tx, base);
    app.refresh(); // initial load

    // Terminal setup
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(out))?;

    let res = run(&mut term, &mut app, &resp_rx);

    // Teardown (always, even on error)
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;

    if let Err(e) = res {
        eprintln!("error: {e:#}");
    }
    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    app: &mut App,
    resp_rx: &mpsc::Receiver<worker::Resp>,
) -> Result<()> {
    const TICK: Duration = Duration::from_secs(3);
    let mut last_tick = Instant::now();

    loop {
        app.frame = app.frame.wrapping_add(1);
        term.draw(|f| ui::draw(f, app))?;

        // Drain any worker responses without blocking
        while let Ok(resp) = resp_rx.try_recv() {
            app.apply(resp);
        }

        // Poll for input with a short timeout
        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key);
                }
            }
        }

        // Auto-refresh live views on timer
        if last_tick.elapsed() >= TICK {
            app.auto_refresh();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
