//! Application state, navigation, and key handling. Rendering lives in `ui.rs`.
//!
//! Architecture mirrors curlyos-tui: a background worker thread owns the blocking
//! HTTP client and communicates via channels. The UI thread never blocks on I/O.

use crate::tui::api::*;
use crate::tui::worker::{Req, Resp};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use std::sync::mpsc::Sender;

pub const TABS: [&str; 5] = ["Dashboard", "Memory", "Fields", "Signals", "System"];
pub const FIELD_SUBS: [&str; 9] = [
    "Overview", "Identity", "Memory", "Agency", "Awareness",
    "Reasoning", "Simulation", "Graph", "Core",
];
pub const MEMORY_SUBS: [&str; 3] = ["Browse", "Episodes", "History"];
pub const SIGNAL_SUBS: [&str; 3] = ["Types", "Distribution", "Processors"];
pub const SYSTEM_SUBS: [&str; 5] = ["Config", "Plugins", "Observability", "Cascade", "Log"];

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Dashboard,
    Memory,
    Fields,
    Signals,
    System,
}

const TAB_ORDER: [Tab; 5] = [Tab::Dashboard, Tab::Memory, Tab::Fields, Tab::Signals, Tab::System];

impl Tab {
    pub fn index(self) -> usize { TAB_ORDER.iter().position(|&t| t == self).unwrap_or(0) }
    fn from_index(i: usize) -> Tab { TAB_ORDER[i.min(TAB_ORDER.len() - 1)] }
    fn is_live(self) -> bool { matches!(self, Tab::Dashboard | Tab::System) }
    pub fn sub_labels(self) -> &'static [&'static str] {
        match self {
            Tab::Memory => &MEMORY_SUBS,
            Tab::Fields => &FIELD_SUBS,
            Tab::Signals => &SIGNAL_SUBS,
            Tab::System => &SYSTEM_SUBS,
            _ => &[],
        }
    }
}

// ── Selection helper ──────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Sel {
    pub state: ListState,
    pub len: usize,
}

impl Sel {
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        self.state.select(if len == 0 { None } else { Some(self.state.selected().unwrap_or(0).min(len - 1)) });
    }
    pub fn selected(&self) -> Option<usize> { self.state.selected() }
    pub fn next(&mut self) {
        if self.len == 0 { return; }
        let i = self.state.selected().map_or(0, |i| (i + 1).min(self.len - 1));
        self.state.select(Some(i));
    }
    pub fn prev(&mut self) {
        if self.len == 0 { return; }
        let i = self.state.selected().map_or(0, |i| i.saturating_sub(1));
        self.state.select(Some(i));
    }
}

// ── Overlay system ───────────────────────────────────────────────────────────

pub enum Overlay {
    None,
    Help,
    Form(Form),
}

pub struct Form {
    pub kind: FormKind,
    pub title: String,
    pub fields: Vec<FormField>,
    pub active: usize,
}

pub enum FormKind {
    Ingest,
    Inject,
}

pub struct FormField {
    pub label: String,
    pub value: String,
}

// ── Main App struct ──────────────────────────────────────────────────────────

pub struct App {
    pub tx: Sender<Req>,
    pub tab: Tab,
    pub sub_idx: usize,
    pub overlay: Overlay,
    pub inflight: usize,
    pub status: Option<(String, bool)>, // (message, is_error)
    pub should_quit: bool,
    pub base: String,
    pub frame: u64,

    // Dashboard
    pub health: Option<Health>,
    pub stats: Stats,
    pub observability: Option<Observability>,

    // Fields — detail JSON for each field
    pub field_details: std::collections::HashMap<String, serde_json::Value>,

    // Signals
    pub signal_types: Vec<SignalTypeItem>,
    pub signal_history: Vec<SignalHistoryEntry>,
    pub history_sel: Sel,

    // Observability
    pub processor_metrics: Vec<ProcessorMetric>,
    pub signal_metrics: Option<SignalMetricsData>,
    pub cascade_trace: Option<CascadeTraceData>,
    pub capabilities: Vec<Capability>,

    // System
    pub plugins: Vec<PluginSummary>,
    pub config: Option<SystemConfig>,

    // Log entries (local)
    pub log_entries: Vec<String>,

    // Inject form state
    pub inject_signal_type: String,
}

impl App {
    pub fn new(tx: Sender<Req>, base: String) -> Self {
        App {
            tx,
            tab: Tab::Dashboard,
            sub_idx: 0,
            overlay: Overlay::None,
            inflight: 0,
            status: None,
            should_quit: false,
            base,
            frame: 0,
            health: None,
            stats: Stats::default(),
            observability: None,
            field_details: std::collections::HashMap::new(),
            signal_types: vec![],
            signal_history: vec![],
            history_sel: Sel::default(),
            processor_metrics: vec![],
            signal_metrics: None,
            cascade_trace: None,
            capabilities: vec![],
            plugins: vec![],
            config: None,
            log_entries: vec![],
            inject_signal_type: String::new(),
        }
    }

    fn send(&mut self, req: Req) {
        self.inflight += 1;
        let _ = self.tx.send(req);
    }

    pub fn loading(&self) -> bool { self.inflight > 0 }

    fn add_log(&mut self, msg: impl Into<String>) {
        let entry = format!("[{}] {}", chrono::Utc::now().format("%H:%M:%S"), msg.into());
        self.log_entries.push(entry);
        if self.log_entries.len() > 200 { self.log_entries.remove(0); }
    }

    /// Auto-refresh live views on timer. Called by event loop.
    pub fn auto_refresh(&mut self) {
        if !(self.tab.is_live() && self.inflight == 0 && matches!(self.overlay, Overlay::None)) {
            return;
        }
        match self.tab {
            Tab::Dashboard => self.refresh(),
            Tab::System => self.refresh(),
            _ => {}
        }
    }

    /// Load data for the current tab + sub-view.
    pub fn refresh(&mut self) {
        self.status = None;
        match self.tab {
            Tab::Dashboard => {
                self.send(Req::Dashboard);
                self.send(Req::Stats);
            }
            Tab::Memory => match self.sub_idx {
                0 => self.send(Req::FieldDetail("memory".to_string())),
                1 => self.send(Req::Episodes),
                2 => self.send(Req::SignalHistory { limit: 100, field: None }),
                _ => {}
            },
            Tab::Fields => {
                let name = FIELD_SUBS[self.sub_idx].to_lowercase();
                if self.sub_idx == 0 {
                    for f in ["identity", "memory", "agency", "awareness", "reasoning", "simulation", "graph", "core"] {
                        self.send(Req::FieldDetail(f.to_string()));
                    }
                } else {
                    self.send(Req::FieldDetail(name));
                }
            }
            Tab::Signals => match self.sub_idx {
                0 => self.send(Req::SignalTypes),
                1 => self.send(Req::SignalMetrics),
                2 => self.send(Req::ProcessorMetrics),
                _ => {}
            },
            Tab::System => match self.sub_idx {
                0 => self.send(Req::Config),
                1 => self.send(Req::Plugins),
                2 => { self.send(Req::ObservabilityOverview); self.send(Req::Stats); }
                3 => self.send(Req::CascadeTrace),
                _ => {}
            },
        }
    }

    pub fn apply(&mut self, resp: Resp) {
        self.inflight = self.inflight.saturating_sub(1);
        match resp {
            Resp::Dashboard(d) => {
                self.health = Some(d.health);
                self.observability = Some(d.observability);
            }
            Resp::Health(h) => self.health = Some(*h),
            Resp::Stats(s) => self.stats = *s,
            Resp::SignalTypes(v) => self.signal_types = v,
            Resp::SignalHistory(v) => {
                self.history_sel.set_len(v.len());
                self.signal_history = v;
            }
            Resp::ObservabilityOverview(o) => {
                let o = *o;
                self.observability = Some(o.clone());
                self.stats.signals_total = o.signals_total.unwrap_or(0);
                self.stats.cascade_cycles = o.cascade_cycles;
            }
            Resp::ProcessorMetrics(v) => self.processor_metrics = v,
            Resp::SignalMetrics(m) => self.signal_metrics = Some(*m),
            Resp::CascadeTrace(t) => self.cascade_trace = Some(*t),
            Resp::Capabilities(v) => self.capabilities = v,
            Resp::Plugins(v) => self.plugins = v,
            Resp::Config(c) => self.config = Some(*c),
            Resp::FieldDetail(name, value) => {
                self.field_details.insert(name, value);
            }
            Resp::ActionOk { msg, refresh } => {
                self.status = Some((msg.clone(), false));
                self.add_log(&msg);
                if refresh { self.refresh(); }
            }
            Resp::Error(e) => {
                self.status = Some((e.clone(), true));
                self.add_log(format!("ERROR: {e}"));
            }
        }
    }

    // ── Key handling ──────────────────────────────────────────────────────

    pub fn on_key(&mut self, key: KeyEvent) {
        // Overlay consumes keys first
        match &mut self.overlay {
            Overlay::Help => { self.overlay = Overlay::None; return; }
            Overlay::Form(_) => { self.handle_form(key); return; }
            Overlay::None => {}
        }

        self.status = None;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if ctrl => self.should_quit = true,
            KeyCode::Char('?') => self.overlay = Overlay::Help,
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Esc => {}
            KeyCode::Tab => self.switch_tab((self.tab.index() + 1) % TABS.len()),
            KeyCode::BackTab => self.switch_tab((self.tab.index() + TABS.len() - 1) % TABS.len()),
            KeyCode::Char(c @ '1'..='5') => self.switch_tab(c as usize - '1' as usize),
            KeyCode::Char('l') | KeyCode::Right => self.cycle_sub(1),
            KeyCode::Char('h') | KeyCode::Left => self.cycle_sub(-1),
            KeyCode::Down | KeyCode::Char('j') => self.cur_sel().next(),
            KeyCode::Up | KeyCode::Char('k') => self.cur_sel().prev(),
            KeyCode::Enter => self.on_enter(),
            KeyCode::Char('a') => self.open_form(FormKind::Ingest),
            KeyCode::Char('i') if self.tab == Tab::Signals && self.sub_idx == 2 => {}
            _ => {}
        }
    }

    fn switch_tab(&mut self, i: usize) {
        self.tab = Tab::from_index(i);
        self.sub_idx = 0;
        self.refresh();
    }

    fn cycle_sub(&mut self, dir: i32) {
        let n = self.tab.sub_labels().len() as i32;
        if n == 0 { return; }
        self.sub_idx = ((self.sub_idx as i32 + dir).rem_euclid(n)) as usize;
        self.refresh();
    }

    fn cur_sel(&mut self) -> &mut Sel {
        match self.tab {
            Tab::Signals if self.sub_idx == 1 => &mut self.history_sel,
            _ => &mut self.history_sel, // fallback
        }
    }

    fn on_enter(&mut self) {
        // Detail selection handled per-tab
    }

    fn open_form(&mut self, kind: FormKind) {
        let form = match kind {
            FormKind::Ingest => Form {
                kind,
                title: "Ingest experience".into(),
                fields: vec![FormField { label: "content".into(), value: String::new() }],
                active: 0,
            },
            FormKind::Inject => Form {
                kind,
                title: "Inject signal".into(),
                fields: vec![
                    FormField { label: "signal_type".into(), value: self.inject_signal_type.clone() },
                    FormField { label: "payload (JSON)".into(), value: "{}".into() },
                ],
                active: 0,
            },
        };
        self.overlay = Overlay::Form(form);
    }

    fn handle_form(&mut self, key: KeyEvent) {
        // Esc to close
        if matches!(key.code, KeyCode::Esc) {
            self.overlay = Overlay::None;
            return;
        }

        let submit = key.code == KeyCode::Enter
            && !matches!(self.overlay, Overlay::Form(Form { kind: FormKind::Ingest, .. }));
        let ingest_submit = matches!(self.overlay, Overlay::Form(Form { kind: FormKind::Ingest, .. }))
            && key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Tab => {
                if let Overlay::Form(f) = &mut self.overlay {
                    f.active = (f.active + 1) % f.fields.len();
                }
                return;
            }
            KeyCode::BackTab => {
                if let Overlay::Form(f) = &mut self.overlay {
                    f.active = (f.active + f.fields.len() - 1) % f.fields.len();
                }
                return;
            }
            KeyCode::Backspace => {
                if let Overlay::Form(f) = &mut self.overlay {
                    f.fields[f.active].value.pop();
                }
                return;
            }
            KeyCode::Char(c) => {
                if !(submit || ingest_submit) {
                    if let Overlay::Form(f) = &mut self.overlay {
                        f.fields[f.active].value.push(c);
                    }
                    return;
                }
            }
            KeyCode::Enter => {
                if let Overlay::Form(f) = &mut self.overlay {
                    if matches!(f.kind, FormKind::Ingest) {
                        f.fields[0].value.push('\n');
                        return;
                    }
                }
            }
            _ => return,
        }

        if submit || ingest_submit {
            self.submit_form();
        }
    }

    fn submit_form(&mut self) {
        let form = match std::mem::replace(&mut self.overlay, Overlay::None) {
            Overlay::Form(f) => f,
            _ => return,
        };
        match form.kind {
            FormKind::Ingest => {
                let content = form.fields[0].value.trim().to_string();
                if !content.is_empty() {
                    self.send(Req::Ingest(content));
                    self.add_log("Sent ingest request");
                }
            }
            FormKind::Inject => {
                let signal_type = form.fields[0].value.trim().to_string();
                let payload: serde_json::Value = serde_json::from_str(form.fields[1].value.trim())
                    .unwrap_or(serde_json::json!({}));
                if !signal_type.is_empty() {
                    self.inject_signal_type = signal_type.clone();
                    self.send(Req::Inject { signal_type, payload });
                    self.add_log("Sent signal inject");
                }
            }
        }
    }
}
